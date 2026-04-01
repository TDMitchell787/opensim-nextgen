# OpenSim Next — Implemented LSL Functions

**LSL functions**: 315 | **OSSL functions**: 74 | **Combined total**: 389
**Files**: `rust/src/scripting/lsl_functions.rs`, `rust/src/scripting/ossl_functions.rs`
**Last updated**: 2026-02-26 (Phase 145)

## Status Legend

| Status | Meaning |
|--------|---------|
| **Action** | Pushes ScriptAction to server — full server-side handler exists |
| **Compute** | Pure computation — returns correct value from args/context |
| **Chat** | Uses chat/listener system — fully functional |
| **Stub** | Logs call and returns default — no server interaction yet |

---

## Chat and Communication (8)

| Function | Status | Notes |
|----------|--------|-------|
| llSay | Chat | channel + message via chat system |
| llShout | Chat | 100m range chat |
| llWhisper | Chat | 10m range chat |
| llOwnerSay | Chat | owner-only message |
| llRegionSay | Chat | region-wide on non-zero channel |
| llListen | Chat | registers listener, returns handle |
| llListenControl | Stub | |
| llListenRemove | Action | ListenRemove |

## Object Manipulation (7)

| Function | Status | Notes |
|----------|--------|-------|
| llSetText | Action | SetText — hover text + color + alpha |
| llSetObjectName | Stub | |
| llSetObjectDesc | Stub | |
| llGetObjectName | Compute | reads context.object_name |
| llGetObjectDesc | Compute | reads context.object_description |
| llGetKey | Compute | reads context.object_id |
| llGetOwner | Compute | reads context.owner_id |

## Position and Movement (8)

| Function | Status | Notes |
|----------|--------|-------|
| llGetPos | Compute | reads context.position |
| llSetPos | Action | SetPos |
| llGetRot | Compute | reads context.rotation |
| llSetRot | Action | SetRot |
| llGetVel | Compute | reads context.velocity |
| llSetVelocity | Stub | |
| llGetRegionCorner | Compute | context.region_corner |
| llGetRegionName | Compute | context.region_name |

## Math Functions (16)

| Function | Status | Notes |
|----------|--------|-------|
| llAbs | Compute | |
| llAcos | Compute | |
| llAsin | Compute | |
| llAtan2 | Compute | |
| llCeil | Compute | |
| llCos | Compute | |
| llFabs | Compute | |
| llFloor | Compute | |
| llFrand | Compute | random float |
| llLog | Compute | |
| llLog10 | Compute | |
| llPow | Compute | |
| llRound | Compute | |
| llSin | Compute | |
| llSqrt | Compute | |
| llTan | Compute | |

## Vector and Rotation Functions (9)

| Function | Status | Notes |
|----------|--------|-------|
| llVecDist | Compute | |
| llVecMag | Compute | |
| llVecNorm | Compute | |
| llAngleBetween | Compute | |
| llEuler2Rot | Compute | |
| llRot2Euler | Compute | |
| llAxes2Rot | Compute | |
| llRot2Axis | Compute | |
| llRot2Angle | Compute | |

## String Functions (7)

| Function | Status | Notes |
|----------|--------|-------|
| llStringLength | Compute | |
| llSubStringIndex | Compute | |
| llGetSubString | Compute | |
| llInsertString | Compute | |
| llDeleteSubString | Compute | |
| llToLower | Compute | |
| llToUpper | Compute | |

## List Functions (10)

| Function | Status | Notes |
|----------|--------|-------|
| llListLength | Compute | |
| llList2String | Compute | |
| llList2Integer | Compute | |
| llList2Float | Compute | |
| llList2Key | Compute | |
| llList2Vector | Compute | |
| llList2Rot | Compute | |
| llListInsertList | Compute | |
| llDeleteSubList | Compute | |
| llGetListLength | Compute | |

## Type Conversion (4)

| Function | Status | Notes |
|----------|--------|-------|
| llList2CSV | Compute | |
| llCSV2List | Compute | |
| llDumpList2String | Compute | |
| llParseString2List | Compute | |

## Time Functions (3 + 7 additional)

| Function | Status | Notes |
|----------|--------|-------|
| llGetTimestamp | Compute | ISO 8601 timestamp |
| llGetUnixTime | Compute | seconds since epoch |
| llSetTimerEvent | Action | SetTimerEvent |
| llGetTime | Compute | script elapsed time |
| llGetAndResetTime | Compute | get + reset timer |
| llResetTime | Compute | reset script timer |
| llGetDate | Compute | YYYY-MM-DD |
| llGetGMTclock | Compute | seconds since midnight |
| llGetWallclock | Compute | |
| llGetTimeOfDay | Compute | |

