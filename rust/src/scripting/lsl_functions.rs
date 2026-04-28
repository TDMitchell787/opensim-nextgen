//! LSL built-in functions implementation

use anyhow::{anyhow, Result};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{LSLRotation, LSLValue, LSLVector, ScriptContext};
use crate::{
    asset::AssetManager, network::grid_events::GridEventManager, region::RegionManager,
    scripting::executor::ScriptAction,
};

const EOF_MARKER: &str = "\nEnd of notecard\n";

/// LSL function executor
pub struct LSLFunctions {
    region_manager: Arc<RegionManager>,
    asset_manager: Arc<AssetManager>,
    grid_event_manager: Option<Arc<GridEventManager>>,
    action_queue: Arc<parking_lot::Mutex<Vec<(Uuid, ScriptAction)>>>,
}

impl LSLFunctions {
    pub fn new(
        region_manager: Arc<RegionManager>,
        asset_manager: Arc<AssetManager>,
        grid_event_manager: Option<Arc<GridEventManager>>,
        action_queue: Arc<parking_lot::Mutex<Vec<(Uuid, ScriptAction)>>>,
    ) -> Self {
        Self {
            region_manager,
            asset_manager,
            grid_event_manager,
            action_queue,
        }
    }

    /// Execute an LSL function
    pub async fn execute_function(
        &self,
        function_name: &str,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        debug!(
            "Executing LSL function: {} with {} args",
            function_name,
            args.len()
        );

        match function_name {
            // Chat and communication
            "llSay" => self.ll_say(args, context).await,
            "llShout" => self.ll_shout(args, context).await,
            "llWhisper" => self.ll_whisper(args, context).await,
            "llOwnerSay" => self.ll_owner_say(args, context).await,
            "llRegionSay" => self.ll_region_say(args, context).await,
            "llListen" => self.ll_listen(args, context).await,
            "llListenControl" => self.ll_listen_control(args, context).await,
            "llListenRemove" => self.ll_listen_remove(args, context).await,

            // Object manipulation
            "llSetText" => self.ll_set_text(args, context).await,
            "llSetObjectName" => self.ll_set_object_name(args, context).await,
            "llSetObjectDesc" => self.ll_set_object_desc(args, context).await,
            "llGetObjectName" => self.ll_get_object_name(args, context).await,
            "llGetObjectDesc" => self.ll_get_object_desc(args, context).await,
            "llGetKey" => self.ll_get_key(args, context).await,
            "llGetOwner" => self.ll_get_owner(args, context).await,

            // Position and movement
            "llGetPos" => self.ll_get_pos(args, context).await,
            "llSetPos" => self.ll_set_pos(args, context).await,
            "llGetRot" => self.ll_get_rot(args, context).await,
            "llSetRot" => self.ll_set_rot(args, context).await,
            "llGetVel" => self.ll_get_vel(args, context).await,
            "llSetVelocity" => self.ll_set_velocity(args, context).await,
            "llGetRegionCorner" => self.ll_get_region_corner(args, context).await,
            "llGetRegionName" => self.ll_get_region_name(args, context).await,

            // Math functions
            "llAbs" => self.ll_abs(args, context).await,
            "llAcos" => self.ll_acos(args, context).await,
            "llAsin" => self.ll_asin(args, context).await,
            "llAtan2" => self.ll_atan2(args, context).await,
            "llCeil" => self.ll_ceil(args, context).await,
            "llCos" => self.ll_cos(args, context).await,
            "llFabs" => self.ll_fabs(args, context).await,
            "llFloor" => self.ll_floor(args, context).await,
            "llFrand" => self.ll_frand(args, context).await,
            "llLog" => self.ll_log(args, context).await,
            "llLog10" => self.ll_log10(args, context).await,
            "llPow" => self.ll_pow(args, context).await,
            "llRound" => self.ll_round(args, context).await,
            "llSin" => self.ll_sin(args, context).await,
            "llSqrt" => self.ll_sqrt(args, context).await,
            "llTan" => self.ll_tan(args, context).await,

            // Vector and rotation functions
            "llVecDist" => self.ll_vec_dist(args, context).await,
            "llVecMag" => self.ll_vec_mag(args, context).await,
            "llVecNorm" => self.ll_vec_norm(args, context).await,
            "llAngleBetween" => self.ll_angle_between(args, context).await,
            "llEuler2Rot" => self.ll_euler2rot(args, context).await,
            "llRot2Euler" => self.ll_rot2euler(args, context).await,
            "llAxes2Rot" => self.ll_axes2rot(args, context).await,
            "llRot2Axis" => self.ll_rot2axis(args, context).await,
            "llRot2Angle" => self.ll_rot2angle(args, context).await,

            // String functions
            "llStringLength" => self.ll_string_length(args, context).await,
            "llSubStringIndex" => self.ll_sub_string_index(args, context).await,
            "llGetSubString" => self.ll_get_sub_string(args, context).await,
            "llInsertString" => self.ll_insert_string(args, context).await,
            "llDeleteSubString" => self.ll_delete_sub_string(args, context).await,
            "llToLower" => self.ll_to_lower(args, context).await,
            "llToUpper" => self.ll_to_upper(args, context).await,

            // List functions
            "llListLength" => self.ll_list_length(args, context).await,
            "llList2String" => self.ll_list2string(args, context).await,
            "llList2Integer" => self.ll_list2integer(args, context).await,
            "llList2Float" => self.ll_list2float(args, context).await,
            "llList2Key" => self.ll_list2key(args, context).await,
            "llList2Vector" => self.ll_list2vector(args, context).await,
            "llList2Rot" => self.ll_list2rot(args, context).await,
            "llListInsertList" => self.ll_list_insert_list(args, context).await,
            "llDeleteSubList" => self.ll_delete_sub_list(args, context).await,
            "llGetListLength" => self.ll_get_list_length(args, context).await,

            // Type conversion
            "llList2CSV" => self.ll_list2csv(args, context).await,
            "llCSV2List" => self.ll_csv2list(args, context).await,
            "llDumpList2String" => self.ll_dump_list2string(args, context).await,
            "llParseString2List" => self.ll_parse_string2list(args, context).await,

            // Time functions
            "llGetTimestamp" => self.ll_get_timestamp(args, context).await,
            "llGetUnixTime" => self.ll_get_unix_time(args, context).await,
            "llSetTimerEvent" => self.ll_set_timer_event(args, context).await,

            // Inventory functions
            "llGetInventoryNumber" => self.ll_get_inventory_number(args, context).await,
            "llGetInventoryName" => self.ll_get_inventory_name(args, context).await,
            "llGetInventoryType" => self.ll_get_inventory_type(args, context).await,
            "llGetInventoryKey" => self.ll_get_inventory_key(args, context).await,

            // HTTP functions
            "llHTTPRequest" => self.ll_http_request(args, context).await,
            "llHTTPResponse" => self.ll_http_response(args, context).await,

            // Sensor functions
            "llSensor" => self.ll_sensor(args, context).await,
            "llSensorRepeat" => self.ll_sensor_repeat(args, context).await,
            "llSensorRemove" => self.ll_sensor_remove(args, context).await,

            // Debug and utility
            "llGetScriptName" => self.ll_get_script_name(args, context).await,
            "llResetScript" => self.ll_reset_script(args, context).await,
            "llSleep" => self.ll_sleep(args, context).await,

            // Physics and movement
            "llApplyImpulse" => self.ll_apply_impulse(args, context).await,
            "llApplyRotationalImpulse" => self.ll_apply_rotational_impulse(args, context).await,
            "llSetForce" => self.ll_set_force(args, context).await,
            "llGetForce" => self.ll_get_force(args, context).await,
            "llSetTorque" => self.ll_set_torque(args, context).await,
            "llGetTorque" => self.ll_get_torque(args, context).await,
            "llSetBuoyancy" => self.ll_set_buoyancy(args, context).await,
            "llSetHoverHeight" => self.ll_set_hover_height(args, context).await,
            "llStopHover" => self.ll_stop_hover(args, context).await,
            "llMoveToTarget" => self.ll_move_to_target(args, context).await,
            "llStopMoveToTarget" => self.ll_stop_move_to_target(args, context).await,
            "llPushObject" => self.ll_push_object(args, context).await,
            "llGetAccel" => self.ll_get_accel(args, context).await,
            "llGetOmega" => self.ll_get_omega(args, context).await,
            "llTargetOmega" => self.ll_target_omega(args, context).await,
            "llSetAngularVelocity" => self.ll_set_angular_velocity(args, context).await,
            "llGetMass" => self.ll_get_mass(args, context).await,
            "llGetMassMKS" => self.ll_get_mass_mks(args, context).await,
            "llGetObjectMass" => self.ll_get_object_mass(args, context).await,
            "llGroundRepel" => self.ll_ground_repel(args, context).await,
            "llSetForceAndTorque" => self.ll_set_force_and_torque(args, context).await,
            "llSetPhysicsMaterial" => self.ll_set_physics_material(args, context).await,
            "llGetPhysicsMaterial" => self.ll_get_physics_material(args, context).await,

            // Animation functions
            "llStartAnimation" => self.ll_start_animation(args, context).await,
            "llStopAnimation" => self.ll_stop_animation(args, context).await,
            "llGetAnimation" => self.ll_get_animation(args, context).await,
            "llGetAnimationList" => self.ll_get_animation_list(args, context).await,
            "llSetAnimationOverride" => self.ll_set_animation_override(args, context).await,
            "llGetAnimationOverride" => self.ll_get_animation_override(args, context).await,
            "llResetAnimationOverride" => self.ll_reset_animation_override(args, context).await,
            "llStartObjectAnimation" => self.ll_start_object_animation(args, context).await,
            "llStopObjectAnimation" => self.ll_stop_object_animation(args, context).await,
            "llGetObjectAnimationNames" => self.ll_get_object_animation_names(args, context).await,

            // Sound functions
            "llPlaySound" => self.ll_play_sound(args, context).await,
            "llLoopSound" => self.ll_loop_sound(args, context).await,
            "llLoopSoundMaster" => self.ll_loop_sound_master(args, context).await,
            "llLoopSoundSlave" => self.ll_loop_sound_slave(args, context).await,
            "llPlaySoundSlave" => self.ll_play_sound_slave(args, context).await,
            "llStopSound" => self.ll_stop_sound(args, context).await,
            "llTriggerSound" => self.ll_trigger_sound(args, context).await,
            "llTriggerSoundLimited" => self.ll_trigger_sound_limited(args, context).await,
            "llPreloadSound" => self.ll_preload_sound(args, context).await,
            "llSound" => self.ll_sound(args, context).await,
            "llSoundPreload" => self.ll_sound_preload(args, context).await,
            "llAdjustSoundVolume" => self.ll_adjust_sound_volume(args, context).await,
            "llSetSoundQueueing" => self.ll_set_sound_queueing(args, context).await,
            "llSetSoundRadius" => self.ll_set_sound_radius(args, context).await,
            "llLinkPlaySound" => self.ll_link_play_sound(args, context).await,
            "llLinkStopSound" => self.ll_link_stop_sound(args, context).await,
            "llLinkAdjustSoundVolume" => self.ll_link_adjust_sound_volume(args, context).await,
            "llLinkSetSoundQueueing" => self.ll_link_set_sound_queueing(args, context).await,
            "llLinkSetSoundRadius" => self.ll_link_set_sound_radius(args, context).await,
            "llCollisionSound" => self.ll_collision_sound(args, context).await,

            // Texture and visual functions
            "llSetTexture" => self.ll_set_texture(args, context).await,
            "llGetTexture" => self.ll_get_texture(args, context).await,
            "llSetColor" => self.ll_set_color(args, context).await,
            "llGetColor" => self.ll_get_color(args, context).await,
            "llSetAlpha" => self.ll_set_alpha(args, context).await,
            "llGetAlpha" => self.ll_get_alpha(args, context).await,
            "llSetScale" => self.ll_set_scale(args, context).await,
            "llGetScale" => self.ll_get_scale(args, context).await,
            "llScaleTexture" => self.ll_scale_texture(args, context).await,
            "llOffsetTexture" => self.ll_offset_texture(args, context).await,
            "llRotateTexture" => self.ll_rotate_texture(args, context).await,
            "llGetTextureOffset" => self.ll_get_texture_offset(args, context).await,
            "llGetTextureScale" => self.ll_get_texture_scale(args, context).await,
            "llGetTextureRot" => self.ll_get_texture_rot(args, context).await,
            "llSetTextureAnim" => self.ll_set_texture_anim(args, context).await,
            "llSetLinkTexture" => self.ll_set_link_texture(args, context).await,
            "llSetLinkColor" => self.ll_set_link_color(args, context).await,
            "llSetLinkAlpha" => self.ll_set_link_alpha(args, context).await,
            "llSetLinkTextureAnim" => self.ll_set_link_texture_anim(args, context).await,

            // Primitive parameters
            "llSetPrimitiveParams" => self.ll_set_primitive_params(args, context).await,
            "llGetPrimitiveParams" => self.ll_get_primitive_params(args, context).await,
            "llSetLinkPrimitiveParams" => self.ll_set_link_primitive_params(args, context).await,
            "llSetLinkPrimitiveParamsFast" => {
                self.ll_set_link_primitive_params_fast(args, context).await
            }
            "llGetLinkPrimitiveParams" => self.ll_get_link_primitive_params(args, context).await,
            "llGetNumberOfSides" => self.ll_get_number_of_sides(args, context).await,
            "llGetLinkNumberOfSides" => self.ll_get_link_number_of_sides(args, context).await,

            // Detected functions (touch, collision, sensor)
            "llDetectedKey" => self.ll_detected_key(args, context).await,
            "llDetectedName" => self.ll_detected_name(args, context).await,
            "llDetectedOwner" => self.ll_detected_owner(args, context).await,
            "llDetectedType" => self.ll_detected_type(args, context).await,
            "llDetectedPos" => self.ll_detected_pos(args, context).await,
            "llDetectedVel" => self.ll_detected_vel(args, context).await,
            "llDetectedRot" => self.ll_detected_rot(args, context).await,
            "llDetectedGroup" => self.ll_detected_group(args, context).await,
            "llDetectedLinkNumber" => self.ll_detected_link_number(args, context).await,
            "llDetectedGrab" => self.ll_detected_grab(args, context).await,
            "llDetectedTouchFace" => self.ll_detected_touch_face(args, context).await,
            "llDetectedTouchPos" => self.ll_detected_touch_pos(args, context).await,
            "llDetectedTouchNormal" => self.ll_detected_touch_normal(args, context).await,
            "llDetectedTouchBinormal" => self.ll_detected_touch_binormal(args, context).await,
            "llDetectedTouchST" => self.ll_detected_touch_st(args, context).await,
            "llDetectedTouchUV" => self.ll_detected_touch_uv(args, context).await,

            // Agent/Avatar functions
            "llGetAgentInfo" => self.ll_get_agent_info(args, context).await,
            "llGetAgentSize" => self.ll_get_agent_size(args, context).await,
            "llGetAgentLanguage" => self.ll_get_agent_language(args, context).await,
            "llGetAgentList" => self.ll_get_agent_list(args, context).await,
            "llRequestAgentData" => self.ll_request_agent_data(args, context).await,
            "llKey2Name" => self.ll_key2name(args, context).await,
            "llName2Key" => self.ll_name2key(args, context).await,
            "llGetDisplayName" => self.ll_get_display_name(args, context).await,
            "llGetUsername" => self.ll_get_username(args, context).await,
            "llRequestDisplayName" => self.ll_request_display_name(args, context).await,
            "llRequestUsername" => self.ll_request_username(args, context).await,
            "llRequestUserKey" => self.ll_request_user_key(args, context).await,
            "llGetHealth" => self.ll_get_health(args, context).await,
            "llGetEnergy" => self.ll_get_energy(args, context).await,
            "llTeleportAgent" => self.ll_teleport_agent(args, context).await,
            "llTeleportAgentHome" => self.ll_teleport_agent_home(args, context).await,
            "llTeleportAgentGlobalCoords" => {
                self.ll_teleport_agent_global_coords(args, context).await
            }
            "llEjectFromLand" => self.ll_eject_from_land(args, context).await,
            "llInstantMessage" => self.ll_instant_message(args, context).await,
            "llGiveMoney" => self.ll_give_money(args, context).await,
            "llTransferLindenDollars" => self.ll_transfer_linden_dollars(args, context).await,

            // Permission functions
            "llRequestPermissions" => self.ll_request_permissions(args, context).await,
            "llGetPermissions" => self.ll_get_permissions(args, context).await,
            "llGetPermissionsKey" => self.ll_get_permissions_key(args, context).await,
            "llTakeControls" => self.ll_take_controls(args, context).await,
            "llReleaseControls" => self.ll_release_controls(args, context).await,
            "llTakeCamera" => self.ll_take_camera(args, context).await,
            "llReleaseCamera" => self.ll_release_camera(args, context).await,
            "llAttachToAvatar" => self.ll_attach_to_avatar(args, context).await,
            "llAttachToAvatarTemp" => self.ll_attach_to_avatar_temp(args, context).await,
            "llDetachFromAvatar" => self.ll_detach_from_avatar(args, context).await,
            "llGetAttached" => self.ll_get_attached(args, context).await,
            "llGetAttachedList" => self.ll_get_attached_list(args, context).await,

            // Camera functions
            "llSetCameraParams" => self.ll_set_camera_params(args, context).await,
            "llClearCameraParams" => self.ll_clear_camera_params(args, context).await,
            "llSetCameraAtOffset" => self.ll_set_camera_at_offset(args, context).await,
            "llSetCameraEyeOffset" => self.ll_set_camera_eye_offset(args, context).await,
            "llGetCameraPos" => self.ll_get_camera_pos(args, context).await,
            "llGetCameraRot" => self.ll_get_camera_rot(args, context).await,
            "llGetCameraAspect" => self.ll_get_camera_aspect(args, context).await,
            "llGetCameraFOV" => self.ll_get_camera_fov(args, context).await,
            "llForceMouselook" => self.ll_force_mouselook(args, context).await,

            // Dialog and UI functions
            "llDialog" => self.ll_dialog(args, context).await,
            "llTextBox" => self.ll_text_box(args, context).await,
            "llMapDestination" => self.ll_map_destination(args, context).await,
            "llLoadURL" => self.ll_load_url(args, context).await,
            "llSetPayPrice" => self.ll_set_pay_price(args, context).await,
            "llSetClickAction" => self.ll_set_click_action(args, context).await,
            "llSetSitText" => self.ll_set_sit_text(args, context).await,
            "llSetTouchText" => self.ll_set_touch_text(args, context).await,

            // Link functions
            "llGetLinkNumber" => self.ll_get_link_number(args, context).await,
            "llGetLinkKey" => self.ll_get_link_key(args, context).await,
            "llGetLinkName" => self.ll_get_link_name(args, context).await,
            "llGetNumberOfPrims" => self.ll_get_number_of_prims(args, context).await,
            "llGetObjectPrimCount" => self.ll_get_object_prim_count(args, context).await,
            "llGetObjectLinkKey" => self.ll_get_object_link_key(args, context).await,
            "llCreateLink" => self.ll_create_link(args, context).await,
            "llBreakLink" => self.ll_break_link(args, context).await,
            "llBreakAllLinks" => self.ll_break_all_links(args, context).await,
            "llMessageLinked" => self.ll_message_linked(args, context).await,
            "llSetLinkCamera" => self.ll_set_link_camera(args, context).await,
            "llLinkSitTarget" => self.ll_link_sit_target(args, context).await,

            // Sit functions
            "llSitTarget" => self.ll_sit_target(args, context).await,
            "llAvatarOnSitTarget" => self.ll_avatar_on_sit_target(args, context).await,
            "llAvatarOnLinkSitTarget" => self.ll_avatar_on_link_sit_target(args, context).await,
            "llUnSit" => self.ll_unsit(args, context).await,
            "llGetLinkSitFlags" => self.ll_get_link_sit_flags(args, context).await,
            "llSetLinkSitFlags" => self.ll_set_link_sit_flags(args, context).await,

            // Land and parcel functions
            "llGetParcelFlags" => self.ll_get_parcel_flags(args, context).await,
            "llGetParcelDetails" => self.ll_get_parcel_details(args, context).await,
            "llGetParcelMaxPrims" => self.ll_get_parcel_max_prims(args, context).await,
            "llGetParcelPrimCount" => self.ll_get_parcel_prim_count(args, context).await,
            "llGetParcelPrimOwners" => self.ll_get_parcel_prim_owners(args, context).await,
            "llGetParcelMusicURL" => self.ll_get_parcel_music_url(args, context).await,
            "llSetParcelMusicURL" => self.ll_set_parcel_music_url(args, context).await,
            "llGetLandOwnerAt" => self.ll_get_land_owner_at(args, context).await,
            "llOverMyLand" => self.ll_over_my_land(args, context).await,
            "llModifyLand" => self.ll_modify_land(args, context).await,
            "llAddToLandBanList" => self.ll_add_to_land_ban_list(args, context).await,
            "llRemoveFromLandBanList" => self.ll_remove_from_land_ban_list(args, context).await,
            "llAddToLandPassList" => self.ll_add_to_land_pass_list(args, context).await,
            "llRemoveFromLandPassList" => self.ll_remove_from_land_pass_list(args, context).await,
            "llResetLandBanList" => self.ll_reset_land_ban_list(args, context).await,
            "llResetLandPassList" => self.ll_reset_land_pass_list(args, context).await,

            // Vehicle functions
            "llSetVehicleType" => self.ll_set_vehicle_type(args, context).await,
            "llSetVehicleFlags" => self.ll_set_vehicle_flags(args, context).await,
            "llRemoveVehicleFlags" => self.ll_remove_vehicle_flags(args, context).await,
            "llSetVehicleFloatParam" => self.ll_set_vehicle_float_param(args, context).await,
            "llSetVehicleVectorParam" => self.ll_set_vehicle_vector_param(args, context).await,
            "llSetVehicleRotationParam" => self.ll_set_vehicle_rotation_param(args, context).await,

            // Region functions
            "llGetRegionFlags" => self.ll_get_region_flags(args, context).await,
            "llGetRegionFPS" => self.ll_get_region_fps(args, context).await,
            "llGetRegionTimeDilation" => self.ll_get_region_time_dilation(args, context).await,
            "llGetRegionAgentCount" => self.ll_get_region_agent_count(args, context).await,
            "llRequestSimulatorData" => self.ll_request_simulator_data(args, context).await,
            "llGetSimulatorHostname" => self.ll_get_simulator_hostname(args, context).await,
            "llGetEnv" => self.ll_get_env(args, context).await,
            "llGetSimStats" => self.ll_get_sim_stats(args, context).await,

            // Ground/terrain functions
            "llGround" => self.ll_ground(args, context).await,
            "llGroundNormal" => self.ll_ground_normal(args, context).await,
            "llGroundSlope" => self.ll_ground_slope(args, context).await,
            "llGroundContour" => self.ll_ground_contour(args, context).await,
            "llWater" => self.ll_water(args, context).await,
            "llCloud" => self.ll_cloud(args, context).await,
            "llWind" => self.ll_wind(args, context).await,
            "llEdgeOfWorld" => self.ll_edge_of_world(args, context).await,

            // Sun/Moon functions
            "llGetSunDirection" => self.ll_get_sun_direction(args, context).await,
            "llGetSunRotation" => self.ll_get_sun_rotation(args, context).await,
            "llGetMoonDirection" => self.ll_get_moon_direction(args, context).await,
            "llGetMoonRotation" => self.ll_get_moon_rotation(args, context).await,
            "llGetRegionSunDirection" => self.ll_get_region_sun_direction(args, context).await,
            "llGetRegionSunRotation" => self.ll_get_region_sun_rotation(args, context).await,
            "llGetRegionMoonDirection" => self.ll_get_region_moon_direction(args, context).await,
            "llGetRegionMoonRotation" => self.ll_get_region_moon_rotation(args, context).await,
            "llGetDayLength" => self.ll_get_day_length(args, context).await,
            "llGetDayOffset" => self.ll_get_day_offset(args, context).await,
            "llGetRegionDayLength" => self.ll_get_region_day_length(args, context).await,
            "llGetRegionDayOffset" => self.ll_get_region_day_offset(args, context).await,

            // Object functions
            "llRezObject" => self.ll_rez_object(args, context).await,
            "llRezAtRoot" => self.ll_rez_at_root(args, context).await,
            "llRezObjectWithParams" => self.ll_rez_object_with_params(args, context).await,
            "llDie" => self.ll_die(args, context).await,
            "llDerezObject" => self.ll_derez_object(args, context).await,
            "llGetCreator" => self.ll_get_creator(args, context).await,
            "llGetOwnerKey" => self.ll_get_owner_key(args, context).await,
            "llGetBoundingBox" => self.ll_get_bounding_box(args, context).await,
            "llGetGeometricCenter" => self.ll_get_geometric_center(args, context).await,
            "llGetCenterOfMass" => self.ll_get_center_of_mass(args, context).await,
            "llGetObjectDetails" => self.ll_get_object_details(args, context).await,
            "llGetStatus" => self.ll_get_status(args, context).await,
            "llSetStatus" => self.ll_set_status(args, context).await,
            "llSetDamage" => self.ll_set_damage(args, context).await,
            "llAllowInventoryDrop" => self.ll_allow_inventory_drop(args, context).await,
            "llPassTouches" => self.ll_pass_touches(args, context).await,
            "llPassCollisions" => self.ll_pass_collisions(args, context).await,
            "llCollisionFilter" => self.ll_collision_filter(args, context).await,
            "llVolumeDetect" => self.ll_volume_detect(args, context).await,

            // Target functions
            "llTarget" => self.ll_target(args, context).await,
            "llTargetRemove" => self.ll_target_remove(args, context).await,
            "llRotTarget" => self.ll_rot_target(args, context).await,
            "llRotTargetRemove" => self.ll_rot_target_remove(args, context).await,

            // Look at functions
            "llLookAt" => self.ll_look_at(args, context).await,
            "llStopLookAt" => self.ll_stop_look_at(args, context).await,
            "llRotLookAt" => self.ll_rot_look_at(args, context).await,
            "llPointAt" => self.ll_point_at(args, context).await,
            "llStopPointAt" => self.ll_stop_point_at(args, context).await,

            // Particle functions
            "llParticleSystem" => self.ll_particle_system(args, context).await,
            "llLinkParticleSystem" => self.ll_link_particle_system(args, context).await,
            "llMakeExplosion" => self.ll_make_explosion(args, context).await,
            "llMakeFire" => self.ll_make_fire(args, context).await,
            "llMakeFountain" => self.ll_make_fountain(args, context).await,
            "llMakeSmoke" => self.ll_make_smoke(args, context).await,

            // Email functions
            "llEmail" => self.ll_email(args, context).await,
            "llTargetedEmail" => self.ll_targeted_email(args, context).await,
            "llGetNextEmail" => self.ll_get_next_email(args, context).await,

            // HTTP/URL additional functions
            "llRequestURL" => self.ll_request_url(args, context).await,
            "llRequestSecureURL" => self.ll_request_secure_url(args, context).await,
            "llReleaseURL" => self.ll_release_url(args, context).await,
            "llGetFreeURLs" => self.ll_get_free_urls(args, context).await,
            "llGetHTTPHeader" => self.ll_get_http_header(args, context).await,
            "llSetContentType" => self.ll_set_content_type(args, context).await,

            // XML-RPC functions
            "llOpenRemoteDataChannel" => self.ll_open_remote_data_channel(args, context).await,
            "llCloseRemoteDataChannel" => self.ll_close_remote_data_channel(args, context).await,
            "llSendRemoteData" => self.ll_send_remote_data(args, context).await,
            "llRemoteDataReply" => self.ll_remote_data_reply(args, context).await,
            "llRemoteDataSetRegion" => self.ll_remote_data_set_region(args, context).await,

            // JSON functions
            "llJson2List" => self.ll_json2list(args, context).await,
            "llList2Json" => self.ll_list2json(args, context).await,
            "llJsonGetValue" => self.ll_json_get_value(args, context).await,
            "llJsonSetValue" => self.ll_json_set_value(args, context).await,
            "llJsonValueType" => self.ll_json_value_type(args, context).await,

            // Linkset Data functions
            "llLinksetDataWrite" => self.ll_linkset_data_write(args, context).await,
            "llLinksetDataWriteProtected" => {
                self.ll_linkset_data_write_protected(args, context).await
            }
            "llLinksetDataRead" => self.ll_linkset_data_read(args, context).await,
            "llLinksetDataReadProtected" => {
                self.ll_linkset_data_read_protected(args, context).await
            }
            "llLinksetDataDelete" => self.ll_linkset_data_delete(args, context).await,
            "llLinksetDataDeleteProtected" => {
                self.ll_linkset_data_delete_protected(args, context).await
            }
            "llLinksetDataDeleteFound" => self.ll_linkset_data_delete_found(args, context).await,
            "llLinksetDataCountKeys" => self.ll_linkset_data_count_keys(args, context).await,
            "llLinksetDataCountFound" => self.ll_linkset_data_count_found(args, context).await,
            "llLinksetDataFindKeys" => self.ll_linkset_data_find_keys(args, context).await,
            "llLinksetDataListKeys" => self.ll_linkset_data_list_keys(args, context).await,
            "llLinksetDataAvailable" => self.ll_linkset_data_available(args, context).await,
            "llLinksetDataReset" => self.ll_linkset_data_reset(args, context).await,

            // Hash/Crypto functions
            "llMD5String" => self.ll_md5_string(args, context).await,
            "llSHA1String" => self.ll_sha1_string(args, context).await,
            "llSHA256String" => self.ll_sha256_string(args, context).await,
            "llHMAC" => self.ll_hmac(args, context).await,
            "llComputeHash" => self.ll_compute_hash(args, context).await,
            "llHash" => self.ll_hash(args, context).await,

            // Base64 functions
            "llStringToBase64" => self.ll_string_to_base64(args, context).await,
            "llBase64ToString" => self.ll_base64_to_string(args, context).await,
            "llIntegerToBase64" => self.ll_integer_to_base64(args, context).await,
            "llBase64ToInteger" => self.ll_base64_to_integer(args, context).await,
            "llXorBase64" => self.ll_xor_base64(args, context).await,
            "llXorBase64Strings" => self.ll_xor_base64_strings(args, context).await,
            "llXorBase64StringsCorrect" => self.ll_xor_base64_strings_correct(args, context).await,

            // Character/String conversion
            "llChar" => self.ll_char(args, context).await,
            "llOrd" => self.ll_ord(args, context).await,
            "llEscapeURL" => self.ll_escape_url(args, context).await,
            "llUnescapeURL" => self.ll_unescape_url(args, context).await,
            "llStringTrim" => self.ll_string_trim(args, context).await,
            "llReplaceSubString" => self.ll_replace_sub_string(args, context).await,

            // Additional list functions
            "llListStatistics" => self.ll_list_statistics(args, context).await,
            "llListSort" => self.ll_list_sort(args, context).await,
            "llListSortStrided" => self.ll_list_sort_strided(args, context).await,
            "llListRandomize" => self.ll_list_randomize(args, context).await,
            "llListReplaceList" => self.ll_list_replace_list(args, context).await,
            "llList2List" => self.ll_list2list(args, context).await,
            "llList2ListSlice" => self.ll_list2list_slice(args, context).await,
            "llList2ListStrided" => self.ll_list2list_strided(args, context).await,
            "llListFindList" => self.ll_list_find_list(args, context).await,
            "llListFindListNext" => self.ll_list_find_list_next(args, context).await,
            "llListFindStrided" => self.ll_list_find_strided(args, context).await,
            "llGetListEntryType" => self.ll_get_list_entry_type(args, context).await,
            "llParseStringKeepNulls" => self.ll_parse_string_keep_nulls(args, context).await,

            // Rotation additional functions
            "llAxisAngle2Rot" => self.ll_axis_angle2rot(args, context).await,
            "llRotBetween" => self.ll_rot_between(args, context).await,
            "llRot2Fwd" => self.ll_rot2fwd(args, context).await,
            "llRot2Left" => self.ll_rot2left(args, context).await,
            "llRot2Up" => self.ll_rot2up(args, context).await,
            "llGetLocalRot" => self.ll_get_local_rot(args, context).await,
            "llSetLocalRot" => self.ll_set_local_rot(args, context).await,
            "llGetRootRotation" => self.ll_get_root_rotation(args, context).await,
            "llGetLocalPos" => self.ll_get_local_pos(args, context).await,
            "llGetRootPosition" => self.ll_get_root_position(args, context).await,

            // Inventory additional functions
            "llGiveInventory" => self.ll_give_inventory(args, context).await,
            "llGiveInventoryList" => self.ll_give_inventory_list(args, context).await,
            "llRemoveInventory" => self.ll_remove_inventory(args, context).await,
            "llGetInventoryCreator" => self.ll_get_inventory_creator(args, context).await,
            "llGetInventoryPermMask" => self.ll_get_inventory_perm_mask(args, context).await,
            "llSetInventoryPermMask" => self.ll_set_inventory_perm_mask(args, context).await,
            "llGetInventoryDesc" => self.ll_get_inventory_desc(args, context).await,
            "llGetInventoryAcquireTime" => self.ll_get_inventory_acquire_time(args, context).await,
            "llRequestInventoryData" => self.ll_request_inventory_data(args, context).await,
            "llGetObjectPermMask" => self.ll_get_object_perm_mask(args, context).await,
            "llSetObjectPermMask" => self.ll_set_object_perm_mask(args, context).await,

            // Notecard functions
            "llGetNotecardLine" => self.ll_get_notecard_line(args, context).await,
            "llGetNotecardLineSync" => self.ll_get_notecard_line_sync(args, context).await,
            "llGetNumberOfNotecardLines" => {
                self.ll_get_number_of_notecard_lines(args, context).await
            }

            // Media functions
            "llSetPrimMediaParams" => self.ll_set_prim_media_params(args, context).await,
            "llGetPrimMediaParams" => self.ll_get_prim_media_params(args, context).await,
            "llClearPrimMedia" => self.ll_clear_prim_media(args, context).await,
            "llSetLinkMedia" => self.ll_set_link_media(args, context).await,
            "llGetLinkMedia" => self.ll_get_link_media(args, context).await,
            "llClearLinkMedia" => self.ll_clear_link_media(args, context).await,
            "llParcelMediaCommandList" => self.ll_parcel_media_command_list(args, context).await,
            "llParcelMediaQuery" => self.ll_parcel_media_query(args, context).await,
            "llSetPrimURL" => self.ll_set_prim_url(args, context).await,
            "llRefreshPrimURL" => self.ll_refresh_prim_url(args, context).await,

            // Cast ray
            "llCastRay" => self.ll_cast_ray(args, context).await,
            "llCastRayV3" => self.ll_cast_ray_v3(args, context).await,

            // Color space conversion
            "llLinear2sRGB" => self.ll_linear2srgb(args, context).await,
            "llsRGB2Linear" => self.ll_srgb2linear(args, context).await,

            // Time additional functions
            "llGetTime" => self.ll_get_time(args, context).await,
            "llGetAndResetTime" => self.ll_get_and_reset_time(args, context).await,
            "llResetTime" => self.ll_reset_time(args, context).await,
            "llGetDate" => self.ll_get_date(args, context).await,
            "llGetGMTclock" => self.ll_get_gmt_clock(args, context).await,
            "llGetWallclock" => self.ll_get_wallclock(args, context).await,
            "llGetTimeOfDay" => self.ll_get_time_of_day(args, context).await,

            // Script control functions
            "llGetScriptState" => self.ll_get_script_state(args, context).await,
            "llSetScriptState" => self.ll_set_script_state(args, context).await,
            "llResetOtherScript" => self.ll_reset_other_script(args, context).await,
            "llRemoteLoadScript" => self.ll_remote_load_script(args, context).await,
            "llRemoteLoadScriptPin" => self.ll_remote_load_script_pin(args, context).await,
            "llSetRemoteScriptAccessPin" => {
                self.ll_set_remote_script_access_pin(args, context).await
            }
            "llGetStartParameter" => self.ll_get_start_parameter(args, context).await,
            "llMinEventDelay" => self.ll_min_event_delay(args, context).await,
            "llScriptDanger" => self.ll_script_danger(args, context).await,
            "llScriptProfiler" => self.ll_script_profiler(args, context).await,
            "llGetMemoryLimit" => self.ll_get_memory_limit(args, context).await,
            "llSetMemoryLimit" => self.ll_set_memory_limit(args, context).await,
            "llGetSPMaxMemory" => self.ll_get_sp_max_memory(args, context).await,

            // Key generation and testing
            "llGenerateKey" => self.ll_generate_key(args, context).await,
            "llSameGroup" => self.ll_same_group(args, context).await,
            "llIsFriend" => self.ll_is_friend(args, context).await,

            // Scale functions
            "llScaleByFactor" => self.ll_scale_by_factor(args, context).await,
            "llGetMaxScaleFactor" => self.ll_get_max_scale_factor(args, context).await,
            "llGetMinScaleFactor" => self.ll_get_min_scale_factor(args, context).await,

            // Estate management
            "llManageEstateAccess" => self.ll_manage_estate_access(args, context).await,

            // Region position
            "llSetRegionPos" => self.ll_set_region_pos(args, context).await,

            // Keyframed motion
            "llSetKeyframedMotion" => self.ll_set_keyframed_motion(args, context).await,

            // Collision sprite (deprecated)
            "llCollisionSprite" => self.ll_collision_sprite(args, context).await,

            // God mode (internal)
            "llGodLikeRezObject" => self.ll_god_like_rez_object(args, context).await,

            // Visual params
            "llGetVisualParams" => self.ll_get_visual_params(args, context).await,

            // Math additional
            "llModPow" => self.ll_mod_pow(args, context).await,

            _ => Err(anyhow!("Unknown LSL function: {}", function_name)),
        }
    }

