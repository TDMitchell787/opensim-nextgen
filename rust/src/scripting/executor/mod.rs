pub mod bytecode;
pub mod cranelift_jit;
pub mod tree_walk;

use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

use super::lsl_interpreter::ASTNode;
use super::{LSLRotation, LSLValue, LSLVector};

#[derive(Debug, Clone)]
pub struct ObjectContext {
    pub object_id: Uuid,
    pub owner_id: Uuid,
    pub object_name: String,
    pub position: LSLVector,
    pub rotation: LSLRotation,
    pub scale: LSLVector,
    pub velocity: LSLVector,
    pub region_name: String,
    pub detect_params: Vec<DetectInfo>,
    pub granted_perms: u32,
    pub perm_granter: Uuid,
    pub sitting_avatar_id: Uuid,
    pub link_num: i32,
    pub link_count: i32,
    pub link_names: Vec<(i32, String)>,
    pub link_scales: Vec<(i32, LSLVector)>,
    pub inventory: Vec<TaskInventoryEntry>,
    pub base_mask: u32,
    pub owner_mask: u32,
    pub group_mask: u32,
    pub everyone_mask: u32,
    pub next_owner_mask: u32,
}

#[derive(Debug, Clone)]
pub struct TaskInventoryEntry {
    pub name: String,
    pub asset_id: Uuid,
    pub inv_type: i32,
    pub asset_type: i32,
    pub permissions: u32,
}