## Inventory Functions (4 + 11 additional)

| Function | Status | Notes |
|----------|--------|-------|
| llGetInventoryNumber | Stub | |
| llGetInventoryName | Stub | |
| llGetInventoryType | Stub | |
| llGetInventoryKey | Stub | |
| llGiveInventory | Stub | |
| llGiveInventoryList | Stub | |
| llRemoveInventory | Stub | |
| llGetInventoryCreator | Stub | |
| llGetInventoryPermMask | Stub | |
| llSetInventoryPermMask | Stub | |
| llGetInventoryDesc | Stub | |
| llGetInventoryAcquireTime | Stub | |
| llRequestInventoryData | Stub | |
| llGetObjectPermMask | Stub | |
| llSetObjectPermMask | Stub | |

## HTTP Functions (2 + 6 additional)

| Function | Status | Notes |
|----------|--------|-------|
| llHTTPRequest | Stub | |
| llHTTPResponse | Stub | |
| llRequestURL | Stub | |
| llRequestSecureURL | Stub | |
| llReleaseURL | Stub | |
| llGetFreeURLs | Stub | |
| llGetHTTPHeader | Stub | |
| llSetContentType | Stub | |

## Sensor Functions (3)

| Function | Status | Notes |
|----------|--------|-------|
| llSensor | Stub | |
| llSensorRepeat | Stub | |
| llSensorRemove | Stub | |

## Debug and Utility (3)

| Function | Status | Notes |
|----------|--------|-------|
| llGetScriptName | Compute | reads context.script_name |
| llResetScript | Action | ResetScript |
| llSleep | Action | Sleep |

## Physics and Movement (23)

| Function | Status | Notes |
|----------|--------|-------|
| llApplyImpulse | Action | ApplyImpulse |
| llApplyRotationalImpulse | Stub | |
| llSetForce | Stub | |
| llGetForce | Stub | |
| llSetTorque | Stub | |
| llGetTorque | Stub | |
| llSetBuoyancy | Stub | |
| llSetHoverHeight | Stub | |
| llStopHover | Stub | |
| llMoveToTarget | Stub | |
| llStopMoveToTarget | Stub | |
| llPushObject | Stub | |
| llGetAccel | Stub | |
| llGetOmega | Stub | |
| llTargetOmega | Stub | |
| llSetAngularVelocity | Stub | |
| llGetMass | Stub | returns 1.0 |
| llGetMassMKS | Stub | returns 1.0 |
| llGetObjectMass | Stub | returns 1.0 |
| llGroundRepel | Stub | |
| llSetForceAndTorque | Stub | |
| llSetPhysicsMaterial | Stub | |
| llGetPhysicsMaterial | Stub | |

## Animation Functions (10)

| Function | Status | Notes |
|----------|--------|-------|
| llStartAnimation | Action | StartAnimation — broadcasts to all nearby |
| llStopAnimation | Action | StopAnimation — broadcasts seq=-1 to all |
| llGetAnimation | Stub | |
| llGetAnimationList | Stub | |
| llSetAnimationOverride | Stub | |
| llGetAnimationOverride | Stub | |
| llResetAnimationOverride | Stub | |
| llStartObjectAnimation | Stub | |
| llStopObjectAnimation | Stub | |
| llGetObjectAnimationNames | Stub | |

## Sound Functions (17 + 2)

| Function | Status | Notes |
|----------|--------|-------|
| llPlaySound | Action | PlaySound |
| llLoopSound | Action | LoopSound |
| llLoopSoundMaster | Stub | |
| llLoopSoundSlave | Stub | |
| llPlaySoundSlave | Stub | |
| llStopSound | Action | StopSound |
| llTriggerSound | Action | TriggerSound |
| llTriggerSoundLimited | Stub | |
| llPreloadSound | Action | TriggerSound vol=0 (Phase 144) |
| llSound | Stub | deprecated |
| llSoundPreload | Stub | deprecated |
| llAdjustSoundVolume | Stub | |
| llSetSoundQueueing | Stub | |
| llSetSoundRadius | Stub | |
| llLinkPlaySound | Stub | |
| llLinkStopSound | Stub | |
| llLinkAdjustSoundVolume | Stub | |
| llLinkSetSoundQueueing | Stub | |
| llLinkSetSoundRadius | Stub | |
| llCollisionSound | Action | TriggerSound (Phase 144) |

## Texture and Visual Functions (19)