    // Chat and communication functions
    async fn ll_say(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSay expects 2 arguments"));
        }

        let channel = args[0].to_integer();
        let message = args[1].to_string();

        info!(
            "Object {} says on channel {}: {}",
            context.object_id, channel, message
        );

        // Send chat message via grid event manager if available
        if let Some(grid_events) = &self.grid_event_manager {
            grid_events
                .publish_chat_message(
                    context.owner_id,
                    &message,
                    channel,
                    context.region_id,
                    Some(context.position),
                )
                .await?;
        }

        Ok(LSLValue::Integer(0))
    }

    async fn ll_shout(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llShout expects 2 arguments"));
        }
        let channel = args[0].to_integer();
        let message = args[1].to_string();
        let msg = if message.len() > 1024 {
            &message[..1024]
        } else {
            &message
        };
        debug!(
            "Object {} shouts on channel {}: {}",
            context.object_id, channel, msg
        );
        if let Some(grid_events) = &self.grid_event_manager {
            grid_events
                .publish_chat_message(
                    context.owner_id,
                    msg,
                    channel,
                    context.region_id,
                    Some(context.position),
                )
                .await?;
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_whisper(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llWhisper expects 2 arguments"));
        }
        let channel = args[0].to_integer();
        let message = args[1].to_string();
        let msg = if message.len() > 1024 {
            &message[..1024]
        } else {
            &message
        };
        debug!(
            "Object {} whispers on channel {}: {}",
            context.object_id, channel, msg
        );
        if let Some(grid_events) = &self.grid_event_manager {
            grid_events
                .publish_chat_message(
                    context.owner_id,
                    msg,
                    channel,
                    context.region_id,
                    Some(context.position),
                )
                .await?;
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_owner_say(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llOwnerSay expects 1 argument"));
        }
        let message = args[0].to_string();
        let msg = if message.len() > 1024 {
            &message[..1024]
        } else {
            &message
        };
        debug!(
            "Object {} says to owner {}: {}",
            context.object_id, context.owner_id, msg
        );
        if let Some(grid_events) = &self.grid_event_manager {
            grid_events
                .publish_chat_message(
                    context.owner_id,
                    msg,
                    -1,
                    context.region_id,
                    Some(context.position),
                )
                .await?;
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_region_say(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRegionSay expects 2 arguments"));
        }
        let channel = args[0].to_integer();
        if channel == 0 {
            return Ok(LSLValue::Integer(0));
        }
        let message = args[1].to_string();
        let msg = if message.len() > 1024 {
            &message[..1024]
        } else {
            &message
        };
        debug!(
            "Object {} region says on channel {}: {}",
            context.object_id, channel, msg
        );
        if let Some(grid_events) = &self.grid_event_manager {
            grid_events
                .publish_chat_message(
                    context.owner_id,
                    msg,
                    channel,
                    context.region_id,
                    Some(context.position),
                )
                .await?;
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_listen(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llListen expects 4 arguments"));
        }

        let channel = args[0].to_integer();
        let name = args[1].to_string();
        let id = args[2].to_key();
        let message = args[3].to_string();

        let handle = rand::random::<i32>().abs();

        let listener = super::LSLListener {
            handle,
            channel,
            name,
            id: if id == Uuid::nil() { None } else { Some(id) },
            message: if message.is_empty() {
                None
            } else {
                Some(message)
            },
            active: true,
        };

        context.listeners.insert(handle.to_string(), listener);

        debug!("Created listener {} for channel {}", handle, channel);
        Ok(LSLValue::Integer(handle))
    }

    async fn ll_listen_control(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llListenControl expects 2 arguments"));
        }

        let handle = args[0].to_integer().to_string();
        let active = args[1].is_true();

        if let Some(listener) = context.listeners.get_mut(&handle) {
            listener.active = active;
            debug!("Set listener {} active: {}", handle, active);
        }

        Ok(LSLValue::Integer(0))
    }

    async fn ll_listen_remove(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llListenRemove expects 1 argument"));
        }

        let handle = args[0].to_integer().to_string();
        context.listeners.remove(&handle);

        debug!("Removed listener {}", handle);
        Ok(LSLValue::Integer(0))
    }

    // Position and movement functions
    async fn ll_get_pos(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(
            context.position.0,
            context.position.1,
            context.position.2,
        )))
    }

    async fn ll_set_pos(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetPos expects 1 argument"));
        }

        let pos = args[0].to_vector();
        context.position = (pos.x, pos.y, pos.z);

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPos {
                object_id: context.object_id,
                position: [pos.x, pos.y, pos.z],
            },
        ));

        debug!("Set position to {}", pos);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_rot(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(
            context.rotation.0,
            context.rotation.1,
            context.rotation.2,
            context.rotation.3,
        )))
    }

    async fn ll_set_rot(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetRot expects 1 argument"));
        }

        let rot = args[0].to_rotation();
        context.rotation = (rot.x, rot.y, rot.z, rot.s);

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetRot {
                object_id: context.object_id,
                rotation: [rot.x, rot.y, rot.z, rot.s],
            },
        ));

        debug!("Set rotation to {}", rot);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_vel(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(
            context.velocity.0,
            context.velocity.1,
            context.velocity.2,
        )))
    }

    async fn ll_set_velocity(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetVelocity expects 2 arguments"));
        }
        let vel = args[0].to_vector();
        let local = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVelocity {
                object_id: context.object_id,
                velocity: [vel.x as f32, vel.y as f32, vel.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    // Math functions
    async fn ll_abs(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAbs expects 1 argument"));
        }
        Ok(LSLValue::Integer(args[0].to_integer().abs()))
    }

    async fn ll_fabs(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llFabs expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().abs()))
    }

    async fn ll_sqrt(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSqrt expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().sqrt()))
    }

    async fn ll_pow(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llPow expects 2 arguments"));
        }
        Ok(LSLValue::Float(args[0].to_float().powf(args[1].to_float())))
    }

    async fn ll_sin(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSin expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().sin()))
    }

    async fn ll_cos(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCos expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().cos()))
    }

    async fn ll_tan(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llTan expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().tan()))
    }

    async fn ll_asin(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAsin expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().asin()))
    }

    async fn ll_acos(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAcos expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().acos()))
    }

    async fn ll_atan2(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llAtan2 expects 2 arguments"));
        }
        Ok(LSLValue::Float(
            args[0].to_float().atan2(args[1].to_float()),
        ))
    }

    async fn ll_floor(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llFloor expects 1 argument"));
        }
        Ok(LSLValue::Integer(args[0].to_float().floor() as i32))
    }

    async fn ll_ceil(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCeil expects 1 argument"));
        }
        Ok(LSLValue::Integer(args[0].to_float().ceil() as i32))
    }

    async fn ll_round(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRound expects 1 argument"));
        }
        Ok(LSLValue::Integer(args[0].to_float().round() as i32))
    }

    async fn ll_frand(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llFrand expects 1 argument"));
        }
        let max = args[0].to_float();
        let r: f32 = rand::random();
        if max < 0.0 {
            Ok(LSLValue::Float(r * max))
        } else {
            Ok(LSLValue::Float(r * max))
        }
    }

    async fn ll_log(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLog expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().ln()))
    }

    async fn ll_log10(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLog10 expects 1 argument"));
        }
        Ok(LSLValue::Float(args[0].to_float().log10()))
    }

    // Vector functions
    async fn ll_vec_dist(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llVecDist expects 2 arguments"));
        }
        let v1 = args[0].to_vector();
        let v2 = args[1].to_vector();
        Ok(LSLValue::Float(v1.distance(&v2)))
    }

    async fn ll_vec_mag(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llVecMag expects 1 argument"));
        }
        let v = args[0].to_vector();
        Ok(LSLValue::Float(v.magnitude()))
    }

    async fn ll_vec_norm(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llVecNorm expects 1 argument"));
        }
        let v = args[0].to_vector();
        Ok(LSLValue::Vector(v.normalize()))
    }

    // Rotation functions
    async fn ll_euler2rot(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llEuler2Rot expects 1 argument"));
        }
        let v = args[0].to_vector();
        let rot = LSLRotation::from_euler(v.x, v.y, v.z);
        Ok(LSLValue::Rotation(rot))
    }

    async fn ll_rot2euler(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Euler expects 1 argument"));
        }
        let rot = args[0].to_rotation();
        let (roll, pitch, yaw) = rot.to_euler();
        Ok(LSLValue::Vector(LSLVector::new(roll, pitch, yaw)))
    }

    // String functions
    async fn ll_string_length(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStringLength expects 1 argument"));
        }
        Ok(LSLValue::Integer(args[0].to_string().len() as i32))
    }

    async fn ll_get_sub_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llGetSubString expects 3 arguments"));
        }
        let string = args[0].to_string();
        let start = args[1].to_integer();
        let end = args[2].to_integer();

        let chars: Vec<char> = string.chars().collect();
        let len = chars.len() as i32;
        if len == 0 {
            return Ok(LSLValue::String(String::new()));
        }

        let si = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len - 1)
        } as usize;
        let ei = if end < 0 {
            (len + end).max(0)
        } else {
            end.min(len - 1)
        } as usize;

        let substring: String = if si <= ei {
            chars[si..=ei].iter().collect()
        } else {
            let mut s: String = chars[..=ei].iter().collect();
            s.push_str(&chars[si..].iter().collect::<String>());
            s
        };

        Ok(LSLValue::String(substring))
    }

    async fn ll_to_lower(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llToLower expects 1 argument"));
        }
        Ok(LSLValue::String(args[0].to_string().to_lowercase()))
    }

    async fn ll_to_upper(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llToUpper expects 1 argument"));
        }
        Ok(LSLValue::String(args[0].to_string().to_uppercase()))
    }

    // List functions
    async fn ll_list_length(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llListLength expects 1 argument"));
        }
        let list = args[0].to_list();
        Ok(LSLValue::Integer(list.len() as i32))
    }

    async fn ll_list2string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2String expects 2 arguments"));
        }
        let list = args[0].to_list();
        let index = args[1].to_integer() as usize;

        if index < list.len() {
            Ok(LSLValue::String(list[index].to_string()))
        } else {
            Ok(LSLValue::String(String::new()))
        }
    }

    async fn ll_list2integer(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Integer expects 2 arguments"));
        }
        let list = args[0].to_list();
        let index = args[1].to_integer() as usize;

        if index < list.len() {
            Ok(LSLValue::Integer(list[index].to_integer()))
        } else {
            Ok(LSLValue::Integer(0))
        }
    }

    async fn ll_list2float(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Float expects 2 arguments"));
        }
        let list = args[0].to_list();
        let index = args[1].to_integer() as usize;

        if index < list.len() {
            Ok(LSLValue::Float(list[index].to_float()))
        } else {
            Ok(LSLValue::Float(0.0))
        }
    }

    // Time functions
    async fn ll_get_timestamp(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S.%fZ")
            .to_string();
        Ok(LSLValue::String(timestamp))
    }

    async fn ll_get_unix_time(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        let unix_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
        Ok(LSLValue::Integer(unix_time))
    }

    async fn ll_set_timer_event(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetTimerEvent expects 1 argument"));
        }

        let interval = args[0].to_float();
        if interval > 0.0 {
            let expiry = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + interval as u64;

            context.timers.insert("default".to_string(), expiry);
            debug!("Set timer for {} seconds", interval);
        } else {
            context.timers.remove("default");
            debug!("Cleared timer");
        }

        Ok(LSLValue::Integer(0))
    }

    // Utility functions
    async fn ll_sleep(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSleep expects 1 argument"));
        }

        let duration = args[0].to_float();
        if duration > 0.0 {
            tokio::time::sleep(tokio::time::Duration::from_secs_f32(duration)).await;
        }

        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_text(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetText expects 3 arguments"));
        }

        let text = args[0].to_string();
        let color = args[1].to_vector();
        let alpha = args[2].to_float();

        context.floating_text = Some(super::FloatingText {
            text: text.clone(),
            color: (color.x, color.y, color.z),
            alpha,
        });

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetText {
                object_id: context.object_id,
                text,
                color: [color.x, color.y, color.z],
                alpha,
            },
        ));

        debug!("Set floating text for object {}", context.object_id);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_object_name(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetObjectName expects 1 argument"));
        }
        let name = args[0].to_string();
        context.object_name = name.clone();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetObjectName {
                object_id: context.object_id,
                name,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_object_desc(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetObjectDesc expects 1 argument"));
        }
        let desc = args[0].to_string();
        context.object_description = desc.clone();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetObjectDesc {
                object_id: context.object_id,
                desc,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_object_name(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::String(context.object_name.clone()))
    }

    async fn ll_get_object_desc(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::String(context.object_description.clone()))
    }

    async fn ll_get_key(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Key(context.object_id))
    }

    async fn ll_get_owner(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Key(context.owner_id))
    }

    async fn ll_get_region_corner(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        let region_x = (context.region_handle >> 32) as f32;
        let region_y = (context.region_handle & 0xFFFFFFFF) as f32;
        Ok(LSLValue::Vector(LSLVector::new(region_x, region_y, 0.0)))
    }

    async fn ll_get_region_name(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::String(context.region_name.clone()))
    }

    async fn ll_angle_between(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llAngleBetween expects 2 arguments"));
        }

        let rot1 = args[0].to_rotation();
        let rot2 = args[1].to_rotation();

        let dot = rot1.x * rot2.x + rot1.y * rot2.y + rot1.z * rot2.z + rot1.s * rot2.s;
        let angle = 2.0 * dot.abs().clamp(-1.0, 1.0).acos();

        Ok(LSLValue::Float(angle))
    }

    async fn ll_axes2rot(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llAxes2Rot expects 3 arguments"));
        }

        let fwd = args[0].to_vector();
        let left = args[1].to_vector();
        let up = args[2].to_vector();

        let trace = fwd.x + left.y + up.z;
        let (x, y, z, s) = if trace > 0.0 {
            let s_root = (trace + 1.0).sqrt() * 2.0;
            (
                (left.z - up.y) / s_root,
                (up.x - fwd.z) / s_root,
                (fwd.y - left.x) / s_root,
                0.25 * s_root,
            )
        } else if fwd.x > left.y && fwd.x > up.z {
            let s_root = (1.0 + fwd.x - left.y - up.z).sqrt() * 2.0;
            (
                0.25 * s_root,
                (fwd.y + left.x) / s_root,
                (up.x + fwd.z) / s_root,
                (left.z - up.y) / s_root,
            )
        } else if left.y > up.z {
            let s_root = (1.0 + left.y - fwd.x - up.z).sqrt() * 2.0;
            (
                (fwd.y + left.x) / s_root,
                0.25 * s_root,
                (left.z + up.y) / s_root,
                (up.x - fwd.z) / s_root,
            )
        } else {
            let s_root = (1.0 + up.z - fwd.x - left.y).sqrt() * 2.0;
            (
                (up.x + fwd.z) / s_root,
                (left.z + up.y) / s_root,
                0.25 * s_root,
                (fwd.y - left.x) / s_root,
            )
        };

        Ok(LSLValue::Rotation(LSLRotation::new(x, y, z, s)))
    }

    async fn ll_rot2axis(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Axis expects 1 argument"));
        }

        let rot = args[0].to_rotation();
        let mag = (rot.x * rot.x + rot.y * rot.y + rot.z * rot.z).sqrt();

        if mag < 0.0001 {
            return Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 1.0)));
        }

        Ok(LSLValue::Vector(LSLVector::new(
            rot.x / mag,
            rot.y / mag,
            rot.z / mag,
        )))
    }

    async fn ll_rot2angle(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Angle expects 1 argument"));
        }

        let rot = args[0].to_rotation();
        let mag = (rot.x * rot.x + rot.y * rot.y + rot.z * rot.z).sqrt();
        let angle = 2.0 * mag.atan2(rot.s);

        Ok(LSLValue::Float(angle))
    }

    async fn ll_sub_string_index(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSubStringIndex expects 2 arguments"));
        }

        let source = args[0].to_string();
        let pattern = args[1].to_string();

        let index = source.find(&pattern).map(|i| i as i32).unwrap_or(-1);
        Ok(LSLValue::Integer(index))
    }

    async fn ll_insert_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llInsertString expects 3 arguments"));
        }

        let mut destination = args[0].to_string();
        let position = args[1].to_integer() as usize;
        let source = args[2].to_string();

        if position <= destination.len() {
            destination.insert_str(position, &source);
        } else {
            destination.push_str(&source);
        }

        Ok(LSLValue::String(destination))
    }

    async fn ll_delete_sub_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llDeleteSubString expects 3 arguments"));
        }

        let source = args[0].to_string();
        let start = args[1].to_integer();
        let end = args[2].to_integer();

        let chars: Vec<char> = source.chars().collect();
        let len = chars.len() as i32;

        let start_idx = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len)
        } as usize;
        let end_idx = if end < 0 {
            (len + end).max(0)
        } else {
            end.min(len - 1)
        } as usize;

        if start_idx > end_idx || start_idx >= chars.len() {
            return Ok(LSLValue::String(source));
        }

        let mut result = String::new();
        for (i, ch) in chars.iter().enumerate() {
            if i < start_idx || i > end_idx {
                result.push(*ch);
            }
        }

        Ok(LSLValue::String(result))
    }

    async fn ll_list2key(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Key expects 2 arguments"));
        }

        let list = args[0].to_list();
        let index = args[1].to_integer();
        let idx = if index < 0 {
            (list.len() as i32 + index).max(0) as usize
        } else {
            index as usize
        };

        if idx < list.len() {
            Ok(LSLValue::Key(list[idx].to_key()))
        } else {
            Ok(LSLValue::Key(Uuid::nil()))
        }
    }

    async fn ll_list2vector(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Vector expects 2 arguments"));
        }

        let list = args[0].to_list();
        let index = args[1].to_integer();
        let idx = if index < 0 {
            (list.len() as i32 + index).max(0) as usize
        } else {
            index as usize
        };

        if idx < list.len() {
            Ok(LSLValue::Vector(list[idx].to_vector()))
        } else {
            Ok(LSLValue::Vector(LSLVector::zero()))
        }
    }

    async fn ll_list2rot(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Rot expects 2 arguments"));
        }

        let list = args[0].to_list();
        let index = args[1].to_integer();
        let idx = if index < 0 {
            (list.len() as i32 + index).max(0) as usize
        } else {
            index as usize
        };

        if idx < list.len() {
            Ok(LSLValue::Rotation(list[idx].to_rotation()))
        } else {
            Ok(LSLValue::Rotation(LSLRotation::identity()))
        }
    }

    async fn ll_list_insert_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llListInsertList expects 3 arguments"));
        }

        let mut dest = args[0].to_list();
        let src = args[1].to_list();
        let position = args[2].to_integer();

        let idx = if position < 0 {
            (dest.len() as i32 + position + 1).max(0) as usize
        } else {
            (position as usize).min(dest.len())
        };

        for (i, item) in src.into_iter().enumerate() {
            dest.insert(idx + i, item);
        }

        Ok(LSLValue::List(dest))
    }

    async fn ll_delete_sub_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llDeleteSubList expects 3 arguments"));
        }

        let list = args[0].to_list();
        let start = args[1].to_integer();
        let end = args[2].to_integer();

        let len = list.len() as i32;
        let start_idx = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len)
        } as usize;
        let end_idx = if end < 0 {
            (len + end).max(0)
        } else {
            end.min(len - 1)
        } as usize;

        if start_idx > end_idx {
            let mut result = Vec::new();
            for (i, item) in list.into_iter().enumerate() {
                if i > end_idx && i < start_idx {
                    result.push(item);
                }
            }
            return Ok(LSLValue::List(result));
        }

        let mut result = Vec::new();
        for (i, item) in list.into_iter().enumerate() {
            if i < start_idx || i > end_idx {
                result.push(item);
            }
        }

        Ok(LSLValue::List(result))
    }

    async fn ll_get_list_length(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetListLength expects 1 argument"));
        }
        let list = args[0].to_list();
        Ok(LSLValue::Integer(list.len() as i32))
    }

    async fn ll_list2csv(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llList2CSV expects 1 argument"));
        }

        let list = args[0].to_list();
        let csv: Vec<String> = list.iter().map(|v| v.to_string()).collect();
        Ok(LSLValue::String(csv.join(", ")))
    }

    async fn ll_csv2list(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCSV2List expects 1 argument"));
        }

        let csv = args[0].to_string();
        let items: Vec<LSLValue> = csv
            .split(',')
            .map(|s| LSLValue::String(s.trim().to_string()))
            .collect();

        Ok(LSLValue::List(items))
    }

    async fn ll_dump_list2string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llDumpList2String expects 2 arguments"));
        }

        let list = args[0].to_list();
        let separator = args[1].to_string();

        let strings: Vec<String> = list.iter().map(|v| v.to_string()).collect();
        Ok(LSLValue::String(strings.join(&separator)))
    }

    async fn ll_parse_string2list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 3 {
            return Err(anyhow!("llParseString2List expects at least 3 arguments"));
        }

        let source = args[0].to_string();
        let separators = args[1].to_list();
        let spacers = args[2].to_list();

        let sep_strings: Vec<String> = separators.iter().map(|v| v.to_string()).collect();
        let spacer_strings: Vec<String> = spacers.iter().map(|v| v.to_string()).collect();

        let mut result = Vec::new();
        let mut current = source.as_str();

        while !current.is_empty() {
            let mut found_sep = false;
            let mut best_pos = current.len();
            let mut best_len = 0;
            let mut is_spacer = false;

            for sep in &sep_strings {
                if !sep.is_empty() {
                    if let Some(pos) = current.find(sep.as_str()) {
                        if pos < best_pos {
                            best_pos = pos;
                            best_len = sep.len();
                            is_spacer = false;
                            found_sep = true;
                        }
                    }
                }
            }

            for spacer in &spacer_strings {
                if !spacer.is_empty() {
                    if let Some(pos) = current.find(spacer.as_str()) {
                        if pos < best_pos || (pos == best_pos && spacer.len() > best_len) {
                            best_pos = pos;
                            best_len = spacer.len();
                            is_spacer = true;
                            found_sep = true;
                        }
                    }
                }
            }

            if found_sep {
                if best_pos > 0 {
                    result.push(LSLValue::String(current[..best_pos].to_string()));
                }
                if is_spacer {
                    result.push(LSLValue::String(
                        current[best_pos..best_pos + best_len].to_string(),
                    ));
                }
                current = &current[best_pos + best_len..];
            } else {
                result.push(LSLValue::String(current.to_string()));
                break;
            }
        }

        Ok(LSLValue::List(result))
    }

    async fn ll_get_inventory_number(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryNumber expects 1 argument"));
        }

        let inv_type = args[0].to_integer();
        let count = context
            .inventory
            .iter()
            .filter(|item| inv_type < 0 || item.asset_type == inv_type)
            .count();

        Ok(LSLValue::Integer(count as i32))
    }

    async fn ll_get_inventory_name(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetInventoryName expects 2 arguments"));
        }

        let inv_type = args[0].to_integer();
        let index = args[1].to_integer() as usize;

        let items: Vec<_> = context
            .inventory
            .iter()
            .filter(|item| inv_type < 0 || item.asset_type == inv_type)
            .collect();

        if index < items.len() {
            Ok(LSLValue::String(items[index].name.clone()))
        } else {
            Ok(LSLValue::String(String::new()))
        }
    }

    async fn ll_get_inventory_type(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryType expects 1 argument"));
        }

        let name = args[0].to_string();

        if let Some(item) = context.inventory.iter().find(|item| item.name == name) {
            Ok(LSLValue::Integer(item.inv_type))
        } else {
            Ok(LSLValue::Integer(-1))
        }
    }

    async fn ll_get_inventory_key(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryKey expects 1 argument"));
        }

        let name = args[0].to_string();

        if let Some(item) = context.inventory.iter().find(|item| item.name == name) {
            Ok(LSLValue::Key(item.asset_id))
        } else {
            Ok(LSLValue::Key(Uuid::nil()))
        }
    }

    async fn ll_http_request(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llHTTPRequest expects 3 arguments"));
        }

        let url = args[0].to_string();
        let parameters = args[1].to_list();
        let body = args[2].to_string();

        let request_id = Uuid::new_v4();

        let mut params: Vec<(String, String)> = Vec::new();
        let mut i = 0;
        while i + 1 < parameters.len() {
            let param_key = parameters[i].to_integer();
            let param_val = parameters[i + 1].to_string();
            params.push((param_key.to_string(), param_val));
            i += 2;
        }

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::HttpRequest {
                object_id: context.object_id,
                script_id: context.script_id,
                url,
                params,
                body,
            },
        ));

        debug!(
            "Queued HTTP request {} for script {}",
            request_id, context.script_id
        );
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_http_response(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llHTTPResponse expects 3 arguments"));
        }

        let _request_id = args[0].to_key();
        let _status = args[1].to_integer();
        let _body = args[2].to_string();

        debug!("Sending HTTP response (server capability required)");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sensor(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llSensor expects 5 arguments"));
        }

        let name = args[0].to_string();
        let id = args[1].to_key();
        let sensor_type = args[2].to_integer();
        let range = args[3].to_float().min(96.0);
        let arc = args[4].to_float();

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SensorRequest {
                object_id: context.object_id,
                script_id: context.script_id,
                name,
                key: id,
                sensor_type,
                range: range as f64,
                arc: arc as f64,
                repeat: false,
                interval: 0.0,
            },
        ));

        debug!(
            "Queued single sensor sweep, type={}, range={}, arc={}",
            sensor_type, range, arc
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sensor_repeat(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 6 {
            return Err(anyhow!("llSensorRepeat expects 6 arguments"));
        }

        let name = args[0].to_string();
        let id = args[1].to_key();
        let sensor_type = args[2].to_integer();
        let range = args[3].to_float().min(96.0);
        let arc = args[4].to_float();
        let rate = args[5].to_float();

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SensorRequest {
                object_id: context.object_id,
                script_id: context.script_id,
                name,
                key: id,
                sensor_type,
                range: range as f64,
                arc: arc as f64,
                repeat: true,
                interval: rate as f64,
            },
        ));

        debug!(
            "Queued repeating sensor, type={}, range={}, arc={}, rate={}",
            sensor_type, range, arc, rate
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sensor_remove(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SensorRemove {
                object_id: context.object_id,
            },
        ));
        debug!("Queued sensor removal");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_script_name(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::String(context.script_name.clone()))
    }

    async fn ll_reset_script(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        context.variables.clear();
        context.timers.clear();
        context.listeners.clear();
        context.active_sensor = None;
        context.pending_http_requests.clear();
        context.floating_text = None;

        debug!("Reset script {}", context.script_name);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_apply_impulse(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llApplyImpulse expects 2 arguments"));
        }
        let impulse = args[0].to_vector();
        let local = args[1].is_true();
        context.velocity.0 += impulse.x;
        context.velocity.1 += impulse.y;
        context.velocity.2 += impulse.z;

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ApplyImpulse {
                object_id: context.object_id,
                impulse: [impulse.x, impulse.y, impulse.z],
                local,
            },
        ));

        debug!(
            "Applied impulse ({},{},{}), local={}",
            impulse.x, impulse.y, impulse.z, local
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_apply_rotational_impulse(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llApplyRotationalImpulse expects 2 arguments"));
        }
        let impulse = args[0].to_vector();
        let local = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ApplyRotationalImpulse {
                object_id: context.object_id,
                impulse: [impulse.x as f32, impulse.y as f32, impulse.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_force(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetForce expects 2 arguments"));
        }
        let force = args[0].to_vector();
        let local = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetForce {
                object_id: context.object_id,
                force: [force.x as f32, force.y as f32, force.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_force(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_set_torque(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetTorque expects 2 arguments"));
        }
        let torque = args[0].to_vector();
        let local = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetTorque {
                object_id: context.object_id,
                torque: [torque.x as f32, torque.y as f32, torque.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_torque(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_set_buoyancy(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetBuoyancy expects 1 argument"));
        }
        let buoyancy = args[0].to_float().clamp(-1.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetBuoyancy {
                object_id: context.object_id,
                buoyancy,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_hover_height(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetHoverHeight expects 3 arguments"));
        }
        let height = args[0].to_float() as f32;
        let water = args[1].to_integer();
        let tau = args[2].to_float().max(0.1) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetHoverHeight {
                object_id: context.object_id,
                height,
                water,
                tau,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_hover(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopHover {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_move_to_target(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llMoveToTarget expects 2 arguments"));
        }
        let target = args[0].to_vector();
        let tau = args[1].to_float().max(0.1) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::MoveToTarget {
                object_id: context.object_id,
                target: [target.x as f32, target.y as f32, target.z as f32],
                tau,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_move_to_target(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopMoveToTarget {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_push_object(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llPushObject expects 4 arguments"));
        }
        let target_id = args[0].to_key();
        let impulse = args[1].to_vector();
        let angular_impulse = args[2].to_vector();
        let local = args[3].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PushObject {
                target_id,
                impulse: [impulse.x as f32, impulse.y as f32, impulse.z as f32],
                angular_impulse: [
                    angular_impulse.x as f32,
                    angular_impulse.y as f32,
                    angular_impulse.z as f32,
                ],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_accel(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_get_omega(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_target_omega(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llTargetOmega expects 3 arguments"));
        }
        let axis = args[0].to_vector();
        let spinrate = args[1].to_float();
        let gain = args[2].to_float();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TargetOmega {
                object_id: context.object_id,
                axis: [axis.x as f32, axis.y as f32, axis.z as f32],
                spinrate,
                gain,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_angular_velocity(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetAngularVelocity expects 2 arguments"));
        }
        let omega = args[0].to_vector();
        let local = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetAngularVelocity {
                object_id: context.object_id,
                velocity: [omega.x as f32, omega.y as f32, omega.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_mass(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_get_mass_mks(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_get_object_mass(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetObjectMass expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_ground_repel(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llGroundRepel expects 3 arguments"));
        }
        let height = args[0].to_float() as f32;
        let water = args[1].to_integer();
        let tau = args[2].to_float().max(0.1) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::GroundRepel {
                object_id: context.object_id,
                height,
                water,
                tau,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_force_and_torque(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetForceAndTorque expects 3 arguments"));
        }
        let force = args[0].to_vector();
        let torque = args[1].to_vector();
        let local = args[2].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetForceAndTorque {
                object_id: context.object_id,
                force: [force.x as f32, force.y as f32, force.z as f32],
                torque: [torque.x as f32, torque.y as f32, torque.z as f32],
                local,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_physics_material(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llSetPhysicsMaterial expects 5 arguments"));
        }
        let flags = args[0].to_integer();
        let gravity = args[1].to_float() as f32;
        let restitution = args[2].to_float() as f32;
        let friction = args[3].to_float() as f32;
        let density = args[4].to_float() as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPhysicsMaterial {
                object_id: context.object_id,
                gravity,
                restitution,
                friction,
                density,
                flags,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_physics_material(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::List(vec![
            LSLValue::Float(1.0),
            LSLValue::Float(0.5),
            LSLValue::Float(0.5),
            LSLValue::Float(1000.0),
        ]))
    }

    async fn ll_start_animation(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStartAnimation expects 1 argument"));
        }
        if context.permissions & 0x0010 == 0 {
            return Ok(LSLValue::Integer(0));
        }
        let anim = args[0].to_string();
        if anim.is_empty() {
            return Ok(LSLValue::Integer(0));
        }
        let mut active: Vec<LSLValue> = match context.variables.get("__active_animations") {
            Some(LSLValue::List(l)) => l.clone(),
            _ => Vec::new(),
        };
        let anim_val = LSLValue::String(anim.clone());
        if !active.contains(&anim_val) {
            active.push(anim_val);
        }
        context
            .variables
            .insert("__active_animations".to_string(), LSLValue::List(active));
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StartAnimation {
                avatar_id: context.permission_key,
                anim_name: anim,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_animation(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStopAnimation expects 1 argument"));
        }
        if context.permissions & 0x0010 == 0 {
            return Ok(LSLValue::Integer(0));
        }
        let anim = args[0].to_string();
        if let Some(LSLValue::List(ref mut active)) =
            context.variables.get_mut("__active_animations")
        {
            active.retain(|v| v.to_string() != anim);
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopAnimation {
                avatar_id: context.permission_key,
                anim_name: anim,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_animation(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAnimation expects 1 argument"));
        }
        let id = args[0].to_key();
        if id.is_nil() {
            return Ok(LSLValue::String(String::new()));
        }
        Ok(LSLValue::String("Standing".to_string()))
    }

    async fn ll_get_animation_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAnimationList expects 1 argument"));
        }
        let id = args[0].to_key();
        if id.is_nil() {
            return Ok(LSLValue::List(vec![]));
        }
        Ok(LSLValue::List(vec![]))
    }

    async fn ll_set_animation_override(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetAnimationOverride expects 2 arguments"));
        }
        if context.permissions & 0x8000 == 0 {
            return Ok(LSLValue::Integer(0));
        }
        let anim_state = args[0].to_string();
        let anim = args[1].to_string();
        let valid_states = [
            "Crouching",
            "CrouchWalking",
            "Falling Down",
            "Flying",
            "FlyingSlow",
            "Hovering",
            "Hovering Down",
            "Hovering Up",
            "Jumping",
            "Landing",
            "PreJumping",
            "Running",
            "Sitting",
            "Sitting on Ground",
            "Standing",
            "Standing Up",
            "Striding",
            "Soft Landing",
            "Taking Off",
            "Turning Left",
            "Turning Right",
            "Walking",
        ];
        if !valid_states.iter().any(|s| *s == anim_state) {
            return Ok(LSLValue::Integer(0));
        }
        context.variables.insert(
            format!("__anim_override_{}", anim_state),
            LSLValue::String(anim.clone()),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetAnimationOverride {
                avatar_id: context.permission_key,
                anim_state: anim_state.clone(),
                anim_name: anim,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_animation_override(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAnimationOverride expects 1 argument"));
        }
        if context.permissions & 0x8000 == 0 {
            return Ok(LSLValue::String(String::new()));
        }
        let anim_state = args[0].to_string();
        match context
            .variables
            .get(&format!("__anim_override_{}", anim_state))
        {
            Some(LSLValue::String(s)) => Ok(LSLValue::String(s.clone())),
            _ => Ok(LSLValue::String(String::new())),
        }
    }

    async fn ll_reset_animation_override(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llResetAnimationOverride expects 1 argument"));
        }
        if context.permissions & 0x8000 == 0 {
            return Ok(LSLValue::Integer(0));
        }
        let anim_state = args[0].to_string();
        if anim_state == "ALL" {
            let keys: Vec<String> = context
                .variables
                .keys()
                .filter(|k| k.starts_with("__anim_override_"))
                .cloned()
                .collect();
            for key in keys {
                context.variables.remove(&key);
            }
        } else {
            context
                .variables
                .remove(&format!("__anim_override_{}", anim_state));
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ResetAnimationOverride {
                avatar_id: context.permission_key,
                anim_state: anim_state.clone(),
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_start_object_animation(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStartObjectAnimation expects 1 argument"));
        }
        let anim = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StartObjectAnimation {
                object_id: context.object_id,
                anim_name: anim,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_object_animation(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStopObjectAnimation expects 1 argument"));
        }
        let anim = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopObjectAnimation {
                object_id: context.object_id,
                anim_name: anim,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_object_animation_names(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::List(vec![]))
    }

    async fn ll_play_sound(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llPlaySound expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PlaySound {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_loop_sound(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLoopSound expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LoopSound {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_loop_sound_master(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLoopSoundMaster expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LoopSoundMaster {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_loop_sound_slave(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLoopSoundSlave expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LoopSoundSlave {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_play_sound_slave(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llPlaySoundSlave expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PlaySoundSlave {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_sound(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopSound {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_trigger_sound(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llTriggerSound expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TriggerSound {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_trigger_sound_limited(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llTriggerSoundLimited expects 5 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0) as f32;
        let top_ne = args[2].to_vector();
        let bot_sw = args[3].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TriggerSoundLimited {
                object_id: context.object_id,
                sound_id,
                volume,
                top_ne: [top_ne.x as f32, top_ne.y as f32, top_ne.z as f32],
                bot_sw: [bot_sw.x as f32, bot_sw.y as f32, bot_sw.z as f32],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_preload_sound(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llPreloadSound expects 1 argument"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PreloadSound {
                object_id: context.object_id,
                sound_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sound(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 2 {
            return Err(anyhow!("llSound expects at least 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PlaySound {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sound_preload(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSoundPreload expects 1 argument"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::PreloadSound {
                object_id: context.object_id,
                sound_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_adjust_sound_volume(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAdjustSoundVolume expects 1 argument"));
        }
        let volume = args[0].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AdjustSoundVolume {
                object_id: context.object_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_sound_queueing(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetSoundQueueing expects 1 argument"));
        }
        let queueing = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSoundQueueing {
                object_id: context.object_id,
                queueing,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_sound_radius(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetSoundRadius expects 1 argument"));
        }
        let radius = args[0].to_float().max(0.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSoundRadius {
                object_id: context.object_id,
                radius,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_play_sound(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llLinkPlaySound expects 3 arguments"));
        }
        let link_num = args[0].to_integer();
        let sound_id = Uuid::parse_str(&args[1].to_string()).unwrap_or(Uuid::nil());
        let volume = args[2].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LinkPlaySound {
                object_id: context.object_id,
                link_num,
                sound_id,
                volume,
                flags: 0,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_stop_sound(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLinkStopSound expects 1 argument"));
        }
        let link_num = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LinkStopSound {
                object_id: context.object_id,
                link_num,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_adjust_sound_volume(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinkAdjustSoundVolume expects 2 arguments"));
        }
        let link_num = args[0].to_integer();
        let volume = args[1].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LinkAdjustSoundVolume {
                object_id: context.object_id,
                link_num,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_set_sound_queueing(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinkSetSoundQueueing expects 2 arguments"));
        }
        let _link = args[0].to_integer();
        let queueing = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSoundQueueing {
                object_id: context.object_id,
                queueing,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_set_sound_radius(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinkSetSoundRadius expects 2 arguments"));
        }
        let _link = args[0].to_integer();
        let radius = args[1].to_float().max(0.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSoundRadius {
                object_id: context.object_id,
                radius,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_collision_sound(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llCollisionSound expects 2 arguments"));
        }
        let sound_id = Uuid::parse_str(&args[0].to_string()).unwrap_or(Uuid::nil());
        let volume = args[1].to_float().clamp(0.0, 1.0) as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCollisionSound {
                object_id: context.object_id,
                sound_id,
                volume,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_texture(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetTexture expects 2 arguments"));
        }
        let texture = args[0].to_string();
        let face = args[1].to_integer();
        let texture_id = Uuid::parse_str(&texture)
            .unwrap_or_else(|_| Uuid::parse_str("89556747-24cb-43ed-920b-47caed15465f").unwrap());
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetTexture {
                object_id: context.object_id,
                texture_id,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_texture(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetTexture expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::String(Uuid::nil().to_string()))
    }

    async fn ll_set_color(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetColor expects 2 arguments"));
        }
        let color = args[0].to_vector();
        let face = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetColor {
                object_id: context.object_id,
                color: [
                    color.x.clamp(0.0, 1.0),
                    color.y.clamp(0.0, 1.0),
                    color.z.clamp(0.0, 1.0),
                ],
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_color(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetColor expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)))
    }

    async fn ll_set_alpha(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetAlpha expects 2 arguments"));
        }
        let alpha = args[0].to_float();
        let face = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetAlpha {
                object_id: context.object_id,
                alpha: alpha.clamp(0.0, 1.0) as f32,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_alpha(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAlpha expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_set_scale(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetScale expects 1 argument"));
        }
        let scale = args[0].to_vector();
        context.scale = (scale.x, scale.y, scale.z);

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetScale {
                object_id: context.object_id,
                scale: [scale.x, scale.y, scale.z],
            },
        ));

        debug!("Setting scale to {:?}", scale);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_scale(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(
            context.scale.0,
            context.scale.1,
            context.scale.2,
        )))
    }

    async fn ll_scale_texture(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llScaleTexture expects 3 arguments"));
        }
        let u = args[0].to_float() as f32;
        let v = args[1].to_float() as f32;
        let face = args[2].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ScaleTexture {
                object_id: context.object_id,
                u,
                v,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_offset_texture(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llOffsetTexture expects 3 arguments"));
        }
        let u = args[0].to_float() as f32;
        let v = args[1].to_float() as f32;
        let face = args[2].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::OffsetTexture {
                object_id: context.object_id,
                u,
                v,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rotate_texture(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRotateTexture expects 2 arguments"));
        }
        let rotation = args[0].to_float() as f32;
        let face = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RotateTexture {
                object_id: context.object_id,
                rotation,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_texture_offset(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetTextureOffset expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_get_texture_scale(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetTextureScale expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0)))
    }

    async fn ll_get_texture_rot(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetTextureRot expects 1 argument"));
        }
        let _face = args[0].to_integer();
        Ok(LSLValue::Float(0.0))
    }

    async fn ll_set_texture_anim(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 7 {
            return Err(anyhow!("llSetTextureAnim expects 7 arguments"));
        }
        let mode = args[0].to_integer();
        let face = args[1].to_integer();
        let size_x = args[2].to_integer();
        let size_y = args[3].to_integer();
        let start = args[4].to_float() as f32;
        let length = args[5].to_float() as f32;
        let rate = args[6].to_float() as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetTextureAnim {
                object_id: context.object_id,
                mode,
                face,
                size_x,
                size_y,
                start,
                length,
                rate,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_texture(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetLinkTexture expects 3 arguments"));
        }
        let link = args[0].to_integer();
        let texture = args[1].to_string();
        let face = args[2].to_integer();
        let texture_id = Uuid::parse_str(&texture).unwrap_or(Uuid::nil());
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetLinkTexture {
                object_id: context.object_id,
                link_num: link,
                texture_id,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_color(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetLinkColor expects 3 arguments"));
        }
        let link = args[0].to_integer();
        let color = args[1].to_vector();
        let face = args[2].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetLinkColor {
                object_id: context.object_id,
                link_num: link,
                color: [
                    color.x.clamp(0.0, 1.0),
                    color.y.clamp(0.0, 1.0),
                    color.z.clamp(0.0, 1.0),
                ],
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_alpha(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetLinkAlpha expects 3 arguments"));
        }
        let link = args[0].to_integer();
        let alpha = args[1].to_float();
        let face = args[2].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetLinkAlpha {
                object_id: context.object_id,
                link_num: link,
                alpha: alpha.clamp(0.0, 1.0) as f32,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_texture_anim(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 8 {
            return Err(anyhow!("llSetLinkTextureAnim expects 8 arguments"));
        }
        let link = args[0].to_integer();
        let mode = args[1].to_integer();
        let face = args[2].to_integer();
        let size_x = args[3].to_integer();
        let size_y = args[4].to_integer();
        let start = args[5].to_float() as f32;
        let length = args[6].to_float() as f32;
        let rate = args[7].to_float() as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetLinkTextureAnim {
                object_id: context.object_id,
                link_num: link,
                mode,
                face,
                sizex: size_x,
                sizey: size_y,
                start,
                length,
                rate,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    fn apply_prim_params(&self, params: &[LSLValue], context: &mut ScriptContext) {
        use crate::scripting::executor::PrimParamRule;
        let mut rules: Vec<PrimParamRule> = Vec::new();
        let mut i = 0;
        while i < params.len() {
            let code = params[i].to_integer();
            i += 1;
            match code {
                2 => {
                    // PRIM_MATERIAL
                    if i < params.len() {
                        rules.push(PrimParamRule::Material {
                            material: params[i].to_integer(),
                        });
                        i += 1;
                    }
                }
                3 => {
                    // PRIM_PHYSICS
                    if i < params.len() {
                        rules.push(PrimParamRule::Physics {
                            value: params[i].is_true(),
                        });
                        i += 1;
                    }
                }
                4 => {
                    // PRIM_TEMP_ON_REZ
                    if i < params.len() {
                        rules.push(PrimParamRule::TempOnRez {
                            value: params[i].is_true(),
                        });
                        i += 1;
                    }
                }
                5 => {
                    // PRIM_PHANTOM
                    if i < params.len() {
                        rules.push(PrimParamRule::Phantom {
                            value: params[i].is_true(),
                        });
                        i += 1;
                    }
                }
                6 => {
                    // PRIM_POSITION
                    if i < params.len() {
                        let v = params[i].to_vector();
                        context.position = (v.x, v.y, v.z);
                        rules.push(PrimParamRule::Position {
                            pos: [v.x, v.y, v.z],
                        });
                        i += 1;
                    }
                }
                7 => {
                    // PRIM_SIZE
                    if i < params.len() {
                        let v = params[i].to_vector();
                        let s = [
                            v.x.clamp(0.01, 64.0),
                            v.y.clamp(0.01, 64.0),
                            v.z.clamp(0.01, 64.0),
                        ];
                        context.scale = (s[0], s[1], s[2]);
                        rules.push(PrimParamRule::Size { size: s });
                        i += 1;
                    }
                }
                8 => {
                    // PRIM_ROTATION
                    if i < params.len() {
                        let r = params[i].to_rotation();
                        context.rotation = (r.x, r.y, r.z, r.s);
                        rules.push(PrimParamRule::Rotation {
                            rot: [r.x, r.y, r.z, r.s],
                        });
                        i += 1;
                    }
                }
                9 => {
                    // PRIM_TYPE - shape_type + shape-specific params (skip but consume)
                    if i < params.len() {
                        let shape = params[i].to_integer();
                        i += 1;
                        let skip = match shape {
                            0 => 7,
                            1 => 7,
                            2 => 5,
                            3 => 5,
                            4 => 4,
                            5 => 4,
                            6 => 4,
                            7 => 4,
                            _ => 0,
                        };
                        let extra = if shape >= 4 && shape <= 6 {
                            5
                        } else if shape == 7 {
                            2
                        } else {
                            0
                        };
                        i += (skip + extra).min(params.len() - i);
                        debug!("Set PRIM_TYPE shape={}", shape);
                    }
                }
                17 => {
                    // PRIM_TEXTURE: face, texture, repeats, offsets, rotation
                    if i + 4 < params.len() {
                        let face = params[i].to_integer();
                        let tex_str = params[i + 1].to_string();
                        let repeats = params[i + 2].to_vector();
                        let offsets = params[i + 3].to_vector();
                        let rotation = params[i + 4].to_float() as f32;
                        let texture_id = Uuid::parse_str(&tex_str).unwrap_or_else(|_| {
                            Uuid::parse_str("89556747-24cb-43ed-920b-47caed15465f").unwrap()
                        });
                        rules.push(PrimParamRule::Texture {
                            face,
                            texture_id,
                            repeats: [repeats.x, repeats.y],
                            offsets: [offsets.x, offsets.y],
                            rotation,
                        });
                    }
                    i += 5.min(params.len() - i);
                }
                18 => {
                    // PRIM_COLOR: face, color, alpha
                    if i + 2 < params.len() {
                        let face = params[i].to_integer();
                        let color = params[i + 1].to_vector();
                        let alpha = params[i + 2].to_float() as f32;
                        rules.push(PrimParamRule::Color {
                            face,
                            color: [
                                color.x.clamp(0.0, 1.0),
                                color.y.clamp(0.0, 1.0),
                                color.z.clamp(0.0, 1.0),
                            ],
                            alpha: alpha.clamp(0.0, 1.0),
                        });
                    }
                    i += 3.min(params.len() - i);
                }
                19 => {
                    // PRIM_BUMP_SHINY: face, shiny, bump
                    if i + 2 < params.len() {
                        rules.push(PrimParamRule::BumpShiny {
                            face: params[i].to_integer(),
                            shiny: params[i + 1].to_integer(),
                            bump: params[i + 2].to_integer(),
                        });
                    }
                    i += 3.min(params.len() - i);
                }
                20 => {
                    // PRIM_FULLBRIGHT: face, value
                    if i + 1 < params.len() {
                        rules.push(PrimParamRule::Fullbright {
                            face: params[i].to_integer(),
                            value: params[i + 1].is_true(),
                        });
                    }
                    i += 2.min(params.len() - i);
                }
                21 => {
                    // PRIM_FLEXIBLE: active, softness, gravity, friction, wind, tension, force
                    if i + 6 < params.len() {
                        let force = params[i + 6].to_vector();
                        rules.push(PrimParamRule::Flexible {
                            enabled: params[i].is_true(),
                            softness: params[i + 1].to_integer(),
                            gravity: params[i + 2].to_float() as f32,
                            friction: params[i + 3].to_float() as f32,
                            wind: params[i + 4].to_float() as f32,
                            tension: params[i + 5].to_float() as f32,
                            force: [force.x, force.y, force.z],
                        });
                    }
                    i += 7.min(params.len() - i);
                }
                22 => {
                    // PRIM_TEXGEN: face, value
                    i += 2.min(params.len() - i);
                }
                23 => {
                    // PRIM_POINT_LIGHT: active, color, intensity, radius, falloff
                    if i + 4 < params.len() {
                        let color = params[i + 1].to_vector();
                        rules.push(PrimParamRule::PointLight {
                            enabled: params[i].is_true(),
                            color: [color.x, color.y, color.z],
                            intensity: params[i + 2].to_float() as f32,
                            radius: params[i + 3].to_float() as f32,
                            falloff: params[i + 4].to_float() as f32,
                        });
                    }
                    i += 5.min(params.len() - i);
                }
                25 => {
                    // PRIM_GLOW: face, intensity
                    if i + 1 < params.len() {
                        rules.push(PrimParamRule::Glow {
                            face: params[i].to_integer(),
                            intensity: params[i + 1].to_float() as f32,
                        });
                    }
                    i += 2.min(params.len() - i);
                }
                26 => {
                    // PRIM_TEXT: text, color, alpha
                    if i + 2 < params.len() {
                        let text = params[i].to_string();
                        let color = params[i + 1].to_vector();
                        let alpha = params[i + 2].to_float();
                        context.floating_text = if text.is_empty() {
                            None
                        } else {
                            Some(super::FloatingText {
                                text,
                                color: (color.x, color.y, color.z),
                                alpha,
                            })
                        };
                    }
                    i += 3.min(params.len() - i);
                }
                27 => {
                    // PRIM_NAME
                    if i < params.len() {
                        context.object_name = params[i].to_string();
                        i += 1;
                    }
                }
                28 => {
                    // PRIM_DESC
                    if i < params.len() {
                        context.object_description = params[i].to_string();
                        i += 1;
                    }
                }
                29 => {
                    // PRIM_ROT_LOCAL
                    if i < params.len() {
                        let r = params[i].to_rotation();
                        context.rotation = (r.x, r.y, r.z, r.s);
                        i += 1;
                    }
                }
                30 => {
                    // PRIM_PHYSICS_SHAPE_TYPE
                    if i < params.len() {
                        debug!("Set PRIM_PHYSICS_SHAPE_TYPE={}", params[i].to_integer());
                        i += 1;
                    }
                }
                32 => {
                    // PRIM_OMEGA: axis, spinrate, gain
                    if i + 2 < params.len() {
                        let axis = params[i].to_vector();
                        let spinrate = params[i + 1].to_float() as f64;
                        let gain = params[i + 2].to_float() as f64;
                        self.action_queue.lock().push((
                            context.script_id,
                            ScriptAction::SetOmega {
                                object_id: context.object_id,
                                link_num: context.link_number,
                                axis: [axis.x, axis.y, axis.z],
                                spinrate,
                                gain,
                            },
                        ));
                        i += 3;
                    } else {
                        i += 3.min(params.len() - i);
                    }
                }
                33 => {
                    // PRIM_POS_LOCAL
                    if i < params.len() {
                        let v = params[i].to_vector();
                        context.position = (v.x, v.y, v.z);
                        i += 1;
                    }
                }
                34 => {
                    // PRIM_LINK_TARGET
                    if i < params.len() {
                        debug!("Set PRIM_LINK_TARGET={}", params[i].to_integer());
                        i += 1;
                    }
                }
                35 => {
                    // PRIM_SLICE
                    if i < params.len() {
                        i += 1;
                    }
                }
                36 => {
                    // PRIM_SPECULAR: face, texture, repeats, offsets, rotation, color, env_intensity
                    i += 7.min(params.len() - i);
                }
                37 => {
                    // PRIM_NORMAL: face, texture, repeats, offsets, rotation
                    i += 5.min(params.len() - i);
                }
                38 => {
                    // PRIM_ALPHA_MODE: face, mode, mask_cutoff
                    i += 3.min(params.len() - i);
                }
                39 => {
                    // PRIM_ALLOW_UNSIT
                    if i < params.len() {
                        i += 1;
                    }
                }
                40 => {
                    // PRIM_SCRIPTED_SIT_ONLY
                    if i < params.len() {
                        i += 1;
                    }
                }
                41 => {
                    // PRIM_SIT_TARGET: active, offset, rotation
                    i += 3.min(params.len() - i);
                }
                _ => {
                    debug!("Unknown PRIM param code {}", code);
                    break;
                }
            }
        }
        if !rules.is_empty() {
            self.action_queue.lock().push((
                context.script_id,
                ScriptAction::SetPrimParams {
                    object_id: context.object_id,
                    link_num: context.link_number,
                    rules,
                },
            ));
        }
    }

    fn apply_prim_params_for_link(
        &self,
        link_num: i32,
        params: &[LSLValue],
        context: &mut ScriptContext,
    ) {
        use crate::scripting::executor::PrimParamRule;
        let mut rules: Vec<PrimParamRule> = Vec::new();
        let mut i = 0;
        while i < params.len() {
            let code = params[i].to_integer();
            i += 1;
            match code {
                6 => {
                    if i < params.len() {
                        let v = params[i].to_vector();
                        rules.push(PrimParamRule::Position {
                            pos: [v.x, v.y, v.z],
                        });
                        i += 1;
                    }
                }
                7 => {
                    if i < params.len() {
                        let v = params[i].to_vector();
                        let s = [
                            v.x.clamp(0.01, 64.0),
                            v.y.clamp(0.01, 64.0),
                            v.z.clamp(0.01, 64.0),
                        ];
                        rules.push(PrimParamRule::Size { size: s });
                        i += 1;
                    }
                }
                8 => {
                    if i < params.len() {
                        let r = params[i].to_rotation();
                        rules.push(PrimParamRule::Rotation {
                            rot: [r.x, r.y, r.z, r.s],
                        });
                        i += 1;
                    }
                }
                17 => {
                    if i + 4 < params.len() {
                        let face = params[i].to_integer();
                        let tex_str = params[i + 1].to_string();
                        let repeats = params[i + 2].to_vector();
                        let offsets = params[i + 3].to_vector();
                        let rotation = params[i + 4].to_float() as f32;
                        let texture_id = Uuid::parse_str(&tex_str).unwrap_or_else(|_| {
                            Uuid::parse_str("89556747-24cb-43ed-920b-47caed15465f").unwrap()
                        });
                        rules.push(PrimParamRule::Texture {
                            face,
                            texture_id,
                            repeats: [repeats.x, repeats.y],
                            offsets: [offsets.x, offsets.y],
                            rotation,
                        });
                    }
                    i += 5.min(params.len() - i);
                }
                18 => {
                    if i + 2 < params.len() {
                        let face = params[i].to_integer();
                        let color = params[i + 1].to_vector();
                        let alpha = params[i + 2].to_float() as f32;
                        rules.push(PrimParamRule::Color {
                            face,
                            color: [
                                color.x.clamp(0.0, 1.0),
                                color.y.clamp(0.0, 1.0),
                                color.z.clamp(0.0, 1.0),
                            ],
                            alpha: alpha.clamp(0.0, 1.0),
                        });
                    }
                    i += 3.min(params.len() - i);
                }
                20 => {
                    if i + 1 < params.len() {
                        rules.push(PrimParamRule::Fullbright {
                            face: params[i].to_integer(),
                            value: params[i + 1].is_true(),
                        });
                    }
                    i += 2.min(params.len() - i);
                }
                25 => {
                    if i + 1 < params.len() {
                        rules.push(PrimParamRule::Glow {
                            face: params[i].to_integer(),
                            intensity: params[i + 1].to_float() as f32,
                        });
                    }
                    i += 2.min(params.len() - i);
                }
                2 => {
                    if i < params.len() {
                        rules.push(PrimParamRule::Material {
                            material: params[i].to_integer(),
                        });
                        i += 1;
                    }
                }
                3 => {
                    if i < params.len() {
                        rules.push(PrimParamRule::Physics {
                            value: params[i].is_true(),
                        });
                        i += 1;
                    }
                }
                5 => {
                    if i < params.len() {
                        rules.push(PrimParamRule::Phantom {
                            value: params[i].is_true(),
                        });
                        i += 1;
                    }
                }
                29 => {
                    if i < params.len() {
                        let r = params[i].to_rotation();
                        rules.push(PrimParamRule::Rotation {
                            rot: [r.x, r.y, r.z, r.s],
                        });
                        i += 1;
                    }
                }
                32 => {
                    if i + 2 < params.len() {
                        let axis = params[i].to_vector();
                        let spinrate = params[i + 1].to_float() as f64;
                        let gain = params[i + 2].to_float() as f64;
                        self.action_queue.lock().push((
                            context.script_id,
                            ScriptAction::SetOmega {
                                object_id: context.object_id,
                                link_num,
                                axis: [axis.x, axis.y, axis.z],
                                spinrate,
                                gain,
                            },
                        ));
                        i += 3;
                    } else {
                        i += 3.min(params.len() - i);
                    }
                }
                33 => {
                    if i < params.len() {
                        let v = params[i].to_vector();
                        rules.push(PrimParamRule::Position {
                            pos: [v.x, v.y, v.z],
                        });
                        i += 1;
                    }
                }
                9 => {
                    if i < params.len() {
                        let shape = params[i].to_integer();
                        i += 1;
                        let skip = match shape {
                            0 => 7,
                            1 => 7,
                            2 => 5,
                            3 => 5,
                            4 => 4,
                            5 => 4,
                            6 => 4,
                            7 => 4,
                            _ => 0,
                        };
                        let extra = if shape >= 4 && shape <= 6 {
                            5
                        } else if shape == 7 {
                            2
                        } else {
                            0
                        };
                        i += (skip + extra).min(params.len() - i);
                    }
                }
                _ => {
                    i += 1.min(params.len() - i);
                }
            }
        }
        if !rules.is_empty() {
            self.action_queue.lock().push((
                context.script_id,
                ScriptAction::SetPrimParams {
                    object_id: context.object_id,
                    link_num,
                    rules,
                },
            ));
        }
    }

    fn get_prim_params(&self, param_codes: &[LSLValue], context: &ScriptContext) -> Vec<LSLValue> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < param_codes.len() {
            let code = param_codes[i].to_integer();
            i += 1;
            match code {
                2 => result.push(LSLValue::Integer(0)), // PRIM_MATERIAL: PRIM_MATERIAL_WOOD
                3 => result.push(LSLValue::Integer(0)), // PRIM_PHYSICS
                4 => result.push(LSLValue::Integer(0)), // PRIM_TEMP_ON_REZ
                5 => result.push(LSLValue::Integer(0)), // PRIM_PHANTOM
                6 => result.push(LSLValue::Vector(LSLVector::new(
                    context.position.0,
                    context.position.1,
                    context.position.2,
                ))),
                7 => result.push(LSLValue::Vector(LSLVector::new(
                    context.scale.0,
                    context.scale.1,
                    context.scale.2,
                ))),
                8 => result.push(LSLValue::Rotation(LSLRotation::new(
                    context.rotation.0,
                    context.rotation.1,
                    context.rotation.2,
                    context.rotation.3,
                ))),
                9 => {
                    // PRIM_TYPE: returns [shape, ...shape_params]
                    result.push(LSLValue::Integer(0)); // PRIM_TYPE_BOX
                    result.push(LSLValue::Integer(0)); // hole_shape
                    result.push(LSLValue::Vector(LSLVector::new(0.0, 1.0, 0.0))); // cut
                    result.push(LSLValue::Float(0.0)); // hollow
                    result.push(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0))); // twist
                    result.push(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0))); // taper
                    result.push(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)));
                    // top_shear
                }
                17 => {
                    // PRIM_TEXTURE: needs face param
                    let face = if i < param_codes.len() {
                        let f = param_codes[i].to_integer();
                        i += 1;
                        f
                    } else {
                        0
                    };
                    let _ = face;
                    result.push(LSLValue::String(Uuid::nil().to_string())); // texture
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0))); // repeats
                    result.push(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0))); // offsets
                    result.push(LSLValue::Float(0.0)); // rotation
                }
                18 => {
                    // PRIM_COLOR: needs face param
                    let face = if i < param_codes.len() {
                        let f = param_codes[i].to_integer();
                        i += 1;
                        f
                    } else {
                        0
                    };
                    let _ = face;
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0))); // color
                    result.push(LSLValue::Float(1.0)); // alpha
                }
                19 => {
                    // PRIM_BUMP_SHINY: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::Integer(0)); // shiny
                    result.push(LSLValue::Integer(0)); // bump
                }
                20 => {
                    // PRIM_FULLBRIGHT: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::Integer(0));
                }
                21 => {
                    // PRIM_FLEXIBLE
                    result.push(LSLValue::Integer(0)); // active
                    result.push(LSLValue::Integer(0)); // softness
                    result.push(LSLValue::Float(0.0)); // gravity
                    result.push(LSLValue::Float(0.0)); // friction
                    result.push(LSLValue::Float(0.0)); // wind
                    result.push(LSLValue::Float(0.0)); // tension
                    result.push(LSLValue::Vector(LSLVector::zero())); // force
                }
                22 => {
                    // PRIM_TEXGEN: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::Integer(0));
                }
                23 => {
                    // PRIM_POINT_LIGHT
                    result.push(LSLValue::Integer(0)); // active
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0))); // color
                    result.push(LSLValue::Float(1.0)); // intensity
                    result.push(LSLValue::Float(10.0)); // radius
                    result.push(LSLValue::Float(0.75)); // falloff
                }
                25 => {
                    // PRIM_GLOW: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::Float(0.0));
                }
                26 => {
                    // PRIM_TEXT
                    if let Some(ref ft) = context.floating_text {
                        result.push(LSLValue::String(ft.text.clone()));
                        result.push(LSLValue::Vector(LSLVector::new(
                            ft.color.0, ft.color.1, ft.color.2,
                        )));
                        result.push(LSLValue::Float(ft.alpha));
                    } else {
                        result.push(LSLValue::String(String::new()));
                        result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)));
                        result.push(LSLValue::Float(1.0));
                    }
                }
                27 => result.push(LSLValue::String(context.object_name.clone())),
                28 => result.push(LSLValue::String(context.object_description.clone())),
                29 => result.push(LSLValue::Rotation(LSLRotation::new(
                    context.rotation.0,
                    context.rotation.1,
                    context.rotation.2,
                    context.rotation.3,
                ))),
                30 => result.push(LSLValue::Integer(0)), // PRIM_PHYSICS_SHAPE_TYPE
                32 => {
                    // PRIM_OMEGA
                    result.push(LSLValue::Vector(LSLVector::zero())); // axis
                    result.push(LSLValue::Float(0.0)); // spinrate
                    result.push(LSLValue::Float(0.0)); // gain
                }
                33 => result.push(LSLValue::Vector(LSLVector::new(
                    context.position.0,
                    context.position.1,
                    context.position.2,
                ))),
                35 => result.push(LSLValue::Vector(LSLVector::new(0.0, 1.0, 0.0))), // PRIM_SLICE
                36 => {
                    // PRIM_SPECULAR: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::String(Uuid::nil().to_string()));
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0)));
                    result.push(LSLValue::Vector(LSLVector::zero()));
                    result.push(LSLValue::Float(0.0));
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)));
                    result.push(LSLValue::Integer(0)); // env_intensity
                }
                37 => {
                    // PRIM_NORMAL: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::String(Uuid::nil().to_string()));
                    result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0)));
                    result.push(LSLValue::Vector(LSLVector::zero()));
                    result.push(LSLValue::Float(0.0));
                }
                38 => {
                    // PRIM_ALPHA_MODE: needs face
                    if i < param_codes.len() {
                        i += 1;
                    }
                    result.push(LSLValue::Integer(0)); // mode
                    result.push(LSLValue::Integer(0)); // mask_cutoff
                }
                39 => result.push(LSLValue::Integer(0)), // PRIM_ALLOW_UNSIT
                40 => result.push(LSLValue::Integer(0)), // PRIM_SCRIPTED_SIT_ONLY
                41 => {
                    // PRIM_SIT_TARGET
                    result.push(LSLValue::Integer(0)); // active
                    result.push(LSLValue::Vector(LSLVector::zero())); // offset
                    result.push(LSLValue::Rotation(LSLRotation::identity())); // rotation
                }
                _ => {
                    debug!("Unknown PRIM get param code {}", code);
                }
            }
        }
        result
    }

    async fn ll_set_primitive_params(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetPrimitiveParams expects 1 argument (list)"));
        }
        let params = args[0].to_list();
        self.apply_prim_params(&params, context);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_primitive_params(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetPrimitiveParams expects 1 argument (list)"));
        }
        let param_codes = args[0].to_list();
        Ok(LSLValue::List(self.get_prim_params(&param_codes, context)))
    }

    async fn ll_set_link_primitive_params(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetLinkPrimitiveParams expects 2 arguments"));
        }
        let link = args[0].to_integer();
        let params = args[1].to_list();
        if link == 0 || link == context.link_number || link == -4 {
            self.apply_prim_params(&params, context);
        } else {
            self.apply_prim_params_for_link(link, &params, context);
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_primitive_params_fast(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetLinkPrimitiveParamsFast expects 2 arguments"));
        }
        let link = args[0].to_integer();
        let params = args[1].to_list();
        if link == 0 || link == context.link_number || link == -4 {
            self.apply_prim_params(&params, context);
        } else {
            self.apply_prim_params_for_link(link, &params, context);
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_link_primitive_params(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetLinkPrimitiveParams expects 2 arguments"));
        }
        let link = args[0].to_integer();
        let param_codes = args[1].to_list();
        if link == 0 || link == context.link_number || link == -4 {
            Ok(LSLValue::List(self.get_prim_params(&param_codes, context)))
        } else {
            let mut result = Vec::new();
            for param in &param_codes {
                let code = param.to_integer();
                if code == 7 {
                    if let Some((_, scale)) =
                        context.link_scales.iter().find(|(num, _)| *num == link)
                    {
                        result.push(LSLValue::Vector(LSLVector::new(scale.0, scale.1, scale.2)));
                    } else {
                        result.push(LSLValue::Vector(LSLVector::new(1.0, 1.0, 1.0)));
                    }
                }
            }
            Ok(LSLValue::List(result))
        }
    }

    async fn ll_get_number_of_sides(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(6))
    }

    async fn ll_get_link_number_of_sides(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetLinkNumberOfSides expects 1 argument"));
        }
        let _link = args[0].to_integer();
        Ok(LSLValue::Integer(6))
    }

    async fn ll_detected_key(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedKey expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let key = context
            .detected_objects
            .get(index)
            .map(|d| d.key)
            .unwrap_or(Uuid::nil());
        Ok(LSLValue::Key(key))
    }

    async fn ll_detected_name(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedName expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let name = context
            .detected_objects
            .get(index)
            .map(|d| d.name.clone())
            .unwrap_or_default();
        Ok(LSLValue::String(name))
    }

    async fn ll_detected_owner(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedOwner expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let owner = context
            .detected_objects
            .get(index)
            .map(|d| d.owner)
            .unwrap_or(Uuid::nil());
        Ok(LSLValue::Key(owner))
    }

    async fn ll_detected_type(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedType expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let obj_type = context
            .detected_objects
            .get(index)
            .map(|d| d.object_type)
            .unwrap_or(0);
        Ok(LSLValue::Integer(obj_type))
    }

    async fn ll_detected_pos(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedPos expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let pos = context
            .detected_objects
            .get(index)
            .map(|d| LSLVector::new(d.position.0, d.position.1, d.position.2))
            .unwrap_or(LSLVector::new(0.0, 0.0, 0.0));
        Ok(LSLValue::Vector(pos))
    }

    async fn ll_detected_vel(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedVel expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let vel = context
            .detected_objects
            .get(index)
            .map(|d| LSLVector::new(d.velocity.0, d.velocity.1, d.velocity.2))
            .unwrap_or(LSLVector::new(0.0, 0.0, 0.0));
        Ok(LSLValue::Vector(vel))
    }

    async fn ll_detected_rot(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedRot expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let rot = context
            .detected_objects
            .get(index)
            .map(|d| LSLRotation::new(d.rotation.0, d.rotation.1, d.rotation.2, d.rotation.3))
            .unwrap_or(LSLRotation::new(0.0, 0.0, 0.0, 1.0));
        Ok(LSLValue::Rotation(rot))
    }

    async fn ll_detected_group(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedGroup expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let group = context
            .detected_objects
            .get(index)
            .map(|d| d.group)
            .unwrap_or(Uuid::nil());
        Ok(LSLValue::Key(group))
    }

    async fn ll_detected_link_number(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedLinkNumber expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        let link = context
            .detected_objects
            .get(index)
            .map(|d| d.link_number)
            .unwrap_or(0);
        Ok(LSLValue::Integer(link))
    }

    async fn ll_detected_grab(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedGrab expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            let p = det.touch_position;
            return Ok(LSLValue::Vector(LSLVector::new(p.0, p.1, p.2)));
        }
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_detected_touch_face(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchFace expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            return Ok(LSLValue::Integer(det.touch_face));
        }
        Ok(LSLValue::Integer(-1))
    }

    async fn ll_detected_touch_pos(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchPos expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            let p = det.touch_position;
            return Ok(LSLValue::Vector(LSLVector::new(p.0, p.1, p.2)));
        }
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_detected_touch_normal(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchNormal expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            let n = det.touch_normal;
            return Ok(LSLValue::Vector(LSLVector::new(n.0, n.1, n.2)));
        }
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_detected_touch_binormal(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchBinormal expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            let b = det.touch_binormal;
            return Ok(LSLValue::Vector(LSLVector::new(b.0, b.1, b.2)));
        }
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    async fn ll_detected_touch_st(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchST expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            return Ok(LSLValue::Vector(LSLVector::new(
                det.touch_st.0,
                det.touch_st.1,
                0.0,
            )));
        }
        Ok(LSLValue::Vector(LSLVector::new(-1.0, -1.0, 0.0)))
    }

    async fn ll_detected_touch_uv(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llDetectedTouchUV expects 1 argument"));
        }
        let index = args[0].to_integer() as usize;
        if let Some(det) = context.detected_objects.get(index) {
            return Ok(LSLValue::Vector(LSLVector::new(
                det.touch_uv.0,
                det.touch_uv.1,
                0.0,
            )));
        }
        Ok(LSLValue::Vector(LSLVector::new(-1.0, -1.0, 0.0)))
    }

    async fn ll_get_agent_info(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAgentInfo expects 1 argument"));
        }
        let id = args[0].to_key();
        if id == Uuid::nil() {
            return Ok(LSLValue::Integer(0));
        }
        let flags = 0x0001 | 0x0100; // AGENT_WALKING | AGENT_ON_OBJECT (reasonable defaults)
        Ok(LSLValue::Integer(flags))
    }

    async fn ll_get_agent_size(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAgentSize expects 1 argument"));
        }
        let id = args[0].to_key();
        if id == Uuid::nil() {
            return Ok(LSLValue::Vector(LSLVector::zero()));
        }
        Ok(LSLValue::Vector(LSLVector::new(0.45, 0.6, 1.8)))
    }

    async fn ll_get_agent_language(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAgentLanguage expects 1 argument"));
        }
        let id = args[0].to_key();
        if id == Uuid::nil() {
            return Ok(LSLValue::String(String::new()));
        }
        Ok(LSLValue::String("en-us".to_string()))
    }

    async fn ll_get_agent_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetAgentList expects 2 arguments"));
        }
        let scope = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::GetAgentList {
                object_id: context.object_id,
                scope,
            },
        ));
        Ok(LSLValue::List(vec![]))
    }

    async fn ll_request_agent_data(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRequestAgentData expects 2 arguments"));
        }
        let agent_id = args[0].to_key();
        let data_type = args[1].to_integer();
        let request_id = Uuid::new_v4();
        let reply_data = match data_type {
            1 => "1".to_string(),             // DATA_ONLINE
            2 => "Unknown Agent".to_string(), // DATA_NAME
            3 => Uuid::nil().to_string(),     // DATA_BORN
            4 => "0".to_string(),             // DATA_RATING
            5 => "0".to_string(),             // DATA_PAYINFO
            _ => String::new(),
        };
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: reply_data,
            },
        ));
        debug!(
            "llRequestAgentData agent={} type={} query={}",
            agent_id, data_type, request_id
        );
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_key2name(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llKey2Name expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_name2key(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llName2Key expects 1 argument"));
        }
        let _name = args[0].to_string();
        Ok(LSLValue::Key(Uuid::nil()))
    }

    async fn ll_get_display_name(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetDisplayName expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_get_username(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetUsername expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_request_display_name(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRequestDisplayName expects 1 argument"));
        }
        let id = args[0].to_key();
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: format!("Resident {}", id),
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_request_username(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRequestUsername expects 1 argument"));
        }
        let id = args[0].to_key();
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: format!("resident.{}", id),
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_request_user_key(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRequestUserKey expects 1 argument"));
        }
        let name = args[0].to_string();
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: Uuid::nil().to_string(),
            },
        ));
        debug!("llRequestUserKey name='{}' query={}", name, request_id);
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_get_health(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetHealth expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Float(100.0))
    }

    async fn ll_get_energy(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_teleport_agent(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llTeleportAgent expects 4 arguments"));
        }
        let agent_id = args[0].to_key();
        let landmark = args[1].to_string();
        let position = args[2].to_vector();
        let look_at = args[3].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TeleportAgent {
                agent_id,
                landmark,
                position: [position.x, position.y, position.z],
                look_at: [look_at.x, look_at.y, look_at.z],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_teleport_agent_home(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llTeleportAgentHome expects 1 argument"));
        }
        let agent_id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TeleportAgentHome {
                avatar_id: agent_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_teleport_agent_global_coords(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llTeleportAgentGlobalCoords expects 4 arguments"));
        }
        let agent_id = args[0].to_key();
        let _global_coords = args[1].to_vector();
        let position = args[2].to_vector();
        let look_at = args[3].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TeleportAgent {
                agent_id,
                landmark: String::new(),
                position: [position.x, position.y, position.z],
                look_at: [look_at.x, look_at.y, look_at.z],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_eject_from_land(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llEjectFromLand expects 1 argument"));
        }
        let agent_id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::EjectFromLand {
                object_id: context.object_id,
                agent_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_instant_message(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llInstantMessage expects 2 arguments"));
        }
        let target = args[0].to_key();
        let message = args[1].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::InstantMessage {
                target_id: target,
                message,
                object_id: context.object_id,
                object_name: context.object_name.clone(),
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_give_money(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGiveMoney expects 2 arguments"));
        }
        let dest = args[0].to_key();
        let amount = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::GiveMoney {
                owner_id: context.owner_id,
                destination_id: dest,
                amount,
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_transfer_linden_dollars(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llTransferLindenDollars expects 2 arguments"));
        }
        let dest = args[0].to_key();
        let amount = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::GiveMoney {
                owner_id: context.owner_id,
                destination_id: dest,
                amount,
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    async fn ll_request_permissions(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRequestPermissions expects 2 arguments"));
        }
        let agent = args[0].to_key();
        let perm = args[1].to_integer() as u32;
        context.permission_key = agent;

        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RequestPermissions {
                script_id: context.script_id,
                object_id: context.object_id,
                object_name: context.object_name.clone(),
                avatar_id: agent,
                permissions: perm,
            },
        ));

        debug!("Requesting permissions 0x{:X} from {}", perm, agent);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_permissions(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(context.permissions as i32))
    }

    async fn ll_get_permissions_key(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Key(context.permission_key))
    }

    async fn ll_take_controls(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llTakeControls expects 3 arguments"));
        }
        if context.permissions & 0x0004 == 0 {
            warn!("llTakeControls called without PERMISSION_TAKE_CONTROLS");
            return Ok(LSLValue::Integer(0));
        }
        let controls = args[0].to_integer();
        let accept = args[1].is_true();
        let pass_on = args[2].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TakeControls {
                script_id: context.script_id,
                object_id: context.object_id,
                avatar_id: context.permission_key,
                controls,
                accept,
                pass_on,
            },
        ));
        debug!(
            "Taking controls: controls=0x{:X}, accept={}, pass_on={}",
            controls, accept, pass_on
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_release_controls(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if context.permissions & 0x0004 == 0 {
            return Ok(LSLValue::Integer(0));
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ReleaseControls {
                script_id: context.script_id,
                object_id: context.object_id,
                avatar_id: context.permission_key,
            },
        ));
        debug!("Releasing controls");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_take_camera(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llTakeCamera expects 1 argument"));
        }
        let avatar_id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCameraParams {
                avatar_id,
                object_id: context.object_id,
                params: vec![(0, 1.0)],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_release_camera(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llReleaseCamera expects 1 argument"));
        }
        let avatar_id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ClearCameraParams { avatar_id },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_attach_to_avatar(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAttachToAvatar expects 1 argument"));
        }
        let attach_point = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AttachToAvatar {
                object_id: context.object_id,
                attach_point,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_attach_to_avatar_temp(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAttachToAvatarTemp expects 1 argument"));
        }
        let attach_point = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AttachToAvatar {
                object_id: context.object_id,
                attach_point,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_detach_from_avatar(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DetachFromAvatar {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_attached(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_attached_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetAttachedList expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::List(vec![]))
    }

    async fn ll_set_camera_params(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetCameraParams expects 1 argument (list)"));
        }
        let params = args[0].to_list();
        let mut camera_params: Vec<(i32, f32)> = Vec::new();
        let mut i = 0;
        while i + 1 < params.len() {
            let code = params[i].to_integer();
            i += 1;
            match code {
                1 | 13 | 17 => {
                    let v = params[i].to_vector();
                    camera_params.push((code + 1, v.x));
                    camera_params.push((code + 2, v.y));
                    camera_params.push((code + 3, v.z));
                }
                _ => {
                    let val = params[i].to_float() as f32;
                    camera_params.push((code, val));
                }
            }
            i += 1;
        }
        if context.permissions & 0x0800 != 0 {
            self.action_queue.lock().push((
                context.script_id,
                ScriptAction::SetCameraParams {
                    avatar_id: context.permission_key,
                    object_id: context.object_id,
                    params: camera_params.clone(),
                },
            ));
        }
        debug!("SetCameraParams: {} entries", camera_params.len());
        Ok(LSLValue::Integer(0))
    }

    async fn ll_clear_camera_params(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if context.permissions & 0x0800 != 0 {
            self.action_queue.lock().push((
                context.script_id,
                ScriptAction::SetCameraParams {
                    avatar_id: context.permission_key,
                    object_id: context.object_id,
                    params: vec![],
                },
            ));
        }
        debug!("Clearing camera params");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_camera_at_offset(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetCameraAtOffset expects 1 argument"));
        }
        let offset = args[0].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCameraAtOffset {
                object_id: context.object_id,
                offset: [offset.x as f32, offset.y as f32, offset.z as f32],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_camera_eye_offset(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetCameraEyeOffset expects 1 argument"));
        }
        let offset = args[0].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCameraEyeOffset {
                object_id: context.object_id,
                offset: [offset.x as f32, offset.y as f32, offset.z as f32],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_camera_pos(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_get_camera_rot(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
    }

    async fn ll_get_camera_aspect(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.778))
    }

    async fn ll_get_camera_fov(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.0472))
    }

    async fn ll_force_mouselook(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llForceMouselook expects 1 argument"));
        }
        let mouselook = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetStatus {
                object_id: context.object_id,
                status: 0x100,
                value: mouselook,
            },
        ));
        debug!("Force mouselook: {}", mouselook);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_dialog(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llDialog expects 4 arguments"));
        }
        let id = args[0].to_key();
        let message = args[1].to_string();
        let buttons = args[2].to_list();
        let channel = args[3].to_integer();
        let msg = if message.len() > 512 {
            &message[..512]
        } else {
            &message
        };
        let button_count = buttons.len().min(12);
        let button_labels: Vec<String> = buttons
            .iter()
            .take(button_count)
            .map(|b| {
                let label = b.to_string();
                if label.len() > 24 {
                    label[..24].to_string()
                } else {
                    label
                }
            })
            .collect();
        let final_buttons = if button_labels.is_empty() {
            vec!["OK".to_string()]
        } else {
            button_labels
        };
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::Dialog {
                avatar_id: id,
                object_name: context.object_name.clone(),
                message: msg.to_string(),
                buttons: final_buttons,
                channel,
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_text_box(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llTextBox expects 3 arguments"));
        }
        let avatar_id = args[0].to_key();
        let message = args[1].to_string();
        let channel = args[2].to_integer();
        let msg = if message.len() > 512 {
            message[..512].to_string()
        } else {
            message
        };
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::TextBox {
                avatar_id,
                object_name: context.object_name.clone(),
                message: msg,
                channel,
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_map_destination(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llMapDestination expects 3 arguments"));
        }
        let sim_name = args[0].to_string();
        let position = args[1].to_vector();
        let look_at = args[2].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::MapDestination {
                avatar_id: context.permission_key,
                sim_name,
                position: [position.x as f32, position.y as f32, position.z as f32],
                look_at: [look_at.x as f32, look_at.y as f32, look_at.z as f32],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_load_url(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llLoadURL expects 3 arguments"));
        }
        let avatar_id = args[0].to_key();
        let message = args[1].to_string();
        let url = args[2].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LoadURL {
                avatar_id,
                message,
                url,
                object_name: context.object_name.clone(),
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_pay_price(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetPayPrice expects 2 arguments"));
        }
        let price = args[0].to_integer();
        let quick_pay = args[1].to_list();
        let mut prices = [price, -2, -2, -2, -2];
        for (i, v) in quick_pay.iter().enumerate().take(4) {
            prices[i + 1] = v.to_integer();
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPayPrice {
                object_id: context.object_id,
                prices,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_click_action(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetClickAction expects 1 argument"));
        }
        let action = args[0].to_integer() as u8;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetClickAction {
                object_id: context.object_id,
                action,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_sit_text(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetSitText expects 1 argument"));
        }
        let text = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSitText {
                object_id: context.object_id,
                text,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_touch_text(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetTouchText expects 1 argument"));
        }
        let text = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetTouchText {
                object_id: context.object_id,
                text: text.clone(),
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_link_number(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(context.link_number))
    }

    async fn ll_get_link_key(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetLinkKey expects 1 argument"));
        }
        let link = args[0].to_integer();
        if link == 0 || link == context.link_number {
            Ok(LSLValue::Key(context.object_id))
        } else {
            Ok(LSLValue::Key(Uuid::nil()))
        }
    }

    async fn ll_get_link_name(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetLinkName expects 1 argument"));
        }
        let link = args[0].to_integer();
        if link == 0 || link == context.link_number {
            Ok(LSLValue::String(context.object_name.clone()))
        } else if let Some((_, name)) = context.link_names.iter().find(|(num, _)| *num == link) {
            Ok(LSLValue::String(name.clone()))
        } else {
            Ok(LSLValue::String(String::new()))
        }
    }

    async fn ll_get_number_of_prims(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(context.link_count.max(1)))
    }

    async fn ll_get_object_prim_count(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetObjectPrimCount expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Integer(1))
    }

    async fn ll_get_object_link_key(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetObjectLinkKey expects 2 arguments"));
        }
        let _id = args[0].to_key();
        let _link = args[1].to_integer();
        Ok(LSLValue::Key(Uuid::nil()))
    }

    async fn ll_create_link(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llCreateLink expects 2 arguments"));
        }
        let target = args[0].to_key();
        let parent = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::CreateLink {
                object_id: context.object_id,
                target_id: target,
                parent,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_break_link(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llBreakLink expects 1 argument"));
        }
        let link = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::BreakLink {
                object_id: context.object_id,
                link_num: link,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_break_all_links(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::BreakAllLinks {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_message_linked(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llMessageLinked expects 4 arguments"));
        }
        let link = args[0].to_integer();
        let num = args[1].to_integer();
        let str_val = args[2].to_string();
        let id = args[3].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::MessageLinked {
                link_num: link,
                num,
                str_val: str_val.clone(),
                id: id.to_string(),
            },
        ));
        debug!(
            "Message linked: link={}, num={}, str={}, id={}",
            link, num, str_val, id
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_camera(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetLinkCamera expects 3 arguments"));
        }
        let _link = args[0].to_integer();
        let eye_offset = args[1].to_vector();
        let at_offset = args[2].to_vector();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCameraEyeOffset {
                object_id: context.object_id,
                offset: [eye_offset.x, eye_offset.y, eye_offset.z],
            },
        ));
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetCameraAtOffset {
                object_id: context.object_id,
                offset: [at_offset.x, at_offset.y, at_offset.z],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_sit_target(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llLinkSitTarget expects 3 arguments"));
        }
        let _link = args[0].to_integer();
        let offset = args[1].to_vector();
        let rot = args[2].to_rotation();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSitTarget {
                object_id: context.object_id,
                position: [offset.x, offset.y, offset.z],
                rotation: [rot.x, rot.y, rot.z, rot.s],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_sit_target(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSitTarget expects 2 arguments"));
        }
        let offset = args[0].to_vector();
        let rot = args[1].to_rotation();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetSitTarget {
                object_id: context.object_id,
                position: [offset.x, offset.y, offset.z],
                rotation: [rot.x, rot.y, rot.z, rot.s],
            },
        ));
        debug!("Setting sit target offset={:?} rot={:?}", offset, rot);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_avatar_on_sit_target(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Key(context.sitting_avatar_id))
    }

    async fn ll_avatar_on_link_sit_target(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAvatarOnLinkSitTarget expects 1 argument"));
        }
        let _link = args[0].to_integer();
        Ok(LSLValue::Key(context.sitting_avatar_id))
    }

    async fn ll_unsit(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llUnSit expects 1 argument"));
        }
        let id = args[0].to_key();
        self.action_queue
            .lock()
            .push((context.script_id, ScriptAction::UnSit { avatar_id: id }));
        debug!("Unsitting agent {}", id);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_link_sit_flags(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetLinkSitFlags expects 1 argument"));
        }
        let _link = args[0].to_integer();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_sit_flags(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetLinkSitFlags expects 2 arguments"));
        }
        let _link = args[0].to_integer();
        let flags = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetLinkSitFlags {
                object_id: context.object_id,
                flags,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_parcel_flags(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetParcelFlags expects 1 argument"));
        }
        let _pos = args[0].to_vector();
        Ok(LSLValue::Integer(13183))
    }

    async fn ll_get_parcel_details(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetParcelDetails expects 2 arguments"));
        }
        let _pos = args[0].to_vector();
        let details = args[1].to_list();
        let mut result = Vec::new();
        for detail in &details {
            match detail.to_integer() {
                0 => result.push(LSLValue::String(context.region_name.clone())),
                1 => result.push(LSLValue::String(String::new())),
                2 => result.push(LSLValue::Key(context.owner_id)),
                3 => result.push(LSLValue::Key(Uuid::nil())),
                4 => result.push(LSLValue::Integer(65536)),
                5 => result.push(LSLValue::Key(Uuid::new_v4())),
                6 => result.push(LSLValue::Integer(1)),
                7 => result.push(LSLValue::Integer(15000)),
                8 => result.push(LSLValue::Integer(0)),
                9 => result.push(LSLValue::Vector(LSLVector::new(128.0, 128.0, 25.0))),
                10 => result.push(LSLValue::Vector(LSLVector::new(0.0, 1.0, 0.0))),
                11 => result.push(LSLValue::String("none".to_string())),
                12 => result.push(LSLValue::Integer(13183)),
                _ => result.push(LSLValue::String(String::new())),
            }
        }
        Ok(LSLValue::List(result))
    }

    async fn ll_get_parcel_max_prims(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetParcelMaxPrims expects 2 arguments"));
        }
        let _pos = args[0].to_vector();
        let _sim_wide = args[1].is_true();
        Ok(LSLValue::Integer(15000))
    }

    async fn ll_get_parcel_prim_count(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llGetParcelPrimCount expects 3 arguments"));
        }
        let _pos = args[0].to_vector();
        let _category = args[1].to_integer();
        let _sim_wide = args[2].is_true();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_parcel_prim_owners(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetParcelPrimOwners expects 1 argument"));
        }
        let _pos = args[0].to_vector();
        Ok(LSLValue::List(vec![]))
    }

    async fn ll_get_parcel_music_url(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_set_parcel_music_url(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetParcelMusicURL expects 1 argument"));
        }
        let url = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetParcelMusicURL {
                object_id: context.object_id,
                url,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_land_owner_at(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetLandOwnerAt expects 1 argument"));
        }
        let _pos = args[0].to_vector();
        Ok(LSLValue::Key(context.owner_id))
    }

    async fn ll_over_my_land(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llOverMyLand expects 1 argument"));
        }
        let id = args[0].to_key();
        let is_owner = id == context.owner_id;
        Ok(LSLValue::Integer(if is_owner { 1 } else { 0 }))
    }

    async fn ll_modify_land(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llModifyLand expects 2 arguments"));
        }
        let action = args[0].to_integer();
        let size = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ModifyLand {
                object_id: context.object_id,
                action,
                brush_size: size,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_add_to_land_ban_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llAddToLandBanList expects 2 arguments"));
        }
        let id = args[0].to_key();
        let hours = args[1].to_float();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AddToLandBanList {
                object_id: context.object_id,
                agent_id: id,
                hours,
                is_ban: true,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remove_from_land_ban_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRemoveFromLandBanList expects 1 argument"));
        }
        let id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RemoveFromLandBanList {
                object_id: context.object_id,
                agent_id: id,
                is_ban: true,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_add_to_land_pass_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llAddToLandPassList expects 2 arguments"));
        }
        let id = args[0].to_key();
        let hours = args[1].to_float();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AddToLandBanList {
                object_id: context.object_id,
                agent_id: id,
                hours,
                is_ban: false,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remove_from_land_pass_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRemoveFromLandPassList expects 1 argument"));
        }
        let id = args[0].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RemoveFromLandBanList {
                object_id: context.object_id,
                agent_id: id,
                is_ban: false,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_reset_land_ban_list(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ResetLandBanList {
                object_id: context.object_id,
                is_ban: true,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_reset_land_pass_list(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ResetLandBanList {
                object_id: context.object_id,
                is_ban: false,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_vehicle_type(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetVehicleType expects 1 argument"));
        }
        let vehicle_type = args[0].to_integer();
        let valid_types = [0, 1, 2, 3, 4, 5];
        if !valid_types.contains(&vehicle_type) {
            return Ok(LSLValue::Integer(0));
        }
        context.variables.insert(
            "__vehicle_type".to_string(),
            LSLValue::Integer(vehicle_type),
        );
        if vehicle_type == 0 {
            let keys_to_remove: Vec<String> = context
                .variables
                .keys()
                .filter(|k| k.starts_with("__vehicle_"))
                .cloned()
                .collect();
            for key in keys_to_remove {
                context.variables.remove(&key);
            }
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVehicleType {
                object_id: context.object_id,
                vehicle_type,
            },
        ));
        debug!("Setting vehicle type to {}", vehicle_type);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_vehicle_flags(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetVehicleFlags expects 1 argument"));
        }
        let flags = args[0].to_integer();
        let current = match context.variables.get("__vehicle_flags") {
            Some(LSLValue::Integer(v)) => *v,
            _ => 0,
        };
        context.variables.insert(
            "__vehicle_flags".to_string(),
            LSLValue::Integer(current | flags),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVehicleFlags {
                object_id: context.object_id,
                flags,
            },
        ));
        debug!("Setting vehicle flags to {}", current | flags);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remove_vehicle_flags(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRemoveVehicleFlags expects 1 argument"));
        }
        let flags = args[0].to_integer();
        let current = match context.variables.get("__vehicle_flags") {
            Some(LSLValue::Integer(v)) => *v,
            _ => 0,
        };
        context.variables.insert(
            "__vehicle_flags".to_string(),
            LSLValue::Integer(current & !flags),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RemoveVehicleFlags {
                object_id: context.object_id,
                flags,
            },
        ));
        debug!("Removed vehicle flags {}", flags);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_vehicle_float_param(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetVehicleFloatParam expects 2 arguments"));
        }
        let param = args[0].to_integer();
        let value = args[1].to_float();
        context
            .variables
            .insert(format!("__vehicle_fp_{}", param), LSLValue::Float(value));
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVehicleFloatParam {
                object_id: context.object_id,
                param_id: param,
                value: value as f64,
            },
        ));
        debug!("Setting vehicle float param {} to {}", param, value);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_vehicle_vector_param(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetVehicleVectorParam expects 2 arguments"));
        }
        let param = args[0].to_integer();
        let value = args[1].to_vector();
        context.variables.insert(
            format!("__vehicle_vp_{}", param),
            LSLValue::Vector(value.clone()),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVehicleVectorParam {
                object_id: context.object_id,
                param_id: param,
                value: [value.x, value.y, value.z],
            },
        ));
        debug!("Setting vehicle vector param {} to {:?}", param, value);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_vehicle_rotation_param(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetVehicleRotationParam expects 2 arguments"));
        }
        let param = args[0].to_integer();
        let value = args[1].to_rotation();
        context.variables.insert(
            format!("__vehicle_rp_{}", param),
            LSLValue::Rotation(value.clone()),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVehicleRotationParam {
                object_id: context.object_id,
                param_id: param,
                value: [value.x, value.y, value.z, value.s],
            },
        ));
        debug!("Setting vehicle rotation param {} to {:?}", param, value);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_region_flags(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        let flags = 0x00000040 // REGION_FLAGS_ALLOW_DAMAGE
            | 0x00000100  // REGION_FLAGS_ALLOW_LANDMARK
            | 0x00000400  // REGION_FLAGS_ALLOW_SET_HOME
            | 0x00001000  // REGION_FLAGS_ALLOW_DIRECT_TELEPORT
            | 0x00010000; // REGION_FLAGS_ALLOW_PARCEL_CHANGES
        Ok(LSLValue::Integer(flags))
    }

    async fn ll_get_region_fps(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(45.0))
    }

    async fn ll_get_region_time_dilation(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(1.0))
    }

    async fn ll_get_region_agent_count(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(1))
    }

    async fn ll_request_simulator_data(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRequestSimulatorData expects 2 arguments"));
        }
        let region = args[0].to_string();
        let data_type = args[1].to_integer();
        let request_id = Uuid::new_v4();
        let is_current = region == context.region_name;
        let result = match data_type {
            5 => {
                if is_current {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            6 => {
                if is_current {
                    "PG".to_string()
                } else {
                    String::new()
                }
            }
            7 => "256".to_string(),
            8 => {
                if is_current {
                    context.region_handle.to_string()
                } else {
                    "0".to_string()
                }
            }
            _ => String::new(),
        };
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: result,
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_get_simulator_hostname(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
        Ok(LSLValue::String(hostname))
    }

    async fn ll_get_env(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetEnv expects 1 argument"));
        }
        let name = args[0].to_string();
        match name.as_str() {
            "agent_limit" => Ok(LSLValue::String("100".to_string())),
            "dynamic_pathfinding" => Ok(LSLValue::String("disabled".to_string())),
            "estate_id" => Ok(LSLValue::String("1".to_string())),
            "estate_name" => Ok(LSLValue::String("Gaia Estate".to_string())),
            "frame_number" => Ok(LSLValue::String("1".to_string())),
            "region_cpu_ratio" => Ok(LSLValue::String("1".to_string())),
            "region_idle" => Ok(LSLValue::String("0".to_string())),
            "region_product_name" => Ok(LSLValue::String("OpenSim Next".to_string())),
            "region_product_sku" => Ok(LSLValue::String("OpenSimNext".to_string())),
            "region_start_time" => Ok(LSLValue::String("0".to_string())),
            "region_max_prims" => Ok(LSLValue::String("15000".to_string())),
            "sim_channel" => Ok(LSLValue::String("OpenSim Next".to_string())),
            "sim_version" => Ok(LSLValue::String("0.9.3".to_string())),
            "simulator_hostname" => Ok(LSLValue::String(
                std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
            )),
            "region_name" => Ok(LSLValue::String(context.region_name.clone())),
            _ => Ok(LSLValue::String(String::new())),
        }
    }

    async fn ll_get_sim_stats(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::List(vec![
            LSLValue::Float(45.0), // SIM_STAT_PCT_CHARS_STEPPED
            LSLValue::Float(1.0),  // time dilation
            LSLValue::Float(45.0), // sim fps
            LSLValue::Float(45.0), // physics fps
            LSLValue::Integer(1),  // agents in region
            LSLValue::Integer(0),  // child agents
            LSLValue::Float(22.0), // total frame time ms
            LSLValue::Float(0.5),  // physics step time ms
        ]))
    }

    async fn ll_ground(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGround expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Float(context.terrain_height))
    }

    async fn ll_ground_normal(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGroundNormal expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 1.0)))
    }

    async fn ll_ground_slope(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGroundSlope expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_ground_contour(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGroundContour expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_water(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llWater expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Float(20.0))
    }

    async fn ll_cloud(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCloud expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Float(0.0))
    }

    async fn ll_wind(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llWind expects 1 argument"));
        }
        let _offset = args[0].to_vector();
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_edge_of_world(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llEdgeOfWorld expects 2 arguments"));
        }
        let _pos = args[0].to_vector();
        let _dir = args[1].to_vector();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_sun_direction(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.938, -0.347)))
    }

    async fn ll_get_sun_rotation(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
    }

    async fn ll_get_moon_direction(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, -0.938, 0.347)))
    }

    async fn ll_get_moon_rotation(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
    }

    async fn ll_get_region_sun_direction(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.938, -0.347)))
    }

    async fn ll_get_region_sun_rotation(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
    }

    async fn ll_get_region_moon_direction(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, -0.938, 0.347)))
    }

    async fn ll_get_region_moon_rotation(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
    }

    async fn ll_get_day_length(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(14400))
    }

    async fn ll_get_day_offset(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_region_day_length(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(14400))
    }

    async fn ll_get_region_day_offset(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rez_object(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llRezObject expects 5 arguments"));
        }
        let inventory = args[0].to_string();
        let pos = args[1].to_vector();
        let vel = args[2].to_vector();
        let rot = args[3].to_rotation();
        let param = args[4].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RezObject {
                prim_id: context.object_id,
                item_name: inventory,
                position: [pos.x as f32, pos.y as f32, pos.z as f32],
                velocity: [vel.x as f32, vel.y as f32, vel.z as f32],
                rotation: [rot.x as f32, rot.y as f32, rot.z as f32, rot.s as f32],
                start_param: param,
                at_root: false,
                owner_id: context.owner_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rez_at_root(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llRezAtRoot expects 5 arguments"));
        }
        let inventory = args[0].to_string();
        let pos = args[1].to_vector();
        let vel = args[2].to_vector();
        let rot = args[3].to_rotation();
        let param = args[4].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RezObject {
                prim_id: context.object_id,
                item_name: inventory,
                position: [pos.x as f32, pos.y as f32, pos.z as f32],
                velocity: [vel.x as f32, vel.y as f32, vel.z as f32],
                rotation: [rot.x as f32, rot.y as f32, rot.z as f32, rot.s as f32],
                start_param: param,
                at_root: true,
                owner_id: context.owner_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rez_object_with_params(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRezObjectWithParams expects 2 arguments"));
        }
        let inventory = args[0].to_string();
        let params = args[1].to_list();
        debug!(
            "Rezzing object with params {}: {} params",
            inventory,
            params.len()
        );
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    async fn ll_die(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::Die {
                object_id: context.object_id,
            },
        ));
        debug!("Object {} dying", context.object_id);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_derez_object(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llDerezObject expects 2 arguments"));
        }
        let id = args[0].to_key();
        let destination = args[1].to_integer();
        debug!("Derezzing object {} to destination {}", id, destination);
        Ok(LSLValue::Integer(1))
    }

    async fn ll_get_creator(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Key(context.owner_id))
    }

    async fn ll_get_owner_key(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetOwnerKey expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Key(Uuid::nil()))
    }

    async fn ll_get_bounding_box(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetBoundingBox expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::List(vec![
            LSLValue::Vector(LSLVector::new(-0.5, -0.5, -0.5)),
            LSLValue::Vector(LSLVector::new(0.5, 0.5, 0.5)),
        ]))
    }

    async fn ll_get_geometric_center(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_get_center_of_mass(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)))
    }

    async fn ll_get_object_details(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetObjectDetails expects 2 arguments"));
        }
        let id = args[0].to_key();
        let details = args[1].to_list();
        let is_self = id == context.object_id;
        let mut result = Vec::new();
        for detail in details {
            match detail.to_integer() {
                1 => result.push(LSLValue::String(if is_self {
                    context.object_name.clone()
                } else {
                    String::new()
                })),
                2 => result.push(LSLValue::String(if is_self {
                    context.object_description.clone()
                } else {
                    String::new()
                })),
                3 => result.push(LSLValue::Vector(if is_self {
                    LSLVector::new(context.position.0, context.position.1, context.position.2)
                } else {
                    LSLVector::zero()
                })),
                4 => result.push(LSLValue::Rotation(if is_self {
                    LSLRotation::new(
                        context.rotation.0,
                        context.rotation.1,
                        context.rotation.2,
                        context.rotation.3,
                    )
                } else {
                    LSLRotation::identity()
                })),
                5 => result.push(LSLValue::Vector(if is_self {
                    LSLVector::new(context.velocity.0, context.velocity.1, context.velocity.2)
                } else {
                    LSLVector::zero()
                })),
                6 => result.push(LSLValue::Key(if is_self {
                    context.owner_id
                } else {
                    Uuid::nil()
                })),
                7 => result.push(LSLValue::Key(Uuid::nil())),
                8 => result.push(LSLValue::Key(context.owner_id)),
                9 => result.push(LSLValue::Integer(if is_self { 0x01 } else { 0 })),
                10 => result.push(LSLValue::Integer(if is_self {
                    context.link_number.max(1)
                } else {
                    0
                })),
                11 => result.push(LSLValue::String(if is_self {
                    context.script_name.clone()
                } else {
                    String::new()
                })),
                12 => result.push(LSLValue::Vector(LSLVector::zero())),
                13 => result.push(LSLValue::Vector(LSLVector::zero())),
                14 => result.push(LSLValue::Float(if is_self {
                    context.script_start_time.elapsed().as_secs_f32()
                } else {
                    0.0
                })),
                15 => result.push(LSLValue::Float(1.0)),
                16 => result.push(LSLValue::Float(1.0)),
                17 => result.push(LSLValue::String(context.region_name.clone())),
                18 => result.push(LSLValue::Vector(LSLVector::zero())),
                19 => result.push(LSLValue::Vector(if is_self {
                    LSLVector::new(context.scale.0, context.scale.1, context.scale.2)
                } else {
                    LSLVector::new(1.0, 1.0, 1.0)
                })),
                20 => result.push(LSLValue::Integer(0)),
                21 => result.push(LSLValue::Integer(0)),
                22 => result.push(LSLValue::Float(0.0)),
                23 => result.push(LSLValue::Float(0.0)),
                24 => result.push(LSLValue::Float(0.0)),
                25 => result.push(LSLValue::Float(0.0)),
                26 => result.push(LSLValue::Float(0.0)),
                27 => result.push(LSLValue::Integer(0)),
                28 => result.push(LSLValue::String(String::new())),
                29 => result.push(LSLValue::Key(Uuid::nil())),
                30 => result.push(LSLValue::Integer(1)),
                31 => result.push(LSLValue::Float(0.0)),
                32 => result.push(LSLValue::Float(0.0)),
                33 => result.push(LSLValue::Integer(0)),
                _ => result.push(LSLValue::String(String::new())),
            }
        }
        Ok(LSLValue::List(result))
    }

    async fn ll_get_status(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetStatus expects 1 argument"));
        }
        let status = args[0].to_integer();
        let flags = context.flags;
        let result = match status {
            1 => {
                if flags & 0x01 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_PHYSICS
            2 => {
                if flags & 0x02 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_ROTATE_X
            4 => {
                if flags & 0x04 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_ROTATE_Y
            8 => {
                if flags & 0x08 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_ROTATE_Z
            16 => {
                if flags & 0x20 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_PHANTOM (flag bit 0x20)
            32 => {
                if flags & 0x40 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_SANDBOX
            64 => {
                if flags & 0x400 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_BLOCK_GRAB
            128 => {
                if flags & 0x80 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_DIE_AT_EDGE
            256 => {
                if flags & 0x100 != 0 {
                    1
                } else {
                    0
                }
            } // STATUS_RETURN_AT_EDGE
            _ => 0,
        };
        Ok(LSLValue::Integer(result))
    }

    async fn ll_set_status(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetStatus expects 2 arguments"));
        }
        let status = args[0].to_integer();
        let value = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetStatus {
                object_id: context.object_id,
                status,
                value,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_damage(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetDamage expects 1 argument"));
        }
        let damage = args[0].to_float() as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetDamage {
                object_id: context.object_id,
                damage,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_allow_inventory_drop(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llAllowInventoryDrop expects 1 argument"));
        }
        let allow = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetAllowInventoryDrop {
                object_id: context.object_id,
                allow,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_pass_touches(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llPassTouches expects 1 argument"));
        }
        let pass = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPassTouches {
                object_id: context.object_id,
                pass,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_pass_collisions(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llPassCollisions expects 1 argument"));
        }
        let pass = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPassCollisions {
                object_id: context.object_id,
                pass,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_collision_filter(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llCollisionFilter expects 3 arguments"));
        }
        let name = args[0].to_string();
        let id = args[1].to_key();
        let accept = args[2].is_true();
        debug!(
            "Collision filter: name={}, id={}, accept={}",
            name, id, accept
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_volume_detect(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llVolumeDetect expects 1 argument"));
        }
        let detect = args[0].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetVolumeDetect {
                object_id: context.object_id,
                detect,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_target(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llTarget expects 2 arguments"));
        }
        let position = args[0].to_vector();
        let range = args[1].to_float().max(0.0);
        let handle = match context.variables.get("__target_next_handle") {
            Some(LSLValue::Integer(h)) => *h,
            _ => 1,
        };
        context.variables.insert(
            "__target_next_handle".to_string(),
            LSLValue::Integer(handle + 1),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AddPosTarget {
                object_id: context.object_id,
                handle,
                position: [position.x, position.y, position.z],
                range: range as f32,
            },
        ));
        debug!(
            "Setting target {} at {:?}, range={}",
            handle, position, range
        );
        Ok(LSLValue::Integer(handle))
    }

    async fn ll_target_remove(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llTargetRemove expects 1 argument"));
        }
        let handle = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RemovePosTarget {
                object_id: context.object_id,
                handle,
            },
        ));
        debug!("Removing target {}", handle);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rot_target(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRotTarget expects 2 arguments"));
        }
        let rotation = args[0].to_rotation();
        let error = args[1].to_float().max(0.0);
        let handle = match context.variables.get("__rot_target_next_handle") {
            Some(LSLValue::Integer(h)) => *h,
            _ => 1,
        };
        context.variables.insert(
            "__rot_target_next_handle".to_string(),
            LSLValue::Integer(handle + 1),
        );
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::AddRotTarget {
                object_id: context.object_id,
                handle,
                rotation: [rotation.x, rotation.y, rotation.z, rotation.s],
                error: error as f32,
            },
        ));
        debug!(
            "Setting rotation target {} {:?}, error={}",
            handle, rotation, error
        );
        Ok(LSLValue::Integer(handle))
    }

    async fn ll_rot_target_remove(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRotTargetRemove expects 1 argument"));
        }
        let handle = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RemoveRotTarget {
                object_id: context.object_id,
                handle,
            },
        ));
        debug!("Removing rotation target {}", handle);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_look_at(&self, args: &[LSLValue], context: &mut ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llLookAt expects 3 arguments"));
        }
        let target = args[0].to_vector();
        let strength = args[1].to_float().max(0.0);
        let damping = args[2].to_float().max(0.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::LookAt {
                object_id: context.object_id,
                target: [target.x, target.y, target.z],
                strength,
                damping,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_look_at(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::StopLookAt {
                object_id: context.object_id,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_rot_look_at(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llRotLookAt expects 3 arguments"));
        }
        let target = args[0].to_rotation();
        let strength = args[1].to_float().max(0.0);
        let damping = args[2].to_float().max(0.0);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RotLookAt {
                object_id: context.object_id,
                rotation: [target.x, target.y, target.z, target.s],
                strength,
                damping,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_point_at(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    async fn ll_stop_point_at(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    fn pack_fixed_u16(val: f32, int_bits: u32, frac_bits: u32) -> u16 {
        let max = ((1u32 << (int_bits + frac_bits)) - 1) as f32;
        let scaled = (val * (1u32 << frac_bits) as f32).clamp(0.0, max);
        scaled as u16
    }

    fn pack_fixed_s16(val: f32, int_bits: u32, frac_bits: u32) -> u16 {
        let total = int_bits + frac_bits + 1;
        let max = ((1u32 << (total - 1)) - 1) as f32;
        let min = -((1u32 << (total - 1)) as f32);
        let scaled = (val * (1u32 << frac_bits) as f32).clamp(min, max);
        (scaled as i16) as u16
    }

    fn pack_fixed_u8(val: f32, int_bits: u32, frac_bits: u32) -> u8 {
        let max = ((1u32 << (int_bits + frac_bits)) - 1) as f32;
        let scaled = (val * (1u32 << frac_bits) as f32).clamp(0.0, max);
        scaled as u8
    }

    fn encode_particle_system(&self, rules: &[LSLValue]) -> Vec<u8> {
        if rules.is_empty() {
            return Vec::new();
        }

        let mut part_data_flags: u32 = 0;
        let mut pattern: u8 = 0x02;
        let mut max_age: f32 = 10.0;
        let mut start_age: f32 = 0.0;
        let mut inner_angle: f32 = 0.0;
        let mut outer_angle: f32 = 0.0;
        let mut burst_rate: f32 = 0.1;
        let mut burst_radius: f32 = 0.0;
        let mut burst_speed_min: f32 = 1.0;
        let mut burst_speed_max: f32 = 1.0;
        let mut burst_part_count: u8 = 1;
        let mut ang_vel = [0.0f32; 3];
        let mut accel = [0.0f32; 3];
        let mut texture = Uuid::nil();
        let mut target = Uuid::nil();
        let mut part_max_age: f32 = 10.0;
        let mut start_color = [1.0f32; 4];
        let mut end_color = [1.0f32, 1.0, 1.0, 0.0];
        let mut start_scale = [1.0f32; 2];
        let mut end_scale = [1.0f32; 2];
        let mut start_glow: f32 = 0.0;
        let mut end_glow: f32 = 0.0;
        let mut blend_source: u8 = 7;
        let mut blend_dest: u8 = 9;
        let mut has_glow = false;
        let mut has_blend = false;

        let mut i = 0;
        while i + 1 < rules.len() {
            let code = rules[i].to_integer();
            i += 1;
            match code {
                0 => {
                    part_data_flags = rules[i].to_integer() as u32;
                    i += 1;
                }
                1 => {
                    let v = rules[i].to_vector();
                    start_color = [v.x, v.y, v.z, start_color[3]];
                    i += 1;
                }
                2 => {
                    start_color[3] = rules[i].to_float();
                    i += 1;
                }
                3 => {
                    let v = rules[i].to_vector();
                    end_color = [v.x, v.y, v.z, end_color[3]];
                    i += 1;
                }
                4 => {
                    end_color[3] = rules[i].to_float();
                    i += 1;
                }
                5 => {
                    let v = rules[i].to_vector();
                    start_scale = [v.x, v.y];
                    i += 1;
                }
                6 => {
                    let v = rules[i].to_vector();
                    end_scale = [v.x, v.y];
                    i += 1;
                }
                7 => {
                    part_max_age = rules[i].to_float();
                    i += 1;
                }
                8 => {
                    let v = rules[i].to_vector();
                    accel = [v.x, v.y, v.z];
                    i += 1;
                }
                9 => {
                    pattern = rules[i].to_integer() as u8;
                    i += 1;
                }
                12 => {
                    let s = rules[i].to_string();
                    texture = Uuid::parse_str(&s).unwrap_or(Uuid::nil());
                    i += 1;
                }
                13 => {
                    burst_rate = rules[i].to_float();
                    i += 1;
                }
                15 => {
                    burst_part_count = rules[i].to_integer().clamp(0, 255) as u8;
                    i += 1;
                }
                16 => {
                    burst_radius = rules[i].to_float();
                    i += 1;
                }
                17 => {
                    burst_speed_min = rules[i].to_float();
                    i += 1;
                }
                18 => {
                    burst_speed_max = rules[i].to_float();
                    i += 1;
                }
                19 => {
                    max_age = rules[i].to_float();
                    i += 1;
                }
                20 => {
                    target = rules[i].to_key();
                    i += 1;
                }
                21 => {
                    let v = rules[i].to_vector();
                    ang_vel = [v.x, v.y, v.z];
                    i += 1;
                }
                22 => {
                    inner_angle = rules[i].to_float();
                    i += 1;
                }
                23 => {
                    outer_angle = rules[i].to_float();
                    i += 1;
                }
                24 => {
                    blend_source = rules[i].to_integer() as u8;
                    has_blend = true;
                    i += 1;
                }
                25 => {
                    blend_dest = rules[i].to_integer() as u8;
                    has_blend = true;
                    i += 1;
                }
                26 => {
                    start_glow = rules[i].to_float();
                    has_glow = true;
                    i += 1;
                }
                27 => {
                    end_glow = rules[i].to_float();
                    has_glow = true;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        if has_glow {
            part_data_flags |= 0x10000;
        }
        if has_blend {
            part_data_flags |= 0x20000;
        }

        let part_data_size: u32 = 18 + if has_glow { 2 } else { 0 } + if has_blend { 2 } else { 0 };
        let mut buf = Vec::with_capacity(92);

        buf.extend_from_slice(&68u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.push(pattern);
        buf.extend_from_slice(&Self::pack_fixed_u16(max_age, 8, 8).to_le_bytes());
        buf.extend_from_slice(&Self::pack_fixed_u16(start_age, 8, 8).to_le_bytes());
        buf.push(Self::pack_fixed_u8(inner_angle, 3, 5));
        buf.push(Self::pack_fixed_u8(outer_angle, 3, 5));
        buf.extend_from_slice(&Self::pack_fixed_u16(burst_rate, 8, 8).to_le_bytes());
        buf.extend_from_slice(&Self::pack_fixed_u16(burst_radius, 8, 8).to_le_bytes());
        buf.extend_from_slice(&Self::pack_fixed_u16(burst_speed_min, 8, 8).to_le_bytes());
        buf.extend_from_slice(&Self::pack_fixed_u16(burst_speed_max, 8, 8).to_le_bytes());
        buf.push(burst_part_count);

        for v in &ang_vel {
            buf.extend_from_slice(&Self::pack_fixed_s16(*v, 8, 7).to_le_bytes());
        }
        for v in &accel {
            buf.extend_from_slice(&Self::pack_fixed_s16(*v, 8, 7).to_le_bytes());
        }

        buf.extend_from_slice(texture.as_bytes());
        buf.extend_from_slice(target.as_bytes());

        buf.extend_from_slice(&part_data_size.to_le_bytes());
        buf.extend_from_slice(&part_data_flags.to_le_bytes());
        buf.extend_from_slice(&Self::pack_fixed_u16(part_max_age, 8, 8).to_le_bytes());

        buf.push((start_color[0] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((start_color[1] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((start_color[2] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((start_color[3] * 255.0).clamp(0.0, 255.0) as u8);

        buf.push((end_color[0] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((end_color[1] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((end_color[2] * 255.0).clamp(0.0, 255.0) as u8);
        buf.push((end_color[3] * 255.0).clamp(0.0, 255.0) as u8);

        buf.push(Self::pack_fixed_u8(start_scale[0], 3, 5));
        buf.push(Self::pack_fixed_u8(start_scale[1], 3, 5));
        buf.push(Self::pack_fixed_u8(end_scale[0], 3, 5));
        buf.push(Self::pack_fixed_u8(end_scale[1], 3, 5));

        if has_glow {
            buf.push((start_glow * 255.0).clamp(0.0, 255.0) as u8);
            buf.push((end_glow * 255.0).clamp(0.0, 255.0) as u8);
        }
        if has_blend {
            buf.push(blend_source);
            buf.push(blend_dest);
        }

        buf
    }

    async fn ll_particle_system(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llParticleSystem expects 1 argument"));
        }
        let rules = args[0].to_list();
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_link_particle_system(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinkParticleSystem expects 2 arguments"));
        }
        let _link = args[0].to_integer();
        let rules = args[1].to_list();
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_make_explosion(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 5 {
            return Err(anyhow!("llMakeExplosion expects at least 5 arguments"));
        }
        let particles = args[0].to_integer();
        let scale = args[1].to_float();
        let rules = vec![
            LSLValue::Integer(0),
            LSLValue::Integer(0x02 | 0x20),
            LSLValue::Integer(1),
            LSLValue::Integer(particles.min(100)),
            LSLValue::Integer(2),
            LSLValue::Float(1.0),
            LSLValue::Integer(3),
            LSLValue::Float(0.5),
            LSLValue::Integer(5),
            LSLValue::Float(scale),
            LSLValue::Integer(7),
            LSLValue::Float(2.0),
        ];
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_make_fire(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 5 {
            return Err(anyhow!("llMakeFire expects at least 5 arguments"));
        }
        let particles = args[0].to_integer();
        let scale = args[1].to_float();
        let rules = vec![
            LSLValue::Integer(0),
            LSLValue::Integer(0x02 | 0x10 | 0x100),
            LSLValue::Integer(1),
            LSLValue::Integer(particles.min(100)),
            LSLValue::Integer(2),
            LSLValue::Float(3.0),
            LSLValue::Integer(3),
            LSLValue::Float(0.5),
            LSLValue::Integer(5),
            LSLValue::Float(scale),
            LSLValue::Integer(7),
            LSLValue::Float(0.5),
        ];
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_make_fountain(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 5 {
            return Err(anyhow!("llMakeFountain expects at least 5 arguments"));
        }
        let particles = args[0].to_integer();
        let scale = args[1].to_float();
        let rules = vec![
            LSLValue::Integer(0),
            LSLValue::Integer(0x02 | 0x80),
            LSLValue::Integer(1),
            LSLValue::Integer(particles.min(100)),
            LSLValue::Integer(2),
            LSLValue::Float(5.0),
            LSLValue::Integer(3),
            LSLValue::Float(1.0),
            LSLValue::Integer(5),
            LSLValue::Float(scale),
            LSLValue::Integer(7),
            LSLValue::Float(0.5),
        ];
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_make_smoke(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 5 {
            return Err(anyhow!("llMakeSmoke expects at least 5 arguments"));
        }
        let particles = args[0].to_integer();
        let scale = args[1].to_float();
        let rules = vec![
            LSLValue::Integer(0),
            LSLValue::Integer(0x02 | 0x10 | 0x100),
            LSLValue::Integer(1),
            LSLValue::Integer(particles.min(100)),
            LSLValue::Integer(2),
            LSLValue::Float(8.0),
            LSLValue::Integer(3),
            LSLValue::Float(2.0),
            LSLValue::Integer(5),
            LSLValue::Float(scale),
            LSLValue::Integer(7),
            LSLValue::Float(0.2),
        ];
        let ps_bytes = self.encode_particle_system(&rules);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParticleSystem {
                object_id: context.object_id,
                rules: ps_bytes,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_email(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llEmail expects 3 arguments"));
        }
        let address = args[0].to_string();
        let subject = args[1].to_string();
        let message = args[2].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::Email {
                object_id: context.object_id,
                address,
                subject,
                message,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_targeted_email(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llTargetedEmail expects 4 arguments"));
        }
        let _target_type = args[0].to_integer();
        let address = args[1].to_string();
        let subject = args[2].to_string();
        let message = args[3].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::Email {
                object_id: context.object_id,
                address,
                subject,
                message,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_next_email(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetNextEmail expects 2 arguments"));
        }
        let _address = args[0].to_string();
        let _subject = args[1].to_string();
        debug!("Getting next email");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_request_url(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RequestURL {
                script_id: context.script_id,
                object_id: context.object_id,
                request_id,
                secure: false,
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_request_secure_url(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::RequestURL {
                script_id: context.script_id,
                object_id: context.object_id,
                request_id,
                secure: true,
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_release_url(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llReleaseURL expects 1 argument"));
        }
        let url = args[0].to_string();
        self.action_queue
            .lock()
            .push((context.script_id, ScriptAction::ReleaseURL { url }));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_free_urls(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(10))
    }

    async fn ll_get_http_header(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetHTTPHeader expects 2 arguments"));
        }
        let _request_id = args[0].to_key();
        let _header = args[1].to_string();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_set_content_type(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetContentType expects 2 arguments"));
        }
        let _request_id = args[0].to_key();
        let content_type = args[1].to_integer();
        debug!("Setting content type: {}", content_type);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_open_remote_data_channel(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        debug!("Opening remote data channel");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_close_remote_data_channel(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCloseRemoteDataChannel expects 1 argument"));
        }
        let channel = args[0].to_key();
        debug!("Closing remote data channel: {}", channel);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_send_remote_data(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llSendRemoteData expects 4 arguments"));
        }
        let channel = args[0].to_key();
        let dest = args[1].to_string();
        let idata = args[2].to_integer();
        let sdata = args[3].to_string();
        debug!(
            "Sending remote data on channel {}: dest={}, idata={}, sdata={}",
            channel, dest, idata, sdata
        );
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    async fn ll_remote_data_reply(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llRemoteDataReply expects 4 arguments"));
        }
        let channel = args[0].to_key();
        let message_id = args[1].to_key();
        let sdata = args[2].to_string();
        let idata = args[3].to_integer();
        debug!(
            "Remote data reply on channel {}: msg={}, sdata={}, idata={}",
            channel, message_id, sdata, idata
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remote_data_set_region(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        debug!("Remote data set region (deprecated)");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_json2list(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llJson2List expects 1 argument"));
        }
        let json = args[0].to_string();
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) {
            let list = match parsed {
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                LSLValue::Integer(i as i32)
                            } else if let Some(f) = n.as_f64() {
                                LSLValue::Float(f as f32)
                            } else {
                                LSLValue::String(n.to_string())
                            }
                        }
                        serde_json::Value::String(s) => LSLValue::String(s.clone()),
                        serde_json::Value::Bool(b) => LSLValue::Integer(if *b { 1 } else { 0 }),
                        _ => LSLValue::String(v.to_string()),
                    })
                    .collect(),
                serde_json::Value::Object(obj) => {
                    let mut result = Vec::new();
                    for (k, v) in obj {
                        result.push(LSLValue::String(k));
                        result.push(LSLValue::String(v.to_string()));
                    }
                    result
                }
                _ => vec![LSLValue::String(json)],
            };
            Ok(LSLValue::List(list))
        } else {
            Ok(LSLValue::List(vec![LSLValue::String(json)]))
        }
    }

    async fn ll_list2json(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llList2Json expects 2 arguments"));
        }
        let json_type = args[0].to_string();
        let values = args[1].to_list();

        let json = if json_type == "JSON_ARRAY" {
            let arr: Vec<serde_json::Value> = values
                .iter()
                .map(|v| match v {
                    LSLValue::Integer(i) => serde_json::Value::Number((*i).into()),
                    LSLValue::Float(f) => serde_json::Number::from_f64(*f as f64)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    LSLValue::String(s) => serde_json::Value::String(s.clone()),
                    _ => serde_json::Value::String(v.to_string()),
                })
                .collect();
            serde_json::to_string(&arr).unwrap_or_default()
        } else {
            let mut obj = serde_json::Map::new();
            let mut iter = values.iter();
            while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                obj.insert(k.to_string(), serde_json::Value::String(v.to_string()));
            }
            serde_json::to_string(&obj).unwrap_or_default()
        };

        Ok(LSLValue::String(json))
    }

    async fn ll_json_get_value(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llJsonGetValue expects 2 arguments"));
        }
        let json = args[0].to_string();
        let specifiers = args[1].to_list();

        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) {
            for spec in specifiers {
                match spec {
                    LSLValue::Integer(i) => {
                        if let Some(arr) = value.as_array() {
                            if let Some(v) = arr.get(i as usize) {
                                value = v.clone();
                            } else {
                                return Ok(LSLValue::String("JSON_INVALID".to_string()));
                            }
                        }
                    }
                    LSLValue::String(s) => {
                        if let Some(obj) = value.as_object() {
                            if let Some(v) = obj.get(&s) {
                                value = v.clone();
                            } else {
                                return Ok(LSLValue::String("JSON_INVALID".to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(LSLValue::String(match value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => if b { "true" } else { "false" }.to_string(),
                serde_json::Value::Null => "null".to_string(),
                _ => value.to_string(),
            }))
        } else {
            Ok(LSLValue::String("JSON_INVALID".to_string()))
        }
    }

    async fn ll_json_set_value(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llJsonSetValue expects 3 arguments"));
        }
        let json = args[0].to_string();
        let specifiers = args[1].to_list();
        let new_value = args[2].to_string();

        let mut root = match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) => v,
            Err(_) => return Ok(LSLValue::String("JSON_INVALID".to_string())),
        };

        let json_val = if new_value == "JSON_DELETE" {
            None
        } else {
            Some(
                match serde_json::from_str::<serde_json::Value>(&new_value) {
                    Ok(v) => v,
                    Err(_) => serde_json::Value::String(new_value),
                },
            )
        };

        if specifiers.is_empty() {
            return match json_val {
                Some(v) => Ok(LSLValue::String(
                    serde_json::to_string(&v).unwrap_or_default(),
                )),
                None => Ok(LSLValue::String("JSON_INVALID".to_string())),
            };
        }

        let mut current = &mut root;
        for (idx, spec) in specifiers.iter().enumerate() {
            let is_last = idx == specifiers.len() - 1;
            match spec {
                LSLValue::Integer(i) => {
                    let i = *i as usize;
                    if is_last {
                        if let Some(arr) = current.as_array_mut() {
                            match &json_val {
                                Some(v) => {
                                    while arr.len() <= i {
                                        arr.push(serde_json::Value::Null);
                                    }
                                    arr[i] = v.clone();
                                }
                                None => {
                                    if i < arr.len() {
                                        arr.remove(i);
                                    }
                                }
                            }
                        }
                        break;
                    }
                    if let Some(arr) = current.as_array_mut() {
                        while arr.len() <= i {
                            arr.push(serde_json::Value::Null);
                        }
                        current = &mut arr[i];
                    } else {
                        return Ok(LSLValue::String("JSON_INVALID".to_string()));
                    }
                }
                LSLValue::String(s) => {
                    if is_last {
                        if let Some(obj) = current.as_object_mut() {
                            match &json_val {
                                Some(v) => {
                                    obj.insert(s.clone(), v.clone());
                                }
                                None => {
                                    obj.remove(s);
                                }
                            }
                        }
                        break;
                    }
                    if !current.is_object() {
                        return Ok(LSLValue::String("JSON_INVALID".to_string()));
                    }
                    if !current.as_object().unwrap().contains_key(s) {
                        current
                            .as_object_mut()
                            .unwrap()
                            .insert(s.clone(), serde_json::Value::Object(Default::default()));
                    }
                    current = current.as_object_mut().unwrap().get_mut(s).unwrap();
                }
                _ => {}
            }
        }

        Ok(LSLValue::String(
            serde_json::to_string(&root).unwrap_or_default(),
        ))
    }

    async fn ll_json_value_type(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llJsonValueType expects 2 arguments"));
        }
        let json = args[0].to_string();
        let specifiers = args[1].to_list();

        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) {
            for spec in &specifiers {
                match spec {
                    LSLValue::Integer(i) => {
                        if let Some(arr) = value.as_array() {
                            if let Some(v) = arr.get(*i as usize) {
                                value = v.clone();
                            } else {
                                return Ok(LSLValue::String("JSON_INVALID".to_string()));
                            }
                        } else {
                            return Ok(LSLValue::String("JSON_INVALID".to_string()));
                        }
                    }
                    LSLValue::String(s) => {
                        if let Some(obj) = value.as_object() {
                            if let Some(v) = obj.get(s) {
                                value = v.clone();
                            } else {
                                return Ok(LSLValue::String("JSON_INVALID".to_string()));
                            }
                        } else {
                            return Ok(LSLValue::String("JSON_INVALID".to_string()));
                        }
                    }
                    _ => {}
                }
            }
            let type_str = match value {
                serde_json::Value::Object(_) => "JSON_OBJECT",
                serde_json::Value::Array(_) => "JSON_ARRAY",
                serde_json::Value::Number(_) => "JSON_NUMBER",
                serde_json::Value::String(_) => "JSON_STRING",
                serde_json::Value::Bool(true) => "JSON_TRUE",
                serde_json::Value::Bool(false) => "JSON_FALSE",
                serde_json::Value::Null => "JSON_NULL",
            };
            Ok(LSLValue::String(type_str.to_string()))
        } else {
            Ok(LSLValue::String("JSON_INVALID".to_string()))
        }
    }

    fn linkset_data_size(data: &std::collections::HashMap<String, String>) -> usize {
        data.iter().map(|(k, v)| k.len() + v.len()).sum()
    }

    async fn ll_linkset_data_write(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataWrite expects 2 arguments"));
        }
        let key = args[0].to_string();
        let value = args[1].to_string();
        if key.is_empty() {
            return Ok(LSLValue::Integer(3)); // DATA_INVALID_KEY
        }
        let new_size = key.len() + value.len();
        let current_size = Self::linkset_data_size(&context.linkset_data);
        let existing_size = context
            .linkset_data
            .get(&key)
            .map(|v| key.len() + v.len())
            .unwrap_or(0);
        if current_size - existing_size + new_size > 131072 {
            // 128KB
            return Ok(LSLValue::Integer(4)); // DATA_TOO_LARGE
        }
        context.linkset_data.insert(key, value);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_linkset_data_write_protected(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llLinksetDataWriteProtected expects 3 arguments"));
        }
        let key = args[0].to_string();
        let value = args[1].to_string();
        let pass = args[2].to_string();
        if key.is_empty() {
            return Ok(LSLValue::Integer(3));
        }
        let pass_key = format!("__linkset_pass_{}", key);
        if let Some(stored_pass) = context.linkset_data.get(&pass_key) {
            if *stored_pass != pass {
                return Ok(LSLValue::Integer(5)); // DATA_ACCESS_DENIED
            }
        }
        let new_size = key.len() + value.len();
        let current_size = Self::linkset_data_size(&context.linkset_data);
        let existing_size = context
            .linkset_data
            .get(&key)
            .map(|v| key.len() + v.len())
            .unwrap_or(0);
        if current_size - existing_size + new_size > 131072 {
            return Ok(LSLValue::Integer(4));
        }
        context.linkset_data.insert(key.clone(), value);
        if !pass.is_empty() {
            context.linkset_data.insert(pass_key, pass);
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_linkset_data_read(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLinksetDataRead expects 1 argument"));
        }
        let key = args[0].to_string();
        let value = context.linkset_data.get(&key).cloned().unwrap_or_default();
        Ok(LSLValue::String(value))
    }

    async fn ll_linkset_data_read_protected(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataReadProtected expects 2 arguments"));
        }
        let key = args[0].to_string();
        let _pass = args[1].to_string();
        let value = context.linkset_data.get(&key).cloned().unwrap_or_default();
        Ok(LSLValue::String(value))
    }

    async fn ll_linkset_data_delete(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLinksetDataDelete expects 1 argument"));
        }
        let key = args[0].to_string();
        let removed = context.linkset_data.remove(&key).is_some();
        Ok(LSLValue::Integer(if removed { 0 } else { 1 }))
    }

    async fn ll_linkset_data_delete_protected(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataDeleteProtected expects 2 arguments"));
        }
        let key = args[0].to_string();
        let _pass = args[1].to_string();
        let removed = context.linkset_data.remove(&key).is_some();
        Ok(LSLValue::Integer(if removed { 0 } else { 1 }))
    }

    async fn ll_linkset_data_delete_found(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataDeleteFound expects 2 arguments"));
        }
        let pattern = args[0].to_string();
        let _pass = args[1].to_string();
        let keys_to_delete: Vec<String> = context
            .linkset_data
            .keys()
            .filter(|k| k.contains(&pattern))
            .cloned()
            .collect();
        let count = keys_to_delete.len();
        for key in keys_to_delete {
            context.linkset_data.remove(&key);
        }
        Ok(LSLValue::Integer(count as i32))
    }

    async fn ll_linkset_data_count_keys(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(context.linkset_data.len() as i32))
    }

    async fn ll_linkset_data_count_found(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataCountFound expects 2 arguments"));
        }
        let pattern = args[0].to_string();
        let _pass = args[1].to_string();
        let count = context
            .linkset_data
            .keys()
            .filter(|k| k.contains(&pattern))
            .count();
        Ok(LSLValue::Integer(count as i32))
    }

    async fn ll_linkset_data_find_keys(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llLinksetDataFindKeys expects 4 arguments"));
        }
        let pattern = args[0].to_string();
        let start = args[1].to_integer() as usize;
        let count = args[2].to_integer() as usize;
        let _pass = args[3].to_string();

        let keys: Vec<LSLValue> = context
            .linkset_data
            .keys()
            .filter(|k| k.contains(&pattern))
            .skip(start)
            .take(count)
            .map(|k| LSLValue::String(k.clone()))
            .collect();
        Ok(LSLValue::List(keys))
    }

    async fn ll_linkset_data_list_keys(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llLinksetDataListKeys expects 2 arguments"));
        }
        let start = args[0].to_integer() as usize;
        let count = args[1].to_integer() as usize;

        let keys: Vec<LSLValue> = context
            .linkset_data
            .keys()
            .skip(start)
            .take(count)
            .map(|k| LSLValue::String(k.clone()))
            .collect();
        Ok(LSLValue::List(keys))
    }

    async fn ll_linkset_data_available(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(65536))
    }

    async fn ll_linkset_data_reset(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        context.linkset_data.clear();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_md5_string(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llMD5String expects 2 arguments"));
        }
        let src = args[0].to_string();
        let nonce = args[1].to_integer();
        let input = format!("{}:{}", src, nonce);
        let digest = md5::compute(input.as_bytes());
        Ok(LSLValue::String(format!("{:x}", digest)))
    }

    async fn ll_sha1_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSHA1String expects 1 argument"));
        }
        let src = args[0].to_string();
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(src.as_bytes());
        let result = hasher.finalize();
        Ok(LSLValue::String(format!("{:x}", result)))
    }

    async fn ll_sha256_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSHA256String expects 1 argument"));
        }
        let src = args[0].to_string();
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(src.as_bytes());
        let result = hasher.finalize();
        Ok(LSLValue::String(format!("{:x}", result)))
    }

    async fn ll_hmac(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llHMAC expects 3 arguments"));
        }
        let data = args[0].to_string();
        let key = args[1].to_string();
        let algorithm = args[2].to_string();
        use hmac::{Hmac, Mac};
        match algorithm.as_str() {
            "SHA256" | "sha256" => {
                type HmacSha256 = Hmac<sha2::Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                    .map_err(|e| anyhow!("HMAC key error: {}", e))?;
                mac.update(data.as_bytes());
                let result = mac.finalize();
                Ok(LSLValue::String(hex::encode(result.into_bytes())))
            }
            "SHA1" | "sha1" => {
                type HmacSha1 = Hmac<sha1::Sha1>;
                let mut mac = HmacSha1::new_from_slice(key.as_bytes())
                    .map_err(|e| anyhow!("HMAC key error: {}", e))?;
                mac.update(data.as_bytes());
                let result = mac.finalize();
                Ok(LSLValue::String(hex::encode(result.into_bytes())))
            }
            _ => {
                type HmacSha256 = Hmac<sha2::Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                    .map_err(|e| anyhow!("HMAC key error: {}", e))?;
                mac.update(data.as_bytes());
                let result = mac.finalize();
                Ok(LSLValue::String(hex::encode(result.into_bytes())))
            }
        }
    }

    async fn ll_compute_hash(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llComputeHash expects 2 arguments"));
        }
        let data = args[0].to_string();
        let algorithm = args[1].to_string();

        let hash = match algorithm.as_str() {
            "md5" | "MD5" => format!("{:x}", md5::compute(data.as_bytes())),
            "sha1" | "SHA1" => {
                use sha1::{Digest, Sha1};
                let mut hasher = Sha1::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            "sha256" | "SHA256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            "sha512" | "SHA512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            _ => format!("{:x}", md5::compute(data.as_bytes())),
        };
        Ok(LSLValue::String(hash))
    }

    async fn ll_hash(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llHash expects 1 argument"));
        }
        let data = args[0].to_string();
        let mut hash: i32 = 0;
        for byte in data.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as i32);
        }
        Ok(LSLValue::Integer(hash))
    }

    async fn ll_string_to_base64(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llStringToBase64 expects 1 argument"));
        }
        let src = args[0].to_string();
        use base64::{engine::general_purpose, Engine as _};
        Ok(LSLValue::String(
            general_purpose::STANDARD.encode(src.as_bytes()),
        ))
    }

    async fn ll_base64_to_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llBase64ToString expects 1 argument"));
        }
        let src = args[0].to_string();
        use base64::{engine::general_purpose, Engine as _};
        match general_purpose::STANDARD.decode(&src) {
            Ok(bytes) => Ok(LSLValue::String(
                String::from_utf8_lossy(&bytes).to_string(),
            )),
            Err(_) => Ok(LSLValue::String(String::new())),
        }
    }

    async fn ll_integer_to_base64(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llIntegerToBase64 expects 1 argument"));
        }
        let value = args[0].to_integer();
        let bytes = value.to_be_bytes();
        use base64::{engine::general_purpose, Engine as _};
        Ok(LSLValue::String(general_purpose::STANDARD.encode(&bytes)))
    }

    async fn ll_base64_to_integer(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llBase64ToInteger expects 1 argument"));
        }
        let src = args[0].to_string();
        use base64::{engine::general_purpose, Engine as _};
        match general_purpose::STANDARD.decode(&src) {
            Ok(bytes) if bytes.len() >= 4 => {
                let value = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Ok(LSLValue::Integer(value))
            }
            _ => Ok(LSLValue::Integer(0)),
        }
    }

    async fn ll_xor_base64(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llXorBase64 expects 2 arguments"));
        }
        let str1 = args[0].to_string();
        let str2 = args[1].to_string();
        use base64::{engine::general_purpose, Engine as _};

        let bytes1 = general_purpose::STANDARD.decode(&str1).unwrap_or_default();
        let bytes2 = general_purpose::STANDARD.decode(&str2).unwrap_or_default();

        let result: Vec<u8> = bytes1
            .iter()
            .zip(bytes2.iter().cycle())
            .map(|(a, b)| a ^ b)
            .collect();

        Ok(LSLValue::String(general_purpose::STANDARD.encode(&result)))
    }

    async fn ll_xor_base64_strings(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.ll_xor_base64(args, _context).await
    }

    async fn ll_xor_base64_strings_correct(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.ll_xor_base64(args, _context).await
    }

    async fn ll_char(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llChar expects 1 argument"));
        }
        let code = args[0].to_integer() as u32;
        let c = char::from_u32(code).unwrap_or('\0');
        Ok(LSLValue::String(c.to_string()))
    }

    async fn ll_ord(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llOrd expects 2 arguments"));
        }
        let src = args[0].to_string();
        let index = args[1].to_integer() as usize;
        let code = src.chars().nth(index).map(|c| c as i32).unwrap_or(0);
        Ok(LSLValue::Integer(code))
    }

    async fn ll_escape_url(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llEscapeURL expects 1 argument"));
        }
        let src = args[0].to_string();
        let escaped = urlencoding::encode(&src);
        Ok(LSLValue::String(escaped.to_string()))
    }

    async fn ll_unescape_url(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llUnescapeURL expects 1 argument"));
        }
        let src = args[0].to_string();
        let unescaped = urlencoding::decode(&src).unwrap_or_default();
        Ok(LSLValue::String(unescaped.to_string()))
    }

    async fn ll_string_trim(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llStringTrim expects 2 arguments"));
        }
        let src = args[0].to_string();
        let trim_type = args[1].to_integer();
        let result = match trim_type {
            1 => src.trim_start().to_string(),
            2 => src.trim_end().to_string(),
            3 => src.trim().to_string(),
            _ => src,
        };
        Ok(LSLValue::String(result))
    }

    async fn ll_replace_sub_string(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llReplaceSubString expects 4 arguments"));
        }
        let src = args[0].to_string();
        let pattern = args[1].to_string();
        let replacement = args[2].to_string();
        let count = args[3].to_integer();

        if count == 0 {
            Ok(LSLValue::String(src.replace(&pattern, &replacement)))
        } else {
            Ok(LSLValue::String(src.replacen(
                &pattern,
                &replacement,
                count as usize,
            )))
        }
    }

    async fn ll_list_statistics(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llListStatistics expects 2 arguments"));
        }
        let operation = args[0].to_integer();
        let list = args[1].to_list();

        let numbers: Vec<f32> = list
            .iter()
            .filter_map(|v| match v {
                LSLValue::Float(f) => Some(*f),
                LSLValue::Integer(i) => Some(*i as f32),
                _ => None,
            })
            .collect();

        if numbers.is_empty() {
            return Ok(LSLValue::Float(0.0));
        }

        let result = match operation {
            0 => {
                // LIST_STAT_RANGE
                let min = numbers.iter().cloned().reduce(f32::min).unwrap_or(0.0);
                let max = numbers.iter().cloned().reduce(f32::max).unwrap_or(0.0);
                max - min
            }
            1 => numbers.iter().cloned().reduce(f32::min).unwrap_or(0.0), // LIST_STAT_MIN
            2 => numbers.iter().cloned().reduce(f32::max).unwrap_or(0.0), // LIST_STAT_MAX
            3 => numbers.iter().sum::<f32>() / numbers.len() as f32,      // LIST_STAT_MEAN
            4 => {
                // LIST_STAT_MEDIAN
                let mut sorted = numbers.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[mid]
                }
            }
            5 => {
                // LIST_STAT_STD_DEV
                let mean = numbers.iter().sum::<f32>() / numbers.len() as f32;
                let variance =
                    numbers.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / numbers.len() as f32;
                variance.sqrt()
            }
            6 => numbers.iter().sum(),                // LIST_STAT_SUM
            7 => numbers.iter().map(|x| x * x).sum(), // LIST_STAT_SUM_SQUARES
            8 => numbers.len() as f32,                // LIST_STAT_NUM_COUNT
            9 => {
                // LIST_STAT_GEOMETRIC_MEAN
                let product: f64 = numbers.iter().map(|x| (*x as f64).ln()).sum();
                (product / numbers.len() as f64).exp() as f32
            }
            _ => 0.0,
        };

        Ok(LSLValue::Float(result))
    }

    async fn ll_list_sort(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llListSort expects 3 arguments"));
        }
        let list = args[0].to_list();
        let stride = args[1].to_integer().max(1) as usize;
        let ascending = args[2].is_true();

        if list.is_empty() || stride == 0 {
            return Ok(LSLValue::List(list));
        }

        if stride == 1 {
            let mut sorted = list;
            sorted.sort_by(|a, b| {
                let cmp = a.to_string().cmp(&b.to_string());
                if ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
            return Ok(LSLValue::List(sorted));
        }

        let chunk_count = list.len() / stride;
        if chunk_count == 0 {
            return Ok(LSLValue::List(list));
        }

        let mut chunks: Vec<Vec<LSLValue>> = (0..chunk_count)
            .map(|i| list[i * stride..(i + 1) * stride].to_vec())
            .collect();
        let remainder = &list[chunk_count * stride..];

        chunks.sort_by(|a, b| {
            let cmp = a[0].to_string().cmp(&b[0].to_string());
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        let mut result: Vec<LSLValue> = chunks.into_iter().flatten().collect();
        result.extend_from_slice(remainder);
        Ok(LSLValue::List(result))
    }

    async fn ll_list_sort_strided(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.ll_list_sort(args, _context).await
    }

    async fn ll_list_randomize(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llListRandomize expects 2 arguments"));
        }
        let mut list = args[0].to_list();
        let _stride = args[1].to_integer();

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        list.shuffle(&mut rng);

        Ok(LSLValue::List(list))
    }

    async fn ll_list_replace_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llListReplaceList expects 4 arguments"));
        }
        let mut dest = args[0].to_list();
        let src = args[1].to_list();
        let start = args[2].to_integer() as usize;
        let end = args[3].to_integer() as usize;

        if start <= end && end < dest.len() {
            dest.splice(start..=end, src);
        }

        Ok(LSLValue::List(dest))
    }

    async fn ll_list2list(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llList2List expects 3 arguments"));
        }
        let list = args[0].to_list();
        let start = args[1].to_integer();
        let end = args[2].to_integer();

        let len = list.len() as i32;
        let start = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len)
        } as usize;
        let end = if end < 0 {
            (len + end + 1).max(0)
        } else {
            (end + 1).min(len)
        } as usize;

        Ok(LSLValue::List(list[start..end].to_vec()))
    }

    async fn ll_list2list_slice(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.ll_list2list(args, _context).await
    }

    async fn ll_list2list_strided(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 4 {
            return Err(anyhow!("llList2ListStrided expects 4 arguments"));
        }
        let list = args[0].to_list();
        let start = args[1].to_integer() as usize;
        let end = args[2].to_integer() as usize;
        let stride = args[3].to_integer().max(1) as usize;

        let result: Vec<LSLValue> = list[start..=end.min(list.len() - 1)]
            .iter()
            .step_by(stride)
            .cloned()
            .collect();

        Ok(LSLValue::List(result))
    }

    async fn ll_list_find_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llListFindList expects 2 arguments"));
        }
        let haystack = args[0].to_list();
        let needle = args[1].to_list();

        if needle.is_empty() {
            return Ok(LSLValue::Integer(-1));
        }

        for i in 0..=(haystack.len().saturating_sub(needle.len())) {
            if haystack[i..i + needle.len()]
                .iter()
                .zip(needle.iter())
                .all(|(a, b)| a.to_string() == b.to_string())
            {
                return Ok(LSLValue::Integer(i as i32));
            }
        }

        Ok(LSLValue::Integer(-1))
    }

    async fn ll_list_find_list_next(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llListFindListNext expects 3 arguments"));
        }
        let haystack = args[0].to_list();
        let needle = args[1].to_list();
        let start = args[2].to_integer() as usize;

        if needle.is_empty() || start >= haystack.len() {
            return Ok(LSLValue::Integer(-1));
        }

        for i in start..=(haystack.len().saturating_sub(needle.len())) {
            if haystack[i..i + needle.len()]
                .iter()
                .zip(needle.iter())
                .all(|(a, b)| a.to_string() == b.to_string())
            {
                return Ok(LSLValue::Integer(i as i32));
            }
        }

        Ok(LSLValue::Integer(-1))
    }

    async fn ll_list_find_strided(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llListFindStrided expects 5 arguments"));
        }
        self.ll_list_find_list(&args[0..2].to_vec(), _context).await
    }

    async fn ll_get_list_entry_type(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetListEntryType expects 2 arguments"));
        }
        let list = args[0].to_list();
        let index = args[1].to_integer() as usize;

        if let Some(item) = list.get(index) {
            let type_val = match item {
                LSLValue::Integer(_) => 1,
                LSLValue::Float(_) => 2,
                LSLValue::String(_) => 3,
                LSLValue::Key(_) => 4,
                LSLValue::Vector(_) => 5,
                LSLValue::Rotation(_) => 6,
                LSLValue::List(_) => 7,
            };
            Ok(LSLValue::Integer(type_val))
        } else {
            Ok(LSLValue::Integer(0))
        }
    }

    async fn ll_parse_string_keep_nulls(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llParseStringKeepNulls expects 3 arguments"));
        }
        let src = args[0].to_string();
        let separators = args[1].to_list();
        let _spacers = args[2].to_list();

        let sep_strs: Vec<String> = separators.iter().map(|v| v.to_string()).collect();
        let mut result = vec![src];

        for sep in sep_strs {
            let mut new_result = Vec::new();
            for item in result {
                for (i, part) in item.split(&sep).enumerate() {
                    if i > 0 {
                        new_result.push(String::new());
                    }
                    new_result.push(part.to_string());
                }
            }
            result = new_result;
        }

        Ok(LSLValue::List(
            result.into_iter().map(LSLValue::String).collect(),
        ))
    }

    async fn ll_axis_angle2rot(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llAxisAngle2Rot expects 2 arguments"));
        }
        let axis = args[0].to_vector();
        let angle = args[1].to_float();

        let half_angle = angle / 2.0;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();
        let mag = (axis.x * axis.x + axis.y * axis.y + axis.z * axis.z).sqrt();

        if mag > 0.0001 {
            Ok(LSLValue::Rotation(LSLRotation::new(
                axis.x / mag * sin_half,
                axis.y / mag * sin_half,
                axis.z / mag * sin_half,
                cos_half,
            )))
        } else {
            Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
        }
    }

    async fn ll_rot_between(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llRotBetween expects 2 arguments"));
        }
        let v1 = args[0].to_vector();
        let v2 = args[1].to_vector();

        let dot = v1.x * v2.x + v1.y * v2.y + v1.z * v2.z;
        let cross_x = v1.y * v2.z - v1.z * v2.y;
        let cross_y = v1.z * v2.x - v1.x * v2.z;
        let cross_z = v1.x * v2.y - v1.y * v2.x;

        let mag = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        let angle = dot.atan2(mag);
        let half_angle = angle / 2.0;

        if mag > 0.0001 {
            Ok(LSLValue::Rotation(LSLRotation::new(
                cross_x / mag * half_angle.sin(),
                cross_y / mag * half_angle.sin(),
                cross_z / mag * half_angle.sin(),
                half_angle.cos(),
            )))
        } else {
            Ok(LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)))
        }
    }

    async fn ll_rot2fwd(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Fwd expects 1 argument"));
        }
        let rot = args[0].to_rotation();
        let x = 1.0 - 2.0 * (rot.y * rot.y + rot.z * rot.z);
        let y = 2.0 * (rot.x * rot.y + rot.z * rot.s);
        let z = 2.0 * (rot.x * rot.z - rot.y * rot.s);
        Ok(LSLValue::Vector(LSLVector::new(x, y, z)))
    }

    async fn ll_rot2left(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Left expects 1 argument"));
        }
        let rot = args[0].to_rotation();
        let x = 2.0 * (rot.x * rot.y - rot.z * rot.s);
        let y = 1.0 - 2.0 * (rot.x * rot.x + rot.z * rot.z);
        let z = 2.0 * (rot.y * rot.z + rot.x * rot.s);
        Ok(LSLValue::Vector(LSLVector::new(x, y, z)))
    }

    async fn ll_rot2up(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRot2Up expects 1 argument"));
        }
        let rot = args[0].to_rotation();
        let x = 2.0 * (rot.x * rot.z + rot.y * rot.s);
        let y = 2.0 * (rot.y * rot.z - rot.x * rot.s);
        let z = 1.0 - 2.0 * (rot.x * rot.x + rot.y * rot.y);
        Ok(LSLValue::Vector(LSLVector::new(x, y, z)))
    }

    async fn ll_get_local_rot(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(
            context.rotation.0,
            context.rotation.1,
            context.rotation.2,
            context.rotation.3,
        )))
    }

    async fn ll_set_local_rot(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetLocalRot expects 1 argument"));
        }
        let rot = args[0].to_rotation();
        context.rotation = (rot.x, rot.y, rot.z, rot.s);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetRot {
                object_id: context.object_id,
                rotation: [rot.x, rot.y, rot.z, rot.s],
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_root_rotation(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Rotation(LSLRotation::new(
            context.rotation.0,
            context.rotation.1,
            context.rotation.2,
            context.rotation.3,
        )))
    }

    async fn ll_get_local_pos(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(
            context.position.0,
            context.position.1,
            context.position.2,
        )))
    }

    async fn ll_get_root_position(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(
            context.position.0,
            context.position.1,
            context.position.2,
        )))
    }

    async fn ll_give_inventory(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGiveInventory expects 2 arguments"));
        }
        let destination_id = args[0].to_key();
        let item_name = args[1].to_string();
        info!(
            "llGiveInventory: giving '{}' to {}",
            item_name, destination_id
        );
        if !destination_id.is_nil() {
            self.action_queue.lock().push((
                context.script_id,
                crate::scripting::executor::ScriptAction::GiveInventory {
                    prim_id: context.object_id,
                    destination_id,
                    item_name,
                    owner_id: context.owner_id,
                },
            ));
        }
        Ok(LSLValue::Integer(0))
    }

    async fn ll_give_inventory_list(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llGiveInventoryList expects 3 arguments"));
        }
        let destination = args[0].to_key();
        let folder = args[1].to_string();
        let items = args[2].to_list();
        debug!(
            "Giving inventory list to {}: folder={}, {} items",
            destination,
            folder,
            items.len()
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remove_inventory(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRemoveInventory expects 1 argument"));
        }
        let item = args[0].to_string();
        debug!("Removing inventory: {}", item);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_inventory_creator(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryCreator expects 1 argument"));
        }
        let _item = args[0].to_string();
        Ok(LSLValue::Key(Uuid::nil()))
    }

    async fn ll_get_inventory_perm_mask(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetInventoryPermMask expects 2 arguments"));
        }
        let item_name = args[0].to_string();
        let mask_type = args[1].to_integer();
        if let Some(inv_item) = context.inventory.iter().find(|i| i.name == item_name) {
            let perms = inv_item.permissions;
            let result = match mask_type {
                0 => perms,
                1 => perms,
                2 => 0,
                3 => 0,
                4 => context.next_owner_mask,
                _ => 0,
            };
            Ok(LSLValue::Integer(result as i32))
        } else {
            Ok(LSLValue::Integer(-1))
        }
    }

    async fn ll_set_inventory_perm_mask(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetInventoryPermMask expects 3 arguments"));
        }
        let item = args[0].to_string();
        let mask = args[1].to_integer();
        let value = args[2].to_integer();
        debug!(
            "Setting inventory perm mask for {}: mask={}, value={}",
            item, mask, value
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_inventory_desc(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryDesc expects 1 argument"));
        }
        let _item = args[0].to_string();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_get_inventory_acquire_time(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetInventoryAcquireTime expects 1 argument"));
        }
        let _item = args[0].to_string();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_request_inventory_data(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llRequestInventoryData expects 1 argument"));
        }
        let item = args[0].to_string();
        let request_id = Uuid::new_v4();
        let data = if let Some(inv) = context.inventory.iter().find(|i| i.name == item) {
            format!(
                "<{:.6}, {:.6}, {:.6}>",
                context.position.0, context.position.1, context.position.2
            )
        } else {
            String::new()
        };
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data,
            },
        ));
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_get_object_perm_mask(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetObjectPermMask expects 1 argument"));
        }
        let mask_type = args[0].to_integer();
        let result = match mask_type {
            0 => context.base_mask,
            1 => context.owner_mask,
            2 => context.group_mask,
            3 => context.everyone_mask,
            4 => context.next_owner_mask,
            _ => 0,
        };
        Ok(LSLValue::Integer(result as i32))
    }

    async fn ll_set_object_perm_mask(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetObjectPermMask expects 2 arguments"));
        }
        let mask = args[0].to_integer();
        let value = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetObjectPermMask {
                object_id: context.object_id,
                mask_type: mask,
                mask_value: value as u32,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_notecard_line(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetNotecardLine expects 2 arguments"));
        }
        let name = args[0].to_string();
        let line = args[1].to_integer();
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: EOF_MARKER.to_string(),
            },
        ));
        debug!(
            "llGetNotecardLine notecard='{}' line={} query={}",
            name, line, request_id
        );
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_get_notecard_line_sync(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetNotecardLineSync expects 2 arguments"));
        }
        let _name = args[0].to_string();
        let _line = args[1].to_integer();
        Ok(LSLValue::String(String::new()))
    }

    async fn ll_get_number_of_notecard_lines(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetNumberOfNotecardLines expects 1 argument"));
        }
        let name = args[0].to_string();
        let request_id = Uuid::new_v4();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::DataserverReply {
                script_id: context.script_id,
                query_id: request_id.to_string(),
                data: "0".to_string(),
            },
        ));
        debug!(
            "llGetNumberOfNotecardLines notecard='{}' query={}",
            name, request_id
        );
        Ok(LSLValue::Key(request_id))
    }

    async fn ll_set_prim_media_params(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetPrimMediaParams expects 2 arguments"));
        }
        let face = args[0].to_integer();
        if face < 0 || face > 8 {
            return Ok(LSLValue::Integer(1));
        }
        let params = args[1].to_list();
        context.variables.insert(
            format!("__media_face_{}", face),
            LSLValue::List(params.clone()),
        );
        let mut param_pairs: Vec<(i32, String)> = Vec::new();
        let mut pi = 0;
        while pi + 1 < params.len() {
            param_pairs.push((params[pi].to_integer(), params[pi + 1].to_string()));
            pi += 2;
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPrimMediaParams {
                object_id: context.object_id,
                face,
                params: param_pairs,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_prim_media_params(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetPrimMediaParams expects 2 arguments"));
        }
        let face = args[0].to_integer();
        let param_codes = args[1].to_list();
        if face < 0 || face > 8 {
            return Ok(LSLValue::List(vec![]));
        }
        let stored = match context.variables.get(&format!("__media_face_{}", face)) {
            Some(LSLValue::List(l)) => l.clone(),
            _ => vec![],
        };
        let mut result = Vec::new();
        for code in &param_codes {
            let code_val = code.to_integer();
            let mut found = false;
            let mut i = 0;
            while i + 1 < stored.len() {
                if stored[i].to_integer() == code_val {
                    result.push(stored[i + 1].clone());
                    found = true;
                    break;
                }
                i += 2;
            }
            if !found {
                result.push(LSLValue::String(String::new()));
            }
        }
        Ok(LSLValue::List(result))
    }

    async fn ll_clear_prim_media(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llClearPrimMedia expects 1 argument"));
        }
        let face = args[0].to_integer();
        if face < 0 || face > 8 {
            return Ok(LSLValue::Integer(1));
        }
        context.variables.remove(&format!("__media_face_{}", face));
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ClearPrimMedia {
                object_id: context.object_id,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_link_media(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llSetLinkMedia expects 3 arguments"));
        }
        let _link = args[0].to_integer();
        let face = args[1].to_integer();
        let params = args[2].to_list();
        let mut param_pairs: Vec<(i32, String)> = Vec::new();
        let mut pi = 0;
        while pi + 1 < params.len() {
            param_pairs.push((params[pi].to_integer(), params[pi + 1].to_string()));
            pi += 2;
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPrimMediaParams {
                object_id: context.object_id,
                face,
                params: param_pairs,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_link_media(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llGetLinkMedia expects 3 arguments"));
        }
        let _link = args[0].to_integer();
        let _face = args[1].to_integer();
        let params = args[2].to_list();
        Ok(LSLValue::List(vec![
            LSLValue::String(String::new());
            params.len()
        ]))
    }

    async fn ll_clear_link_media(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llClearLinkMedia expects 2 arguments"));
        }
        let _link = args[0].to_integer();
        let face = args[1].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ClearPrimMedia {
                object_id: context.object_id,
                face,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_parcel_media_command_list(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llParcelMediaCommandList expects 1 argument"));
        }
        let commands = args[0].to_list();
        let cmd_ints: Vec<i32> = commands.iter().map(|c| c.to_integer()).collect();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ParcelMediaCommandList {
                object_id: context.object_id,
                commands: cmd_ints,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_parcel_media_query(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llParcelMediaQuery expects 1 argument"));
        }
        let query = args[0].to_list();
        Ok(LSLValue::List(vec![
            LSLValue::String(String::new());
            query.len()
        ]))
    }

    async fn ll_set_prim_url(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetPrimURL expects 1 argument"));
        }
        let url = args[0].to_string();
        debug!("Setting prim URL: {}", url);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_refresh_prim_url(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        debug!("Refreshing prim URL");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_cast_ray(&self, args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llCastRay expects 3 arguments"));
        }
        let start = args[0].to_vector();
        let end = args[1].to_vector();
        let options = args[2].to_list();
        let mut reject_types = 0i32;
        let mut max_hits = 1i32;
        let mut i = 0;
        while i + 1 < options.len() {
            match options[i].to_integer() {
                0 => {
                    reject_types = options[i + 1].to_integer();
                }
                2 => {
                    max_hits = options[i + 1].to_integer().max(1).min(256);
                }
                _ => {}
            }
            i += 2;
        }
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::CastRay {
                object_id: context.object_id,
                start: [start.x, start.y, start.z],
                end: [end.x, end.y, end.z],
                reject_types,
                max_hits,
            },
        ));
        Ok(LSLValue::List(vec![LSLValue::Integer(0)]))
    }

    async fn ll_cast_ray_v3(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        self.ll_cast_ray(args, _context).await
    }

    async fn ll_linear2srgb(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llLinear2sRGB expects 1 argument"));
        }
        let color = args[0].to_vector();
        fn linear_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }
        Ok(LSLValue::Vector(LSLVector::new(
            linear_to_srgb(color.x),
            linear_to_srgb(color.y),
            linear_to_srgb(color.z),
        )))
    }

    async fn ll_srgb2linear(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llsRGB2Linear expects 1 argument"));
        }
        let color = args[0].to_vector();
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        Ok(LSLValue::Vector(LSLVector::new(
            srgb_to_linear(color.x),
            srgb_to_linear(color.y),
            srgb_to_linear(color.z),
        )))
    }

    async fn ll_get_time(&self, _args: &[LSLValue], context: &ScriptContext) -> Result<LSLValue> {
        let elapsed = context.script_start_time.elapsed().as_secs_f32();
        Ok(LSLValue::Float(elapsed))
    }

    async fn ll_get_and_reset_time(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        let elapsed = context.script_start_time.elapsed().as_secs_f32();
        context.script_start_time = std::time::Instant::now();
        Ok(LSLValue::Float(elapsed))
    }

    async fn ll_reset_time(
        &self,
        _args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        context.script_start_time = std::time::Instant::now();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_date(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        let now = chrono::Utc::now();
        Ok(LSLValue::String(now.format("%Y-%m-%d").to_string()))
    }

    async fn ll_get_gmt_clock(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        use chrono::Timelike;
        let now = chrono::Utc::now();
        let seconds = now.num_seconds_from_midnight() as f32;
        Ok(LSLValue::Float(seconds))
    }

    async fn ll_get_wallclock(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let seconds = now.num_seconds_from_midnight() as f32;
        Ok(LSLValue::Float(seconds))
    }

    async fn ll_get_time_of_day(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        let seconds = (chrono::Utc::now().timestamp() % 14400) as f32;
        Ok(LSLValue::Float(seconds))
    }

    async fn ll_get_script_state(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llGetScriptState expects 1 argument"));
        }
        let _script = args[0].to_string();
        Ok(LSLValue::Integer(1))
    }

    async fn ll_set_script_state(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetScriptState expects 2 arguments"));
        }
        let script = args[0].to_string();
        let running = args[1].is_true();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetScriptState {
                object_id: context.object_id,
                script_name: script,
                running,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_reset_other_script(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llResetOtherScript expects 1 argument"));
        }
        let script = args[0].to_string();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ResetOtherScript {
                object_id: context.object_id,
                script_name: script,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remote_load_script(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() < 4 {
            return Err(anyhow!("llRemoteLoadScript expects at least 4 arguments"));
        }
        debug!("Remote load script (deprecated)");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_remote_load_script_pin(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("llRemoteLoadScriptPin expects 5 arguments"));
        }
        let target = args[0].to_key();
        let script = args[1].to_string();
        let pin = args[2].to_integer();
        let running = args[3].is_true();
        let start_param = args[4].to_integer();
        debug!(
            "Remote load script {} to {}, pin={}, running={}, param={}",
            script, target, pin, running, start_param
        );
        Ok(LSLValue::Integer(0))
    }

    async fn ll_set_remote_script_access_pin(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetRemoteScriptAccessPin expects 1 argument"));
        }
        let pin = args[0].to_integer();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetScriptAccessPin {
                object_id: context.object_id,
                pin,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_start_parameter(
        &self,
        _args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(context.start_parameter))
    }

    async fn ll_min_event_delay(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llMinEventDelay expects 1 argument"));
        }
        let delay = args[0].to_float().max(0.0);
        context.min_event_delay = delay as f64;
        Ok(LSLValue::Integer(0))
    }

    async fn ll_script_danger(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llScriptDanger expects 1 argument"));
        }
        let _pos = args[0].to_vector();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_script_profiler(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llScriptProfiler expects 1 argument"));
        }
        let enable = args[0].to_integer();
        debug!("Script profiler: {}", enable);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_memory_limit(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(65536))
    }

    async fn ll_set_memory_limit(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetMemoryLimit expects 1 argument"));
        }
        let _limit = args[0].to_integer();
        Ok(LSLValue::Integer(65536))
    }

    async fn ll_get_sp_max_memory(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Integer(65536))
    }

    async fn ll_generate_key(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    async fn ll_same_group(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSameGroup expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_is_friend(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llIsFriend expects 1 argument"));
        }
        let _id = args[0].to_key();
        Ok(LSLValue::Integer(0))
    }

    async fn ll_scale_by_factor(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llScaleByFactor expects 1 argument"));
        }
        let factor = args[0].to_float().clamp(1e-6, 1e6) as f64;
        context.scale.0 *= factor as f32;
        context.scale.1 *= factor as f32;
        context.scale.2 *= factor as f32;
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ScaleByFactor {
                object_id: context.object_id,
                factor,
            },
        ));
        Ok(LSLValue::Integer(1))
    }

    async fn ll_get_max_scale_factor(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(64.0))
    }

    async fn ll_get_min_scale_factor(
        &self,
        _args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        Ok(LSLValue::Float(0.01))
    }

    async fn ll_manage_estate_access(
        &self,
        args: &[LSLValue],
        context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llManageEstateAccess expects 2 arguments"));
        }
        let action = args[0].to_integer();
        let agent_id = args[1].to_key();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::ManageEstateAccess {
                object_id: context.object_id,
                action,
                agent_id,
            },
        ));
        Ok(LSLValue::Integer(1))
    }

    async fn ll_set_region_pos(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llSetRegionPos expects 1 argument"));
        }
        let pos = args[0].to_vector();
        context.position = (pos.x, pos.y, pos.z);
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::SetPos {
                object_id: context.object_id,
                position: [pos.x, pos.y, pos.z],
            },
        ));
        Ok(LSLValue::Integer(1))
    }

    async fn ll_set_keyframed_motion(
        &self,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llSetKeyframedMotion expects 2 arguments"));
        }
        let keyframes = args[0].to_list();
        let options = args[1].to_list();
        let mut mode = 0i32;
        let mut data = 0i32;
        let mut i = 0;
        while i + 1 < options.len() {
            let opt = options[i].to_integer();
            let val = options[i + 1].to_integer();
            match opt {
                0 => mode = val,
                1 => data = val,
                2 => {
                    data = val;
                }
                _ => {}
            }
            i += 2;
        }
        let kf_data: Vec<f32> = keyframes.iter().map(|v| v.to_float()).collect();
        self.action_queue.lock().push((
            context.script_id,
            ScriptAction::KeyframedMotion {
                object_id: context.object_id,
                keyframes: kf_data,
                mode,
                data,
            },
        ));
        Ok(LSLValue::Integer(0))
    }

    async fn ll_collision_sprite(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("llCollisionSprite expects 1 argument"));
        }
        let _texture = args[0].to_string();
        debug!("Collision sprite (deprecated)");
        Ok(LSLValue::Integer(0))
    }

    async fn ll_god_like_rez_object(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGodLikeRezObject expects 2 arguments"));
        }
        let inventory = args[0].to_key();
        let pos = args[1].to_vector();
        debug!("God-like rezzing object {} at {:?}", inventory, pos);
        Ok(LSLValue::Integer(0))
    }

    async fn ll_get_visual_params(
        &self,
        args: &[LSLValue],
        _context: &ScriptContext,
    ) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("llGetVisualParams expects 2 arguments"));
        }
        let _id = args[0].to_key();
        let params = args[1].to_list();
        Ok(LSLValue::List(vec![LSLValue::Float(0.5); params.len()]))
    }

    async fn ll_mod_pow(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("llModPow expects 3 arguments"));
        }
        let base = args[0].to_integer();
        let exp = args[1].to_integer();
        let modulus = args[2].to_integer();

        if modulus == 0 {
            return Ok(LSLValue::Integer(0));
        }

        let mut result: i64 = 1;
        let mut base = (base as i64) % (modulus as i64);
        let mut exp = exp as u32;

        while exp > 0 {
            if exp % 2 == 1 {
                result = (result * base) % (modulus as i64);
            }
            exp /= 2;
            base = (base * base) % (modulus as i64);
        }

        Ok(LSLValue::Integer(result as i32))
    }
}