impl Default for ObjectContext {
    fn default() -> Self {
        Self {
            object_id: Uuid::nil(),
            owner_id: Uuid::nil(),
            object_name: String::new(),
            position: LSLVector::zero(),
            rotation: LSLRotation::identity(),
            scale: LSLVector::new(1.0, 1.0, 1.0),
            velocity: LSLVector::zero(),
            region_name: String::new(),
            detect_params: Vec::new(),
            granted_perms: 0,
            perm_granter: Uuid::nil(),
            sitting_avatar_id: Uuid::nil(),
            link_num: 0,
            link_count: 1,
            link_names: Vec::new(),
            link_scales: Vec::new(),
            inventory: Vec::new(),
            base_mask: 0x7FFFFFFF,
            owner_mask: 0x7FFFFFFF,
            group_mask: 0,
            everyone_mask: 0,
            next_owner_mask: 0x7FFFFFFF,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectInfo {
    pub key: Uuid,
    pub name: String,
    pub owner: Uuid,
    pub position: LSLVector,
    pub rotation: LSLRotation,
    pub velocity: LSLVector,
    pub link_num: i32,
    pub det_type: i32,
}

#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub script_id: Uuid,
    pub globals: Vec<(String, String, Option<ASTNode>)>,
    pub functions: HashMap<String, UserFunction>,
    pub states: HashMap<String, StateDefinition>,
    pub ast: ASTNode,
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<(String, String)>,
    pub body: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct StateDefinition {
    pub name: String,
    pub events: HashMap<String, EventHandler>,
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub name: String,
    pub parameters: Vec<(String, String)>,
    pub body: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Complete(LSLValue),
    StateChange(String),
    Yield,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PrimParamRule {
    Color {
        face: i32,
        color: [f32; 3],
        alpha: f32,
    },
    Texture {
        face: i32,
        texture_id: Uuid,
        repeats: [f32; 2],
        offsets: [f32; 2],
        rotation: f32,
    },
    Text {
        text: String,
        color: [f32; 3],
        alpha: f32,
    },
    Fullbright {
        face: i32,
        value: bool,
    },
    Glow {
        face: i32,
        intensity: f32,
    },
    BumpShiny {
        face: i32,
        shiny: i32,
        bump: i32,
    },
    PointLight {
        enabled: bool,
        color: [f32; 3],
        intensity: f32,
        radius: f32,
        falloff: f32,
    },
    Size {
        size: [f32; 3],
    },
    Position {
        pos: [f32; 3],
    },
    Rotation {
        rot: [f32; 4],
    },
    Material {
        material: i32,
    },
    Phantom {
        value: bool,
    },
    Physics {
        value: bool,
    },
    TempOnRez {
        value: bool,
    },
    Flexible {
        enabled: bool,
        softness: i32,
        gravity: f32,
        friction: f32,
        wind: f32,
        tension: f32,
        force: [f32; 3],
    },
    Omega {
        axis: [f32; 3],
        spinrate: f64,
        gain: f64,
    },
}

#[derive(Debug, Clone)]
pub enum ScriptAction {
    Say {
        channel: i32,
        message: String,
        object_id: Uuid,
    },
    Shout {
        channel: i32,
        message: String,
        object_id: Uuid,
    },
    Whisper {
        channel: i32,
        message: String,
        object_id: Uuid,
    },
    OwnerSay {
        message: String,
        object_id: Uuid,
        owner_id: Uuid,
    },
    SetTimerEvent {
        interval: f64,
    },
    Listen {
        channel: i32,
        name: String,
        id: String,
        msg: String,
    },
    SetPos {
        object_id: Uuid,
        position: [f32; 3],
    },
    SetRot {
        object_id: Uuid,
        rotation: [f32; 4],
    },
    SetScale {
        object_id: Uuid,
        scale: [f32; 3],
    },
    SetText {
        object_id: Uuid,
        text: String,
        color: [f32; 3],
        alpha: f32,
    },
    Die {
        object_id: Uuid,
    },
    RequestPermissions {
        script_id: Uuid,
        object_id: Uuid,
        object_name: String,
        avatar_id: Uuid,
        permissions: u32,
    },
    ApplyImpulse {
        object_id: Uuid,
        impulse: [f32; 3],
        local: bool,
    },
    ResetScript,
    MessageLinked {
        link_num: i32,
        num: i32,
        str_val: String,
        id: String,
    },
    Dialog {
        avatar_id: Uuid,
        object_name: String,
        message: String,
        buttons: Vec<String>,
        channel: i32,
        object_id: Uuid,
    },
    ListenRemove {
        handle: i32,
    },
    Sleep {
        seconds: f64,
    },
    RegionSay {
        channel: i32,
        message: String,
        object_id: Uuid,
    },
    RegionSayTo {
        target_id: Uuid,
        channel: i32,
        message: String,
        object_id: Uuid,
    },
    TakeControls {
        script_id: Uuid,
        object_id: Uuid,
        avatar_id: Uuid,
        controls: i32,
        accept: bool,
        pass_on: bool,
    },
    ReleaseControls {
        script_id: Uuid,
        object_id: Uuid,
        avatar_id: Uuid,
    },
    SetVehicleType {
        object_id: Uuid,
        vehicle_type: i32,
    },
    SetVehicleFloatParam {
        object_id: Uuid,
        param_id: i32,
        value: f64,
    },
    SetVehicleVectorParam {
        object_id: Uuid,
        param_id: i32,
        value: [f32; 3],
    },
    SetVehicleRotationParam {
        object_id: Uuid,
        param_id: i32,
        value: [f32; 4],
    },
    SetVehicleFlags {
        object_id: Uuid,
        flags: i32,
    },
    RemoveVehicleFlags {
        object_id: Uuid,
        flags: i32,
    },
    SetStatus {
        object_id: Uuid,
        status: i32,
        value: bool,
    },
    TriggerSound {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    PlaySound {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    LoopSound {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    StopSound {
        object_id: Uuid,
    },
    StartAnimation {
        avatar_id: Uuid,
        anim_name: String,
    },
    StopAnimation {
        avatar_id: Uuid,
        anim_name: String,
    },
    SetSitTarget {
        object_id: Uuid,
        position: [f32; 3],
        rotation: [f32; 4],
    },
    UnSit {
        avatar_id: Uuid,
    },
    SetCameraParams {
        avatar_id: Uuid,
        object_id: Uuid,
        params: Vec<(i32, f32)>,
    },
    SetAlpha {
        object_id: Uuid,
        alpha: f32,
        face: i32,
    },
    SetColor {
        object_id: Uuid,
        color: [f32; 3],
        face: i32,
    },
    SetTexture {
        object_id: Uuid,
        texture_id: Uuid,
        face: i32,
    },
    ScaleTexture {
        object_id: Uuid,
        u: f32,
        v: f32,
        face: i32,
    },
    OffsetTexture {
        object_id: Uuid,
        u: f32,
        v: f32,
        face: i32,
    },
    RotateTexture {
        object_id: Uuid,
        rotation: f32,
        face: i32,
    },
    SetTextureAnim {
        object_id: Uuid,
        mode: i32,
        face: i32,
        size_x: i32,
        size_y: i32,
        start: f32,
        length: f32,
        rate: f32,
    },
    SetLinkColor {
        object_id: Uuid,
        link_num: i32,
        color: [f32; 3],
        face: i32,
    },
    SetLinkAlpha {
        object_id: Uuid,
        link_num: i32,
        alpha: f32,
        face: i32,
    },
    InstantMessage {
        target_id: Uuid,
        message: String,
        object_id: Uuid,
        object_name: String,
    },
    SetLinkPrimParams {
        object_id: Uuid,
        link_num: i32,
        rules: Vec<(i32, bool)>,
    },
    SetPrimParams {
        object_id: Uuid,
        link_num: i32,
        rules: Vec<PrimParamRule>,
    },
    ParticleSystem {
        object_id: Uuid,
        rules: Vec<u8>,
    },
    HttpRequest {
        object_id: Uuid,
        script_id: Uuid,
        url: String,
        params: Vec<(String, String)>,
        body: String,
    },
    SensorRequest {
        object_id: Uuid,
        script_id: Uuid,
        name: String,
        key: Uuid,
        sensor_type: i32,
        range: f64,
        arc: f64,
        repeat: bool,
        interval: f64,
    },
    SensorRemove {
        object_id: Uuid,
    },
    SetBuoyancy {
        object_id: Uuid,
        buoyancy: f32,
    },
    TextBox {
        avatar_id: Uuid,
        object_name: String,
        message: String,
        channel: i32,
        object_id: Uuid,
    },
    SetSitText {
        object_id: Uuid,
        text: String,
    },
    SetTouchText {
        object_id: Uuid,
        text: String,
    },
    SetOmega {
        object_id: Uuid,
        link_num: i32,
        axis: [f32; 3],
        spinrate: f64,
        gain: f64,
    },
    GiveInventory {
        prim_id: Uuid,
        destination_id: Uuid,
        item_name: String,
        owner_id: Uuid,
    },
    RezObject {
        prim_id: Uuid,
        item_name: String,
        position: [f32; 3],
        velocity: [f32; 3],
        rotation: [f32; 4],
        start_param: i32,
        at_root: bool,
        owner_id: Uuid,
    },
    AddPosTarget {
        object_id: Uuid,
        handle: i32,
        position: [f32; 3],
        range: f32,
    },
    RemovePosTarget {
        object_id: Uuid,
        handle: i32,
    },
    AddRotTarget {
        object_id: Uuid,
        handle: i32,
        rotation: [f32; 4],
        error: f32,
    },
    RemoveRotTarget {
        object_id: Uuid,
        handle: i32,
    },
    DataserverReply {
        script_id: Uuid,
        query_id: String,
        data: String,
    },
    SetClickAction {
        object_id: Uuid,
        action: u8,
    },
    SetVolumeDetect {
        object_id: Uuid,
        detect: bool,
    },
    SetCollisionSound {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    SetDamage {
        object_id: Uuid,
        damage: f32,
    },
    SetScriptAccessPin {
        object_id: Uuid,
        pin: i32,
    },
    SetAllowInventoryDrop {
        object_id: Uuid,
        allow: bool,
    },
    SetPassTouches {
        object_id: Uuid,
        pass: bool,
    },
    SetPassCollisions {
        object_id: Uuid,
        pass: bool,
    },
    SetSoundRadius {
        object_id: Uuid,
        radius: f32,
    },
    SetSoundQueueing {
        object_id: Uuid,
        queueing: bool,
    },
    AdjustSoundVolume {
        object_id: Uuid,
        volume: f32,
    },
    SetPayPrice {
        object_id: Uuid,
        prices: [i32; 5],
    },
    SetCameraAtOffset {
        object_id: Uuid,
        offset: [f32; 3],
    },
    SetCameraEyeOffset {
        object_id: Uuid,
        offset: [f32; 3],
    },
    ClearCameraParams {
        avatar_id: Uuid,
    },
    SetLinkCamera {
        object_id: Uuid,
        link_num: i32,
        eye: [f32; 3],
        at: [f32; 3],
    },
    PreloadSound {
        object_id: Uuid,
        sound_id: Uuid,
    },
    TriggerSoundLimited {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
        top_ne: [f32; 3],
        bot_sw: [f32; 3],
    },
    LoopSoundMaster {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    LoopSoundSlave {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    PlaySoundSlave {
        object_id: Uuid,
        sound_id: Uuid,
        volume: f32,
    },
    LinkPlaySound {
        object_id: Uuid,
        link_num: i32,
        sound_id: Uuid,
        volume: f32,
        flags: i32,
    },
    LinkStopSound {
        object_id: Uuid,
        link_num: i32,
    },
    LinkAdjustSoundVolume {
        object_id: Uuid,
        link_num: i32,
        volume: f32,
    },
    SetVelocity {
        object_id: Uuid,
        velocity: [f32; 3],
        local: bool,
    },
    SetAngularVelocity {
        object_id: Uuid,
        velocity: [f32; 3],
        local: bool,
    },
    SetForce {
        object_id: Uuid,
        force: [f32; 3],
        local: bool,
    },
    SetTorque {
        object_id: Uuid,
        torque: [f32; 3],
        local: bool,
    },
    SetForceAndTorque {
        object_id: Uuid,
        force: [f32; 3],
        torque: [f32; 3],
        local: bool,
    },
    ApplyRotationalImpulse {
        object_id: Uuid,
        impulse: [f32; 3],
        local: bool,
    },
    SetHoverHeight {
        object_id: Uuid,
        height: f32,
        water: i32,
        tau: f32,
    },
    StopHover {
        object_id: Uuid,
    },
    MoveToTarget {
        object_id: Uuid,
        target: [f32; 3],
        tau: f32,
    },
    StopMoveToTarget {
        object_id: Uuid,
    },
    PushObject {
        target_id: Uuid,
        impulse: [f32; 3],
        angular_impulse: [f32; 3],
        local: bool,
    },
    LookAt {
        object_id: Uuid,
        target: [f32; 3],
        strength: f32,
        damping: f32,
    },
    StopLookAt {
        object_id: Uuid,
    },
    RotLookAt {
        object_id: Uuid,
        rotation: [f32; 4],
        strength: f32,
        damping: f32,
    },
    GroundRepel {
        object_id: Uuid,
        height: f32,
        water: i32,
        tau: f32,
    },
    SetPhysicsMaterial {
        object_id: Uuid,
        gravity: f32,
        restitution: f32,
        friction: f32,
        density: f32,
        flags: i32,
    },
    StartObjectAnimation {
        object_id: Uuid,
        anim_name: String,
    },
    StopObjectAnimation {
        object_id: Uuid,
        anim_name: String,
    },
    RemoveInventory {
        object_id: Uuid,
        script_id: Uuid,
        item_name: String,
    },
    ResetOtherScript {
        object_id: Uuid,
        script_name: String,
    },
    SetScriptState {
        object_id: Uuid,
        script_name: String,
        running: bool,
    },
    DerezObject {
        object_id: Uuid,
        target_id: Uuid,
    },
    LoadURL {
        avatar_id: Uuid,
        message: String,
        url: String,
        object_name: String,
    },
    MapDestination {
        avatar_id: Uuid,
        sim_name: String,
        position: [f32; 3],
        look_at: [f32; 3],
    },
    ScaleByFactor {
        object_id: Uuid,
        factor: f64,
    },
    KeyframedMotion {
        object_id: Uuid,
        keyframes: Vec<f32>,
        mode: i32,
        data: i32,
    },
    SetLinkSitFlags {
        object_id: Uuid,
        flags: i32,
    },
    LinksetDataReset {
        object_id: Uuid,
    },
    NotecardRead {
        object_id: Uuid,
        script_id: Uuid,
        notecard_name: String,
        line: i32,
        query_id: String,
    },
    NotecardLineCount {
        object_id: Uuid,
        script_id: Uuid,
        notecard_name: String,
        query_id: String,
    },
    RequestUserData {
        script_id: Uuid,
        agent_id: Uuid,
        query_id: String,
        data_type: i32,
    },
    RequestSimulatorData {
        script_id: Uuid,
        sim_name: String,
        data_type: i32,
        query_id: String,
    },
    SetParcelMusicURL {
        object_id: Uuid,
        url: String,
    },
    AddToLandBanList {
        object_id: Uuid,
        agent_id: Uuid,
        hours: f32,
        is_ban: bool,
    },
    RemoveFromLandBanList {
        object_id: Uuid,
        agent_id: Uuid,
        is_ban: bool,
    },
    ResetLandBanList {
        object_id: Uuid,
        is_ban: bool,
    },
    EjectFromLand {
        object_id: Uuid,
        agent_id: Uuid,
    },
    ManageEstateAccess {
        object_id: Uuid,
        action: i32,
        agent_id: Uuid,
    },
    GiveMoney {
        owner_id: Uuid,
        destination_id: Uuid,
        amount: i32,
        object_id: Uuid,
    },
    TeleportAgent {
        agent_id: Uuid,
        landmark: String,
        position: [f32; 3],
        look_at: [f32; 3],
    },
    TeleportAgentHome {
        avatar_id: Uuid,
    },
    SetObjectPermMask {
        object_id: Uuid,
        mask_type: i32,
        mask_value: u32,
    },
    CreateLink {
        object_id: Uuid,
        target_id: Uuid,
        parent: bool,
    },
    BreakLink {
        object_id: Uuid,
        link_num: i32,
    },
    BreakAllLinks {
        object_id: Uuid,
    },
    ModifyLand {
        object_id: Uuid,
        action: i32,
        brush_size: i32,
    },
    SetProjectionParams {
        object_id: Uuid,
        enabled: bool,
        texture_id: Uuid,
        fov: f32,
        focus: f32,
        ambiance: f32,
    },
    CollisionSprite {
        object_id: Uuid,
    },
    AttachToAvatar {
        object_id: Uuid,
        attach_point: i32,
    },
    DetachFromAvatar {
        object_id: Uuid,
    },
    Email {
        object_id: Uuid,
        address: String,
        subject: String,
        message: String,
    },
    RequestURL {
        object_id: Uuid,
        script_id: Uuid,
        request_id: Uuid,
        secure: bool,
    },
    ReleaseURL {
        url: String,
    },
    HTTPInResponse {
        request_id: String,
        status: i32,
        body: String,
        content_type: String,
    },
    GetAgentList {
        object_id: Uuid,
        scope: i32,
    },
    SetObjectName {
        object_id: Uuid,
        name: String,
    },
    SetObjectDesc {
        object_id: Uuid,
        desc: String,
    },
    SetAnimationOverride {
        avatar_id: Uuid,
        anim_state: String,
        anim_name: String,
    },
    ResetAnimationOverride {
        avatar_id: Uuid,
        anim_state: String,
    },
    SetLinkTexture {
        object_id: Uuid,
        link_num: i32,
        face: i32,
        texture_id: Uuid,
    },
    SetLinkTextureAnim {
        object_id: Uuid,
        link_num: i32,
        mode: i32,
        face: i32,
        sizex: i32,
        sizey: i32,
        start: f32,
        length: f32,
        rate: f32,
    },
    TargetOmega {
        object_id: Uuid,
        axis: [f32; 3],
        spinrate: f32,
        gain: f32,
    },
    CollisionFilter {
        object_id: Uuid,
        name: String,
        id: Uuid,
        accept: bool,
    },
    GiveInventoryList {
        object_id: Uuid,
        destination_id: Uuid,
        folder_name: String,
        items: Vec<String>,
    },
    SetInventoryPermMask {
        object_id: Uuid,
        item_name: String,
        mask_type: i32,
        mask_value: u32,
    },
    SetPrimMediaParams {
        object_id: Uuid,
        face: i32,
        params: Vec<(i32, String)>,
    },
    ClearPrimMedia {
        object_id: Uuid,
        face: i32,
    },
    ParcelMediaCommandList {
        object_id: Uuid,
        commands: Vec<i32>,
    },
    ListenControl {
        object_id: Uuid,
        handle: i32,
        active: bool,
    },
    ScriptProfiler {
        object_id: Uuid,
        flags: i32,
    },
    ReturnObjectsByID {
        object_id: Uuid,
        ids: Vec<Uuid>,
    },
    ReturnObjectsByOwner {
        object_id: Uuid,
        owner_id: Uuid,
    },
    KickAvatar {
        avatar_id: Uuid,
        message: String,
    },
    SetRegionWaterHeight {
        height: f32,
    },
    RegionNotice {
        message: String,
    },
    RegionRestart {
        seconds: i32,
        message: String,
    },
    CastRay {
        object_id: Uuid,
        start: [f32; 3],
        end: [f32; 3],
        reject_types: i32,
        max_hits: i32,
    },
    NpcCreate {
        first_name: String,
        last_name: String,
        position: [f32; 3],
        notecard: String,
        options: i32,
    },
    NpcRemove {
        npc_id: Uuid,
    },
    NpcSay {
        npc_id: Uuid,
        channel: i32,
        message: String,
    },
    NpcMoveTo {
        npc_id: Uuid,
        position: [f32; 3],
        options: i32,
    },
    NpcSetRot {
        npc_id: Uuid,
        rotation: [f32; 4],
    },
    NpcPlayAnimation {
        npc_id: Uuid,
        anim_name: String,
    },
    NpcStopAnimation {
        npc_id: Uuid,
        anim_name: String,
    },
    NpcShout {
        npc_id: Uuid,
        channel: i32,
        message: String,
    },
    NpcWhisper {
        npc_id: Uuid,
        channel: i32,
        message: String,
    },
    NpcTouch {
        npc_id: Uuid,
        target_id: Uuid,
        link_num: i32,
    },
    NpcSit {
        npc_id: Uuid,
        target_id: Uuid,
    },
    NpcStand {
        npc_id: Uuid,
    },
    NpcLoadAppearance {
        npc_id: Uuid,
        notecard: String,
    },
    NpcSaveAppearance {
        npc_id: Uuid,
        notecard: String,
    },
    AgentSaveAppearance {
        avatar_id: Uuid,
        notecard: String,
    },
    ForceAttachToOtherAvatar {
        object_id: Uuid,
        avatar_id: Uuid,
        attach_point: i32,
    },
    AvatarPlayAnimation {
        avatar_id: Uuid,
        anim_name: String,
    },
    AvatarStopAnimation {
        avatar_id: Uuid,
        anim_name: String,
    },
    MessageObject {
        target_id: Uuid,
        message: String,
    },
    MakeNotecard {
        name: String,
        contents: String,
        object_id: Uuid,
    },
    SetTerrainHeight {
        x: i32,
        y: i32,
        height: f32,
    },
    SetTerrainTexture {
        corner: i32,
        texture_id: Uuid,
    },
    SetTerrainTextureHeight {
        corner: i32,
        low: f32,
        high: f32,
    },
    SetSunParam {
        param: String,
        value: f32,
    },
    SetWindParam {
        param: String,
        value: f32,
    },
    ForceAttachToAvatar {
        object_id: Uuid,
        avatar_id: Uuid,
        attach_point: i32,
    },
    ForceDetachFromAvatar {
        object_id: Uuid,
        avatar_id: Uuid,
    },
    SetSpeed {
        avatar_id: Uuid,
        speed: f32,
    },
    ForceOtherSit {
        avatar_id: Uuid,
        target_id: Uuid,
    },
    SetParcelDetails {
        position: [f32; 3],
        params: Vec<(i32, String)>,
    },
    CreateStatue {
        owner_id: Uuid,
        anim_name: String,
        frame: i32,
        total_frames: i32,
        position: [f32; 3],
        name: String,
    },
    CreateSnapshotStatue {
        owner_id: Uuid,
        anim_name: String,
        frame: i32,
        position: [f32; 3],
        name: String,
    },
    InvokeSkill {
        object_id: Uuid,
        script_id: Uuid,
        domain: String,
        skill_id: String,
        params_json: String,
    },
}

pub struct ScriptInstance {
    pub script_id: Uuid,
    pub current_state: String,
    pub global_vars: HashMap<String, LSLValue>,
    pub heap_used: usize,
    pub heap_limit: usize,
    pub compiled: CompiledScript,
    pub running: bool,
    pub min_event_delay: f64,
    pub pending_actions: Vec<ScriptAction>,
    pub context: ObjectContext,
}

impl ScriptInstance {
    pub fn new(script_id: Uuid, compiled: CompiledScript, heap_limit: usize) -> Self {
        let mut global_vars = HashMap::new();
        for (name, type_name, init_value) in &compiled.globals {
            let val = match init_value {
                Some(crate::scripting::lsl_interpreter::ASTNode::Literal(v)) => v.clone(),
                Some(crate::scripting::lsl_interpreter::ASTNode::UnaryOp { operator, operand }) => {
                    if let (
                        crate::scripting::lsl_interpreter::Token::Minus,
                        crate::scripting::lsl_interpreter::ASTNode::Literal(v),
                    ) = (operator, operand.as_ref())
                    {
                        match v {
                            LSLValue::Integer(i) => LSLValue::Integer(-i),
                            LSLValue::Float(f) => LSLValue::Float(-f),
                            _ => v.clone(),
                        }
                    } else {
                        LSLValue::type_default(type_name)
                    }
                }
                _ => LSLValue::type_default(type_name),
            };
            global_vars.insert(name.clone(), val);
        }

        Self {
            script_id,
            current_state: "default".to_string(),
            global_vars,
            heap_used: 0,
            heap_limit,
            compiled,
            running: true,
            min_event_delay: 0.0,
            pending_actions: Vec::new(),
            context: ObjectContext::default(),
        }
    }

    pub fn track_heap(&mut self, value: &LSLValue) -> Result<()> {
        let size = value.heap_size();
        if self.heap_used + size > self.heap_limit {
            return Err(anyhow::anyhow!(
                "Script heap limit exceeded ({}/{})",
                self.heap_used + size,
                self.heap_limit
            ));
        }
        self.heap_used += size;
        Ok(())
    }
}

pub trait ScriptExecutor: Send + Sync {
    fn name(&self) -> &'static str;

    fn compile(&self, source: &str, script_id: Uuid) -> Result<CompiledScript>;

    fn execute_event(
        &self,
        instance: &mut ScriptInstance,
        event: &str,
        args: &[LSLValue],
    ) -> Result<ExecutionResult>;
}