| Function | Status | Notes |
|----------|--------|-------|
| llSetTexture | Stub | |
| llGetTexture | Stub | |
| llSetColor | Stub | |
| llGetColor | Stub | |
| llSetAlpha | Action | SetAlpha |
| llGetAlpha | Stub | |
| llSetScale | Action | SetScale |
| llGetScale | Compute | reads context.scale |
| llScaleTexture | Stub | |
| llOffsetTexture | Stub | |
| llRotateTexture | Stub | |
| llGetTextureOffset | Stub | |
| llGetTextureScale | Stub | |
| llGetTextureRot | Stub | |
| llSetTextureAnim | Stub | |
| llSetLinkTexture | Stub | |
| llSetLinkColor | Stub | |
| llSetLinkAlpha | Stub | |
| llSetLinkTextureAnim | Stub | |

## Primitive Parameters (7)

| Function | Status | Notes |
|----------|--------|-------|
| llSetPrimitiveParams | Action | handles PRIM_OMEGA → SetOmega (Phase 144) |
| llGetPrimitiveParams | Stub | |
| llSetLinkPrimitiveParams | Stub | |
| llSetLinkPrimitiveParamsFast | Stub | |
| llGetLinkPrimitiveParams | Stub | |
| llGetNumberOfSides | Stub | |
| llGetLinkNumberOfSides | Stub | |

## Detected Functions (16)

| Function | Status | Notes |
|----------|--------|-------|
| llDetectedKey | Compute | reads detect_params |
| llDetectedName | Compute | reads detect_params |
| llDetectedOwner | Compute | reads detect_params |
| llDetectedType | Compute | reads detect_params |
| llDetectedPos | Compute | reads detect_params |
| llDetectedVel | Compute | reads detect_params |
| llDetectedRot | Compute | reads detect_params |
| llDetectedGroup | Compute | reads detect_params |
| llDetectedLinkNumber | Compute | reads detect_params |
| llDetectedGrab | Stub | |
| llDetectedTouchFace | Stub | |
| llDetectedTouchPos | Stub | |
| llDetectedTouchNormal | Stub | |
| llDetectedTouchBinormal | Stub | |
| llDetectedTouchST | Stub | |
| llDetectedTouchUV | Stub | |

## Agent/Avatar Functions (21)

| Function | Status | Notes |
|----------|--------|-------|
| llGetAgentInfo | Stub | |
| llGetAgentSize | Stub | |
| llGetAgentLanguage | Stub | |
| llGetAgentList | Stub | |
| llRequestAgentData | Stub | |
| llKey2Name | Stub | |
| llName2Key | Stub | |
| llGetDisplayName | Stub | |
| llGetUsername | Stub | |
| llRequestDisplayName | Stub | |
| llRequestUsername | Stub | |
| llRequestUserKey | Stub | |
| llGetHealth | Stub | returns 100.0 |
| llGetEnergy | Stub | returns 1.0 |
| llTeleportAgent | Stub | |
| llTeleportAgentHome | Stub | |
| llTeleportAgentGlobalCoords | Stub | |
| llEjectFromLand | Stub | |
| llInstantMessage | Action | InstantMessage |
| llGiveMoney | Stub | |
| llTransferLindenDollars | Stub | |

## Permission Functions (12)

| Function | Status | Notes |
|----------|--------|-------|
| llRequestPermissions | Action | RequestPermissions |
| llGetPermissions | Compute | reads context.granted_perms |
| llGetPermissionsKey | Compute | reads context.perm_granter |
| llTakeControls | Action | TakeControls (Phase 144) |
| llReleaseControls | Action | ReleaseControls (Phase 144) |
| llTakeCamera | Stub | deprecated |
| llReleaseCamera | Stub | deprecated |
| llAttachToAvatar | Stub | |
| llAttachToAvatarTemp | Stub | |
| llDetachFromAvatar | Stub | |
| llGetAttached | Stub | |
| llGetAttachedList | Stub | |

## Camera Functions (9)

| Function | Status | Notes |
|----------|--------|-------|
| llSetCameraParams | Action | SetCameraParams (Phase 144) |
| llClearCameraParams | Action | SetCameraParams w/ CAMERA_ACTIVE=0 (Phase 144) |
| llSetCameraAtOffset | Stub | |
| llSetCameraEyeOffset | Stub | |
| llGetCameraPos | Stub | |
| llGetCameraRot | Stub | |
| llGetCameraAspect | Stub | |
| llGetCameraFOV | Stub | |
| llForceMouselook | Action | SetStatus 0x100 (Phase 144) |

## Dialog and UI Functions (8)

| Function | Status | Notes |
|----------|--------|-------|
| llDialog | Action | Dialog |
| llTextBox | Stub | |
| llMapDestination | Stub | |
| llLoadURL | Stub | |
| llSetPayPrice | Stub | |
| llSetClickAction | Stub | |
| llSetSitText | Action | SetSitText |
| llSetTouchText | Action | SetTouchText |

## Link Functions (12)

| Function | Status | Notes |
|----------|--------|-------|
| llGetLinkNumber | Compute | reads context.link_num |
| llGetLinkKey | Stub | |
| llGetLinkName | Compute | searches context.link_names (Phase 144) |
| llGetNumberOfPrims | Compute | reads context.link_count (Phase 144) |
| llGetObjectPrimCount | Stub | |
| llGetObjectLinkKey | Stub | |
| llCreateLink | Stub | |
| llBreakLink | Stub | |
| llBreakAllLinks | Stub | |
| llMessageLinked | Action | MessageLinked (Phase 144) |
| llSetLinkCamera | Stub | |
| llLinkSitTarget | Stub | |

## Sit Functions (6)

| Function | Status | Notes |
|----------|--------|-------|
| llSitTarget | Action | SetSitTarget (Phase 144) — Phase 145: sit target offset used for ParentID linkage |
| llAvatarOnSitTarget | Compute | returns context.sitting_avatar_id (Phase 144) |
| llAvatarOnLinkSitTarget | Compute | returns context.sitting_avatar_id (Phase 144) |
| llUnSit | Action | UnSit (Phase 144) — Phase 145: stand-up broadcasts ObjectUpdate parent_id=0 |
| llGetLinkSitFlags | Stub | |
| llSetLinkSitFlags | Stub | |

## Land and Parcel Functions (16)

| Function | Status | Notes |
|----------|--------|-------|
| llGetParcelFlags | Stub | |
| llGetParcelDetails | Stub | |
| llGetParcelMaxPrims | Stub | |
| llGetParcelPrimCount | Stub | |
| llGetParcelPrimOwners | Stub | |
| llGetParcelMusicURL | Stub | |
| llSetParcelMusicURL | Stub | |
| llGetLandOwnerAt | Stub | |
| llOverMyLand | Stub | |
| llModifyLand | Stub | |
| llAddToLandBanList | Stub | |
| llRemoveFromLandBanList | Stub | |
| llAddToLandPassList | Stub | |
| llRemoveFromLandPassList | Stub | |
| llResetLandBanList | Stub | |
| llResetLandPassList | Stub | |

## Vehicle Functions (6)

| Function | Status | Notes |
|----------|--------|-------|
| llSetVehicleType | Action | SetVehicleType (Phase 144) |
| llSetVehicleFlags | Action | SetVehicleFlags (Phase 144) |
| llRemoveVehicleFlags | Action | RemoveVehicleFlags (Phase 144) |
| llSetVehicleFloatParam | Action | SetVehicleFloatParam (Phase 144) |
| llSetVehicleVectorParam | Action | SetVehicleVectorParam (Phase 144) |
| llSetVehicleRotationParam | Action | SetVehicleRotationParam (Phase 144) |

## Region Functions (8)

| Function | Status | Notes |
|----------|--------|-------|
| llGetRegionFlags | Stub | |
| llGetRegionFPS | Stub | returns 45.0 |
| llGetRegionTimeDilation | Stub | returns 1.0 |
| llGetRegionAgentCount | Stub | |
| llRequestSimulatorData | Stub | |
| llGetSimulatorHostname | Stub | |
| llGetEnv | Stub | |
| llGetSimStats | Stub | |

## Ground/Terrain Functions (8)

| Function | Status | Notes |
|----------|--------|-------|
| llGround | Stub | |
| llGroundNormal | Stub | |
| llGroundSlope | Stub | |
| llGroundContour | Stub | |
| llWater | Stub | returns 20.0 |
| llCloud | Stub | returns 0.0 |
| llWind | Stub | |
| llEdgeOfWorld | Stub | |

## Sun/Moon Functions (12)

| Function | Status | Notes |
|----------|--------|-------|
| llGetSunDirection | Stub | |
| llGetSunRotation | Stub | |
| llGetMoonDirection | Stub | |
| llGetMoonRotation | Stub | |
| llGetRegionSunDirection | Stub | |
| llGetRegionSunRotation | Stub | |
| llGetRegionMoonDirection | Stub | |
| llGetRegionMoonRotation | Stub | |
| llGetDayLength | Stub | |
| llGetDayOffset | Stub | |
| llGetRegionDayLength | Stub | |
| llGetRegionDayOffset | Stub | |

## Object Functions (19)

| Function | Status | Notes |
|----------|--------|-------|
| llRezObject | Stub | |
| llRezAtRoot | Stub | |
| llRezObjectWithParams | Stub | |
| llDie | Action | Die |
| llDerezObject | Stub | |
| llGetCreator | Stub | |
| llGetOwnerKey | Stub | |
| llGetBoundingBox | Stub | |
| llGetGeometricCenter | Stub | |
| llGetCenterOfMass | Stub | |
| llGetObjectDetails | Stub | |
| llGetStatus | Stub | |
| llSetStatus | Stub | |
| llSetDamage | Stub | |
| llAllowInventoryDrop | Stub | |
| llPassTouches | Stub | |
| llPassCollisions | Stub | |
| llCollisionFilter | Stub | |
| llVolumeDetect | Stub | |

## Target Functions (4)

| Function | Status | Notes |
|----------|--------|-------|
| llTarget | Stub | |
| llTargetRemove | Stub | |
| llRotTarget | Stub | |
| llRotTargetRemove | Stub | |

## Look-at Functions (5)

| Function | Status | Notes |
|----------|--------|-------|
| llLookAt | Stub | |
| llStopLookAt | Stub | |
| llRotLookAt | Stub | |
| llPointAt | Stub | |
| llStopPointAt | Stub | |

## Particle Functions (6)

| Function | Status | Notes |
|----------|--------|-------|
| llParticleSystem | Stub | |
| llLinkParticleSystem | Stub | |
| llMakeExplosion | Stub | deprecated |
| llMakeFire | Stub | deprecated |
| llMakeFountain | Stub | deprecated |
| llMakeSmoke | Stub | deprecated |

## Email Functions (3)

| Function | Status | Notes |
|----------|--------|-------|
| llEmail | Stub | |
| llTargetedEmail | Stub | |
| llGetNextEmail | Stub | |

## XML-RPC Functions (5)

| Function | Status | Notes |
|----------|--------|-------|
| llOpenRemoteDataChannel | Stub | |
| llCloseRemoteDataChannel | Stub | |
| llSendRemoteData | Stub | |
| llRemoteDataReply | Stub | |
| llRemoteDataSetRegion | Stub | |

## JSON Functions (5)

| Function | Status | Notes |
|----------|--------|-------|
| llJson2List | Compute | |
| llList2Json | Compute | |
| llJsonGetValue | Compute | |
| llJsonSetValue | Compute | |
| llJsonValueType | Compute | |

## Linkset Data Functions (13)

| Function | Status | Notes |
|----------|--------|-------|
| llLinksetDataWrite | Stub | |
| llLinksetDataWriteProtected | Stub | |
| llLinksetDataRead | Stub | |
| llLinksetDataReadProtected | Stub | |
| llLinksetDataDelete | Stub | |
| llLinksetDataDeleteProtected | Stub | |
| llLinksetDataDeleteFound | Stub | |
| llLinksetDataCountKeys | Stub | |
| llLinksetDataCountFound | Stub | |
| llLinksetDataFindKeys | Stub | |
| llLinksetDataListKeys | Stub | |
| llLinksetDataAvailable | Stub | |
| llLinksetDataReset | Stub | |

## Hash/Crypto Functions (6)

| Function | Status | Notes |
|----------|--------|-------|
| llMD5String | Compute | |
| llSHA1String | Compute | |
| llSHA256String | Compute | |
| llHMAC | Compute | |
| llComputeHash | Compute | |
| llHash | Compute | |

## Base64 Functions (7)

| Function | Status | Notes |
|----------|--------|-------|
| llStringToBase64 | Compute | |
| llBase64ToString | Compute | |
| llIntegerToBase64 | Compute | |
| llBase64ToInteger | Compute | |
| llXorBase64 | Compute | |
| llXorBase64Strings | Compute | deprecated |
| llXorBase64StringsCorrect | Compute | |

## Character/String Conversion (6)

| Function | Status | Notes |
|----------|--------|-------|
| llChar | Compute | |
| llOrd | Compute | |
| llEscapeURL | Compute | |
| llUnescapeURL | Compute | |
| llStringTrim | Compute | |
| llReplaceSubString | Compute | |

## Additional List Functions (13)

| Function | Status | Notes |
|----------|--------|-------|
| llListStatistics | Compute | |
| llListSort | Compute | |
| llListSortStrided | Compute | |
| llListRandomize | Compute | |
| llListReplaceList | Compute | |
| llList2List | Compute | |
| llList2ListSlice | Compute | |
| llList2ListStrided | Compute | |
| llListFindList | Compute | |
| llListFindListNext | Compute | |
| llListFindStrided | Compute | |
| llGetListEntryType | Compute | |
| llParseStringKeepNulls | Compute | |

## Rotation Additional Functions (10)

| Function | Status | Notes |
|----------|--------|-------|
| llAxisAngle2Rot | Compute | |
| llRotBetween | Compute | |
| llRot2Fwd | Compute | |
| llRot2Left | Compute | |
| llRot2Up | Compute | |
| llGetLocalRot | Compute | |
| llSetLocalRot | Stub | |
| llGetRootRotation | Compute | |
| llGetLocalPos | Compute | |
| llGetRootPosition | Compute | |

## Notecard Functions (3)

| Function | Status | Notes |
|----------|--------|-------|
| llGetNotecardLine | Stub | |
| llGetNotecardLineSync | Stub | |
| llGetNumberOfNotecardLines | Stub | |

## Media Functions (10)

| Function | Status | Notes |
|----------|--------|-------|
| llSetPrimMediaParams | Stub | |
| llGetPrimMediaParams | Stub | |
| llClearPrimMedia | Stub | |
| llSetLinkMedia | Stub | |
| llGetLinkMedia | Stub | |
| llClearLinkMedia | Stub | |
| llParcelMediaCommandList | Stub | |
| llParcelMediaQuery | Stub | |
| llSetPrimURL | Stub | deprecated |
| llRefreshPrimURL | Stub | deprecated |

## Cast Ray (2)

| Function | Status | Notes |
|----------|--------|-------|
| llCastRay | Stub | |
| llCastRayV3 | Stub | |

## Color Space (2)

| Function | Status | Notes |
|----------|--------|-------|
| llLinear2sRGB | Compute | |
| llsRGB2Linear | Compute | |

## Script Control Functions (13)

| Function | Status | Notes |
|----------|--------|-------|
| llGetScriptState | Stub | |
| llSetScriptState | Stub | |
| llResetOtherScript | Stub | |
| llRemoteLoadScript | Stub | deprecated |
| llRemoteLoadScriptPin | Stub | |
| llSetRemoteScriptAccessPin | Stub | |
| llGetStartParameter | Stub | returns 0 |
| llMinEventDelay | Stub | |
| llScriptDanger | Stub | |
| llScriptProfiler | Stub | |
| llGetMemoryLimit | Stub | returns 65536 |
| llSetMemoryLimit | Stub | |
| llGetSPMaxMemory | Stub | |

## Key/Group Functions (3)

| Function | Status | Notes |
|----------|--------|-------|
| llGenerateKey | Compute | generates random UUID |
| llSameGroup | Stub | |
| llIsFriend | Stub | |

## Scale Functions (3)

| Function | Status | Notes |
|----------|--------|-------|
| llScaleByFactor | Stub | |
| llGetMaxScaleFactor | Stub | |
| llGetMinScaleFactor | Stub | |

## Miscellaneous (5)

| Function | Status | Notes |
|----------|--------|-------|
| llManageEstateAccess | Stub | |
| llSetRegionPos | Stub | |
| llSetKeyframedMotion | Stub | |
| llCollisionSprite | Stub | deprecated |
| llGodLikeRezObject | Stub | internal |
| llGetVisualParams | Stub | |
| llModPow | Compute | modular exponentiation |

---

## Summary Statistics

| Status | Count | Percentage |
|--------|-------|------------|
| **Action** | 38 | ~12% |
| **Compute** | 110 | ~35% |
| **Chat** | 6 | ~2% |
| **Stub** | 161 | ~51% |
| **Total** | 315 | 100% |

### Action Functions (push ScriptAction to server) — 38 total

| Function | ScriptAction | Added |
|----------|-------------|-------|
| llSetPos | SetPos | Pre-144 |
| llSetRot | SetRot | Pre-144 |
| llSetText | SetText | Pre-144 |
| llSetScale | SetScale | Pre-144 |
| llSetAlpha | SetAlpha | Pre-144 |
| llApplyImpulse | ApplyImpulse | Pre-144 |
| llDie | Die | Pre-144 |
| llRequestPermissions | RequestPermissions | Pre-144 |
| llDialog | Dialog | Pre-144 |
| llSleep | Sleep | Pre-144 |
| llResetScript | ResetScript | Pre-144 |
| llSetTimerEvent | SetTimerEvent | Pre-144 |
| llListenRemove | ListenRemove | Pre-144 |
| llInstantMessage | InstantMessage | Pre-144 |
| llStartAnimation | StartAnimation | Pre-144 |
| llStopAnimation | StopAnimation | Pre-144 |
| llPlaySound | PlaySound | Pre-144 |
| llTriggerSound | TriggerSound | Pre-144 |
| llLoopSound | LoopSound | Pre-144 |
| llStopSound | StopSound | Pre-144 |
| llSetSitText | SetSitText | Pre-144 |
| llSetTouchText | SetTouchText | Pre-144 |
| llSetPrimitiveParams | SetOmega (PRIM_OMEGA) | Phase 144 |
| llTakeControls | TakeControls | Phase 144 |
| llReleaseControls | ReleaseControls | Phase 144 |
| llSetCameraParams | SetCameraParams | Phase 144 |
| llClearCameraParams | SetCameraParams (ACTIVE=0) | Phase 144 |
| llForceMouselook | SetStatus (0x100) | Phase 144 |
| llMessageLinked | MessageLinked | Phase 144 |
| llSitTarget | SetSitTarget | Phase 144 |
| llUnSit | UnSit | Phase 144 |
| llPreloadSound | TriggerSound (vol=0) | Phase 144 |
| llCollisionSound | TriggerSound | Phase 144 |
| llSetVehicleType | SetVehicleType | Phase 144 |
| llSetVehicleFlags | SetVehicleFlags | Phase 144 |
| llRemoveVehicleFlags | RemoveVehicleFlags | Phase 144 |
| llSetVehicleFloatParam | SetVehicleFloatParam | Phase 144 |
| llSetVehicleVectorParam | SetVehicleVectorParam | Phase 144 |
| llSetVehicleRotationParam | SetVehicleRotationParam | Phase 144 |

**Phase 144 added 16 new Action functions** (was 22, now 38).

### Phase 145 — Avatar Sit/Ride ParentID Fix (no new LSL functions)

Phase 145 fixed the server-side sit/stand behavior that the sit-related LSL functions depend on:
- `llSitTarget` offset is now used as the actual sit position (was hardcoded 0,0,0.4)
- Avatar ObjectUpdate broadcasts `parent_id = object.local_id` so viewer treats position as relative
- Stand-up computes absolute world position and broadcasts `parent_id = 0`
- Avatar now moves with the object it's sitting on (vehicles, rotating platforms, etc.)

---

# OpenSim Next — Implemented OSSL Functions

**Total registered functions**: 67
**File**: `rust/src/scripting/ossl_functions.rs`
**Last updated**: 2026-02-26 (Phase 145)

## Threat Level System

All OSSL functions are gated by a threat level. The region's `max_threat_level` setting determines which functions scripts may call. Functions above the threshold return an error.

| Threat Level | Value | Description |
|-------------|-------|-------------|
| None | 0 | Safe — read-only info, drawing commands |
| Nuisance | 1 | Minor annoyance potential |
| VeryLow | 2 | Minimal risk (default for unknown functions) |
| Low | 3 | Terrain changes, notecard access |
| Moderate | 4 | Speed changes, projection params |
| High | 5 | NPC control, teleport, attachments, parcel admin |
| VeryHigh | 6 | Reserved |
| Severe | 7 | Admin-only — kick, restart, console |

## Status Legend

| Status | Meaning |
|--------|---------|
| **Stub** | Registered, enforces threat level, parses args, returns default value |
| **Compute** | Returns meaningful computed value (drawing string builders, region info) |

All 67 OSSL functions are stub-level (no server-side effects). Drawing functions build command strings but don't render textures yet.

---

## Simulator Info (7)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osGetTerrainHeight | None | Stub | returns 21.0 |
| osGetRegionSize | None | Compute | returns <256, 256, 0> |
| osGetSimulatorVersion | None | Compute | returns "OpenSim Next (YEngine/Rust)" |
| osGetScriptEngineName | None | Compute | returns "YEngine" |
| osGetSimulatorMemory | None | Compute | returns 512MB |
| osGetSimulatorMemoryKB | None | Compute | returns 512*1024 |
| osGetPhysicsEngineType | None | Compute | returns "ODE" |

## NPC Functions (21)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osIsNpc | None | Stub | always returns 0 |
| osNpcCreate | High | Stub | returns random UUID |
| osNpcRemove | High | Stub | logs call |
| osNpcMoveTo | High | Stub | |
| osNpcMoveToTarget | High | Stub | |
| osNpcSay | High | Stub | |
| osNpcShout | High | Stub | |
| osNpcWhisper | High | Stub | |
| osNpcSit | High | Stub | |
| osNpcStand | High | Stub | |
| osNpcPlayAnimation | High | Stub | |
| osNpcStopAnimation | High | Stub | |
| osNpcGetPos | None | Stub | returns ZERO_VECTOR |
| osNpcGetRot | None | Stub | returns ZERO_ROTATION |
| osNpcSetRot | High | Stub | |
| osNpcTouch | High | Stub | |
| osNpcLoadAppearance | High | Stub | |
| osNpcSaveAppearance | High | Stub | returns random UUID |
| osNpcSetProfileAbout | High | Stub | |
| osNpcSetProfileImage | High | Stub | |
| osGetNPCList | None | Stub | returns empty list |

## Terrain Functions (4)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osSetTerrainHeight | Low | Stub | |
| osSetRegionWaterHeight | Low | Stub | |
| osSetTerrainTexture | Low | Stub | |
| osSetTerrainTextureHeight | Low | Stub | |

## Drawing / Dynamic Texture Functions (17)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osDrawText | None | Compute | builds "DrawText" command string |
| osMovePen | None | Compute | builds "MoveTo x,y" command string |
| osDrawLine | None | Compute | builds "LineTo" command string |
| osDrawRectangle | None | Compute | builds "Rectangle" command string |
| osDrawFilledRectangle | None | Compute | builds "FillRectangle" command string |
| osDrawEllipse | None | Compute | builds "Ellipse" command string |
| osDrawFilledEllipse | None | Compute | builds "FillEllipse" command string |
| osSetFontName | None | Compute | builds "FontName" command string |
| osSetFontSize | None | Compute | builds "FontSize" command string |
| osSetPenSize | None | Compute | builds "PenSize" command string |
| osSetPenColor | None | Compute | builds "PenColor" command string |
| osSetPenCap | None | Compute | builds "PenCap" command string |
| osSetDynamicTextureData | None | Stub | returns random UUID |
| osSetDynamicTextureDataFace | None | Stub | registered in threat map only |
| osSetDynamicTextureURL | None | Stub | registered in threat map only |
| osSetDynamicTextureURLBlend | None | Stub | registered in threat map only |
| osGetDrawStringSize | None | Compute | approx width = len*8, height=16 |
| osSetDynamicTextureDataBlendFace | None | Stub | registered in threat map only |

## Agent Management (4)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osSetSpeed | Moderate | Stub | |
| osGetAgentIP | Severe | Stub | returns "0.0.0.0" |
| osKickAvatar | Severe | Stub | logs call |
| osTeleportAgent | High | Stub | |

## Sun / Environment (3)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osSetSunParam | Low | Stub | |
| osGetSunParam | None | Stub | returns 0.0 |
| osGetCurrentSunHour | None | Compute | returns 12.0 |

## Grid Info (7)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osKey2Name | None | Stub | returns empty string |
| osGetGridName | None | Compute | returns "OpenSim Next" |
| osGetGridLoginURI | None | Compute | returns "http://localhost:9000" |
| osGetGridHomeURI | None | Compute | returns "http://localhost:9000" |
| osGetGridGatekeeperURI | None | Compute | returns "http://localhost:9000" |
| osGetGridCustom | None | Stub | returns empty string |
| osGetAvatarList | None | Stub | returns empty list |

## Map Functions (2)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osGetMapTexture | None | Stub | returns nil UUID |
| osGetRegionMapTexture | None | Stub | returns nil UUID |

## Notecard Functions (4)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osMakeNotecard | Low | Stub | |
| osGetNotecard | Low | Stub | returns empty string |
| osGetNotecardLine | Low | Stub | returns empty string |
| osGetNumberOfNotecardLines | Low | Stub | returns 0 |

## Parcel / Region Admin (5)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osMessageObject | Low | Stub | |
| osSetParcelDetails | High | Stub | |
| osRegionNotice | High | Stub | logs message |
| osRegionRestart | Severe | Stub | logs warning |
| osConsoleCommand | Severe | Stub | registered in threat map only |

## Attachment / Projection (5)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osSetProjectionParams | Moderate | Stub | |
| osGetNumberOfAttachments | None | Stub | returns 0 |
| osForceAttachToAvatar | High | Stub | |
| osForceDetachFromAvatar | High | Stub | |
| osForceDropAttachment | High | Stub | |

## Prim Parameter Functions (2)

| Function | ThreatLevel | Status | Notes |
|----------|-------------|--------|-------|
| osSetPrimitiveParams | High | Stub | registered in threat map only |
| osGetLinkPrimitiveParams | Moderate | Stub | registered in threat map only |

---

## OSSL Summary Statistics

| ThreatLevel | Count |
|-------------|-------|
| None | 35 |
| Low | 10 |
| Moderate | 3 |
| High | 22 |
| Severe | 4 |
| **Total** | **74** (67 with execute handlers + 7 threat-map-only) |

---

## Combined LSL + OSSL Summary

| Category | Count |
|----------|-------|
| LSL Functions | 315 |
| OSSL Functions | 74 |
| **Grand Total** | **389** |

All OSSL functions enforce threat level gating and parse arguments. Drawing functions build command strings (Compute). All others return defaults (Stub). No OSSL function currently triggers server-side effects.
