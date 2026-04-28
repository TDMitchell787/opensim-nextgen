use anyhow::{bail, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use tracing::{info, warn};

use super::encoder::{JointInfluence, MeshSkinInfo, VertexWeights};

#[derive(Debug)]
pub struct ColladaSkinData {
    pub bind_shape_matrix: [f32; 16],
    pub joint_names: Vec<String>,
    pub inverse_bind_matrices: Vec<[f32; 16]>,
    pub weight_values: Vec<f32>,
    pub vertex_joint_pairs: Vec<(usize, usize)>,
    pub vcounts: Vec<usize>,
}

pub fn parse_collada_skin(dae_xml: &str) -> Result<ColladaSkinData> {
    let mut reader = Reader::from_str(dae_xml);
    reader.config_mut().trim_text(true);

    let mut in_skin = false;
    let mut bind_shape_matrix = [0.0f32; 16];
    bind_shape_matrix[0] = 1.0;
    bind_shape_matrix[5] = 1.0;
    bind_shape_matrix[10] = 1.0;
    bind_shape_matrix[15] = 1.0;

    let mut sources: HashMap<String, SourceData> = HashMap::new();
    let mut current_source_id = String::new();
    let mut in_source = false;
    let mut in_name_array = false;
    let mut in_float_array = false;
    let mut in_bind_shape = false;
    let mut in_vcount = false;
    let mut in_v = false;
    let mut current_array_count = 0usize;

    let mut joint_source_ref = String::new();
    let mut inv_bind_source_ref = String::new();
    let mut weight_source_ref = String::new();

    let mut vcounts: Vec<usize> = Vec::new();
    let mut v_data: Vec<usize> = Vec::new();
    let mut joint_input_offset = 0usize;
    let mut weight_input_offset = 1usize;
    let mut input_stride = 2usize;

    let mut in_joints = false;
    let mut in_vertex_weights = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let ln = e.local_name();
                let local_name = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                match local_name {
                    "skin" => {
                        in_skin = true;
                    }
                    "bind_shape_matrix" if in_skin => {
                        in_bind_shape = true;
                    }
                    "source" if in_skin => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_source_id =
                                    format!("#{}", String::from_utf8_lossy(&attr.value));
                                in_source = true;
                            }
                        }
                    }
                    "Name_array" | "IDREF_array" if in_source => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"count" {
                                current_array_count =
                                    String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                            }
                        }
                        in_name_array = true;
                    }
                    "float_array" if in_source => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"count" {
                                current_array_count =
                                    String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                            }
                        }
                        in_float_array = true;
                    }
                    "joints" if in_skin => {
                        in_joints = true;
                    }
                    "vertex_weights" if in_skin => {
                        in_vertex_weights = true;
                    }
                    "input" if in_joints => {
                        let mut semantic = String::new();
                        let mut source = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"semantic" => {
                                    semantic = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"source" => {
                                    source = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                _ => {}
                            }
                        }
                        match semantic.as_str() {
                            "JOINT" => joint_source_ref = source,
                            "INV_BIND_MATRIX" => inv_bind_source_ref = source,
                            _ => {}
                        }
                    }
                    "input" if in_vertex_weights => {
                        let mut semantic = String::new();
                        let mut source = String::new();
                        let mut offset = 0usize;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"semantic" => {
                                    semantic = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"source" => {
                                    source = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"offset" => {
                                    offset =
                                        String::from_utf8_lossy(&attr.value).parse().unwrap_or(0)
                                }
                                _ => {}
                            }
                        }
                        match semantic.as_str() {
                            "JOINT" => {
                                joint_input_offset = offset;
                            }
                            "WEIGHT" => {
                                weight_source_ref = source;
                                weight_input_offset = offset;
                            }
                            _ => {}
                        }
                        input_stride = input_stride.max(offset + 1);
                    }
                    "vcount" if in_vertex_weights => {
                        in_vcount = true;
                    }
                    "v" if in_vertex_weights => {
                        in_v = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let ln = e.local_name();
                let local_name = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                match local_name {
                    "skin" => {
                        in_skin = false;
                    }
                    "bind_shape_matrix" => {
                        in_bind_shape = false;
                    }
                    "source" => {
                        in_source = false;
                        current_source_id.clear();
                    }
                    "Name_array" | "IDREF_array" => {
                        in_name_array = false;
                    }
                    "float_array" => {
                        in_float_array = false;
                    }
                    "joints" => {
                        in_joints = false;
                    }
                    "vertex_weights" => {
                        in_vertex_weights = false;
                    }
                    "vcount" => {
                        in_vcount = false;
                    }
                    "v" => {
                        in_v = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_bind_shape {
                    let vals: Vec<f32> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if vals.len() >= 16 {
                        bind_shape_matrix.copy_from_slice(&vals[..16]);
                    }
                } else if in_name_array && in_source {
                    let names: Vec<String> =
                        text.split_whitespace().map(|s| s.to_string()).collect();
                    sources.insert(current_source_id.clone(), SourceData::Names(names));
                } else if in_float_array && in_source {
                    let vals: Vec<f32> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    sources.insert(current_source_id.clone(), SourceData::Floats(vals));
                } else if in_vcount {
                    vcounts = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                } else if in_v {
                    v_data = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                }
            }
            Err(e) => bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    let joint_names = match sources.get(&joint_source_ref) {
        Some(SourceData::Names(names)) => names.clone(),
        _ => bail!(
            "No joint names found in Collada skin (ref: {})",
            joint_source_ref
        ),
    };

    let inv_bind_floats = match sources.get(&inv_bind_source_ref) {
        Some(SourceData::Floats(f)) => f.clone(),
        _ => bail!(
            "No inverse bind matrices found (ref: {})",
            inv_bind_source_ref
        ),
    };

    let mut inverse_bind_matrices = Vec::new();
    for chunk in inv_bind_floats.chunks_exact(16) {
        let mut mat = [0.0f32; 16];
        mat.copy_from_slice(chunk);
        inverse_bind_matrices.push(mat);
    }

    if inverse_bind_matrices.len() != joint_names.len() {
        bail!(
            "Joint count ({}) != inverse bind matrix count ({})",
            joint_names.len(),
            inverse_bind_matrices.len()
        );
    }

    let weight_values = match sources.get(&weight_source_ref) {
        Some(SourceData::Floats(f)) => f.clone(),
        _ => vec![1.0],
    };

    let mut vertex_joint_pairs = Vec::new();
    let mut idx = 0;
    for &vc in &vcounts {
        for _ in 0..vc {
            if idx + input_stride > v_data.len() {
                break;
            }
            let joint_idx = v_data[idx + joint_input_offset];
            let weight_idx = v_data[idx + weight_input_offset];
            vertex_joint_pairs.push((joint_idx, weight_idx));
            idx += input_stride;
        }
    }

    info!(
        "[COLLADA_SKIN] Parsed: {} joints, {} vertices, {} weight values",
        joint_names.len(),
        vcounts.len(),
        weight_values.len()
    );

    Ok(ColladaSkinData {
        bind_shape_matrix,
        joint_names,
        inverse_bind_matrices,
        weight_values,
        vertex_joint_pairs,
        vcounts,
    })
}

#[derive(Debug)]
enum SourceData {
    Names(Vec<String>),
    Floats(Vec<f32>),
}

pub fn normalize_joint_name(name: &str) -> String {
    if name.starts_with('m')
        && name.len() > 1
        && name.chars().nth(1).map_or(false, |c| c.is_uppercase())
    {
        return name.to_string();
    }

    let upper = name.to_uppercase();
    let upper = upper.replace('.', "_").replace('-', "_");

    static COLLISION_VOL_MAP: &[(&str, &str)] = &[
        ("PELVIS", "mPelvis"),
        ("BUTT", "mPelvis"),
        ("BELLY", "mTorso"),
        ("LEFT_HANDLE", "mTorso"),
        ("RIGHT_HANDLE", "mTorso"),
        ("LOWER_BACK", "mTorso"),
        ("CHEST", "mChest"),
        ("LEFT_PEC", "mChest"),
        ("RIGHT_PEC", "mChest"),
        ("UPPER_BACK", "mChest"),
        ("NECK", "mNeck"),
        ("HEAD", "mHead"),
        ("L_CLAVICLE", "mCollarLeft"),
        ("L_UPPER_ARM", "mShoulderLeft"),
        ("L_LOWER_ARM", "mElbowLeft"),
        ("L_HAND", "mWristLeft"),
        ("R_CLAVICLE", "mCollarRight"),
        ("R_UPPER_ARM", "mShoulderRight"),
        ("R_LOWER_ARM", "mElbowRight"),
        ("R_HAND", "mWristRight"),
        ("R_UPPER_LEG", "mHipRight"),
        ("R_LOWER_LEG", "mKneeRight"),
        ("R_FOOT", "mFootRight"),
        ("L_UPPER_LEG", "mHipLeft"),
        ("L_LOWER_LEG", "mKneeLeft"),
        ("L_FOOT", "mFootLeft"),
    ];

    for &(vol, joint) in COLLISION_VOL_MAP {
        if upper == vol {
            return joint.to_string();
        }
    }

    static AVASTAR_MAP: &[(&str, &str)] = &[
        ("PELVIS", "mPelvis"),
        ("SPINE1", "mSpine1"),
        ("SPINE2", "mSpine2"),
        ("SPINE3", "mSpine3"),
        ("SPINE4", "mSpine4"),
        ("TORSO", "mTorso"),
        ("CHEST", "mChest"),
        ("NECK", "mNeck"),
        ("HEAD", "mHead"),
        ("SKULL", "mSkull"),
        ("EYE_RIGHT", "mEyeRight"),
        ("EYE_LEFT", "mEyeLeft"),
        ("COLLAR_LEFT", "mCollarLeft"),
        ("SHOULDER_LEFT", "mShoulderLeft"),
        ("ELBOW_LEFT", "mElbowLeft"),
        ("WRIST_LEFT", "mWristLeft"),
        ("COLLAR_RIGHT", "mCollarRight"),
        ("SHOULDER_RIGHT", "mShoulderRight"),
        ("ELBOW_RIGHT", "mElbowRight"),
        ("WRIST_RIGHT", "mWristRight"),
        ("HIP_RIGHT", "mHipRight"),
        ("KNEE_RIGHT", "mKneeRight"),
        ("ANKLE_RIGHT", "mAnkleRight"),
        ("FOOT_RIGHT", "mFootRight"),
        ("TOE_RIGHT", "mToeRight"),
        ("HIP_LEFT", "mHipLeft"),
        ("KNEE_LEFT", "mKneeLeft"),
        ("ANKLE_LEFT", "mAnkleLeft"),
        ("FOOT_LEFT", "mFootLeft"),
        ("TOE_LEFT", "mToeLeft"),
        ("GROIN", "mGroin"),
        ("FACE_ROOT", "mFaceRoot"),
        ("FACE_JAW", "mFaceJaw"),
        ("FACE_JAWSHAPER", "mFaceJawShaper"),
        ("FACE_CHIN", "mFaceChin"),
        ("FACE_TEETH_LOWER", "mFaceTeethLower"),
        ("FACE_TEETH_UPPER", "mFaceTeethUpper"),
        ("FACE_TONGUE_BASE", "mFaceTongueBase"),
        ("FACE_TONGUE_TIP", "mFaceTongueTip"),
        ("FACE_NOSE_LEFT", "mFaceNoseLeft"),
        ("FACE_NOSE_CENTER", "mFaceNoseCenter"),
        ("FACE_NOSE_RIGHT", "mFaceNoseRight"),
        ("FACE_NOSE_BASE", "mFaceNoseBase"),
        ("FACE_NOSE_BRIDGE", "mFaceNoseBridge"),
        ("FACE_FOREHEAD_LEFT", "mFaceForeheadLeft"),
        ("FACE_FOREHEAD_RIGHT", "mFaceForeheadRight"),
        ("FACE_FOREHEAD_CENTER", "mFaceForeheadCenter"),
        ("FACE_EYEBROW_OUTER_LEFT", "mFaceEyebrowOuterLeft"),
        ("FACE_EYEBROW_CENTER_LEFT", "mFaceEyebrowCenterLeft"),
        ("FACE_EYEBROW_INNER_LEFT", "mFaceEyebrowInnerLeft"),
        ("FACE_EYEBROW_OUTER_RIGHT", "mFaceEyebrowOuterRight"),
        ("FACE_EYEBROW_CENTER_RIGHT", "mFaceEyebrowCenterRight"),
        ("FACE_EYEBROW_INNER_RIGHT", "mFaceEyebrowInnerRight"),
        ("FACE_EYELID_UPPER_LEFT", "mFaceEyeLidUpperLeft"),
        ("FACE_EYELID_LOWER_LEFT", "mFaceEyeLidLowerLeft"),
        ("FACE_EYELID_UPPER_RIGHT", "mFaceEyeLidUpperRight"),
        ("FACE_EYELID_LOWER_RIGHT", "mFaceEyeLidLowerRight"),
        ("FACE_EAR1_LEFT", "mFaceEar1Left"),
        ("FACE_EAR2_LEFT", "mFaceEar2Left"),
        ("FACE_EAR1_RIGHT", "mFaceEar1Right"),
        ("FACE_EAR2_RIGHT", "mFaceEar2Right"),
        ("FACE_CHEEK_LOWER_LEFT", "mFaceCheekLowerLeft"),
        ("FACE_CHEEK_UPPER_LEFT", "mFaceCheekUpperLeft"),
        ("FACE_CHEEK_LOWER_RIGHT", "mFaceCheekLowerRight"),
        ("FACE_CHEEK_UPPER_RIGHT", "mFaceCheekUpperRight"),
        ("FACE_LIP_LOWER_LEFT", "mFaceLipLowerLeft"),
        ("FACE_LIP_LOWER_RIGHT", "mFaceLipLowerRight"),
        ("FACE_LIP_LOWER_CENTER", "mFaceLipLowerCenter"),
        ("FACE_LIP_UPPER_LEFT", "mFaceLipUpperLeft"),
        ("FACE_LIP_UPPER_RIGHT", "mFaceLipUpperRight"),
        ("FACE_LIP_UPPER_CENTER", "mFaceLipUpperCenter"),
        ("FACE_LIP_CORNER_LEFT", "mFaceLipCornerLeft"),
        ("FACE_LIP_CORNER_RIGHT", "mFaceLipCornerRight"),
        ("FACE_EYECORNER_INNER_LEFT", "mFaceEyecornerInnerLeft"),
        ("FACE_EYECORNER_INNER_RIGHT", "mFaceEyecornerInnerRight"),
        ("FACE_EYE_ALT_RIGHT", "mFaceEyeAltRight"),
        ("FACE_EYE_ALT_LEFT", "mFaceEyeAltLeft"),
        ("WINGS_ROOT", "mWingsRoot"),
        ("WING1_LEFT", "mWing1Left"),
        ("WING2_LEFT", "mWing2Left"),
        ("WING3_LEFT", "mWing3Left"),
        ("WING4_LEFT", "mWing4Left"),
        ("WING4_FAN_LEFT", "mWing4FanLeft"),
        ("WING1_RIGHT", "mWing1Right"),
        ("WING2_RIGHT", "mWing2Right"),
        ("WING3_RIGHT", "mWing3Right"),
        ("WING4_RIGHT", "mWing4Right"),
        ("WING4_FAN_RIGHT", "mWing4FanRight"),
        ("TAIL1", "mTail1"),
        ("TAIL2", "mTail2"),
        ("TAIL3", "mTail3"),
        ("TAIL4", "mTail4"),
        ("TAIL5", "mTail5"),
        ("TAIL6", "mTail6"),
        ("HIND_LIMBS_ROOT", "mHindLimbsRoot"),
        ("HIND_LIMB1_LEFT", "mHindLimb1Left"),
        ("HIND_LIMB2_LEFT", "mHindLimb2Left"),
        ("HIND_LIMB3_LEFT", "mHindLimb3Left"),
        ("HIND_LIMB4_LEFT", "mHindLimb4Left"),
        ("HIND_LIMB1_RIGHT", "mHindLimb1Right"),
        ("HIND_LIMB2_RIGHT", "mHindLimb2Right"),
        ("HIND_LIMB3_RIGHT", "mHindLimb3Right"),
        ("HIND_LIMB4_RIGHT", "mHindLimb4Right"),
        ("HAND_MIDDLE1_LEFT", "mHandMiddle1Left"),
        ("HAND_MIDDLE2_LEFT", "mHandMiddle2Left"),
        ("HAND_MIDDLE3_LEFT", "mHandMiddle3Left"),
        ("HAND_INDEX1_LEFT", "mHandIndex1Left"),
        ("HAND_INDEX2_LEFT", "mHandIndex2Left"),
        ("HAND_INDEX3_LEFT", "mHandIndex3Left"),
        ("HAND_RING1_LEFT", "mHandRing1Left"),
        ("HAND_RING2_LEFT", "mHandRing2Left"),
        ("HAND_RING3_LEFT", "mHandRing3Left"),
        ("HAND_PINKY1_LEFT", "mHandPinky1Left"),
        ("HAND_PINKY2_LEFT", "mHandPinky2Left"),
        ("HAND_PINKY3_LEFT", "mHandPinky3Left"),
        ("HAND_THUMB1_LEFT", "mHandThumb1Left"),
        ("HAND_THUMB2_LEFT", "mHandThumb2Left"),
        ("HAND_THUMB3_LEFT", "mHandThumb3Left"),
        ("HAND_MIDDLE1_RIGHT", "mHandMiddle1Right"),
        ("HAND_MIDDLE2_RIGHT", "mHandMiddle2Right"),
        ("HAND_MIDDLE3_RIGHT", "mHandMiddle3Right"),
        ("HAND_INDEX1_RIGHT", "mHandIndex1Right"),
        ("HAND_INDEX2_RIGHT", "mHandIndex2Right"),
        ("HAND_INDEX3_RIGHT", "mHandIndex3Right"),
        ("HAND_RING1_RIGHT", "mHandRing1Right"),
        ("HAND_RING2_RIGHT", "mHandRing2Right"),
        ("HAND_RING3_RIGHT", "mHandRing3Right"),
        ("HAND_PINKY1_RIGHT", "mHandPinky1Right"),
        ("HAND_PINKY2_RIGHT", "mHandPinky2Right"),
        ("HAND_PINKY3_RIGHT", "mHandPinky3Right"),
        ("HAND_THUMB1_RIGHT", "mHandThumb1Right"),
        ("HAND_THUMB2_RIGHT", "mHandThumb2Right"),
        ("HAND_THUMB3_RIGHT", "mHandThumb3Right"),
    ];

    for &(avastar, sl) in AVASTAR_MAP {
        if upper == avastar {
            return sl.to_string();
        }
    }

    name.to_string()
}

fn sl_skeleton_world_positions() -> HashMap<String, [f32; 3]> {
    // (name, parent, local_x, local_y, local_z) — from Firestorm avatar_skeleton.xml
    // Parent "" means root bone (mPelvis uses absolute world position)
    static SKELETON: &[(&str, &str, f32, f32, f32)] = &[
        ("mPelvis", "", 0.0, 0.0, 1.067),
        ("mSpine1", "mPelvis", 0.0, 0.0, 0.084),
        ("mSpine2", "mSpine1", 0.0, 0.0, -0.084),
        ("mTorso", "mSpine2", 0.0, 0.0, 0.084),
        ("mSpine3", "mTorso", -0.015, 0.0, 0.205),
        ("mSpine4", "mSpine3", 0.015, 0.0, -0.205),
        ("mChest", "mSpine4", -0.015, 0.0, 0.205),
        ("mNeck", "mChest", -0.010, 0.0, 0.251),
        ("mHead", "mNeck", 0.0, 0.0, 0.076),
        ("mSkull", "mHead", 0.0, 0.0, 0.079),
        ("mEyeRight", "mHead", 0.098, -0.036, 0.079),
        ("mEyeLeft", "mHead", 0.098, 0.036, 0.079),
        // Left arm
        ("mCollarLeft", "mChest", -0.021, 0.085, 0.165),
        ("mShoulderLeft", "mCollarLeft", 0.0, 0.079, 0.0),
        ("mElbowLeft", "mShoulderLeft", 0.0, 0.248, 0.0),
        ("mWristLeft", "mElbowLeft", 0.0, 0.205, 0.0),
        // Right arm
        ("mCollarRight", "mChest", -0.021, -0.085, 0.165),
        ("mShoulderRight", "mCollarRight", 0.0, -0.079, 0.0),
        ("mElbowRight", "mShoulderRight", 0.0, -0.248, 0.0),
        ("mWristRight", "mElbowRight", 0.0, -0.205, 0.0),
        // Left leg
        ("mHipLeft", "mPelvis", 0.034, 0.127, -0.041),
        ("mKneeLeft", "mHipLeft", -0.001, -0.046, -0.491),
        ("mAnkleLeft", "mKneeLeft", -0.029, 0.001, -0.468),
        ("mFootLeft", "mAnkleLeft", 0.112, 0.0, -0.061),
        ("mToeLeft", "mFootLeft", 0.109, 0.0, 0.0),
        // Right leg
        ("mHipRight", "mPelvis", 0.034, -0.129, -0.041),
        ("mKneeRight", "mHipRight", -0.001, 0.049, -0.491),
        ("mAnkleRight", "mKneeRight", -0.029, 0.0, -0.468),
        ("mFootRight", "mAnkleRight", 0.112, 0.0, -0.061),
        ("mToeRight", "mFootRight", 0.109, 0.0, 0.0),
        ("mGroin", "mPelvis", 0.064, 0.0, -0.097),
        // Face
        ("mFaceRoot", "mHead", 0.025, 0.0, 0.045),
        ("mFaceEyeAltRight", "mFaceRoot", 0.073, -0.036, 0.034),
        ("mFaceEyeAltLeft", "mFaceRoot", 0.073, 0.036, 0.034),
        ("mFaceForeheadLeft", "mFaceRoot", 0.061, 0.035, 0.083),
        ("mFaceForeheadRight", "mFaceRoot", 0.061, -0.035, 0.083),
        ("mFaceForeheadCenter", "mFaceRoot", 0.069, 0.0, 0.065),
        ("mFaceEyebrowOuterLeft", "mFaceRoot", 0.064, 0.051, 0.048),
        ("mFaceEyebrowCenterLeft", "mFaceRoot", 0.070, 0.043, 0.056),
        ("mFaceEyebrowInnerLeft", "mFaceRoot", 0.075, 0.022, 0.051),
        ("mFaceEyebrowOuterRight", "mFaceRoot", 0.064, -0.051, 0.048),
        ("mFaceEyebrowCenterRight", "mFaceRoot", 0.070, -0.043, 0.056),
        ("mFaceEyebrowInnerRight", "mFaceRoot", 0.075, -0.022, 0.051),
        ("mFaceEyeLidUpperLeft", "mFaceRoot", 0.073, 0.036, 0.034),
        ("mFaceEyeLidLowerLeft", "mFaceRoot", 0.073, 0.036, 0.034),
        ("mFaceEyeLidUpperRight", "mFaceRoot", 0.073, -0.036, 0.034),
        ("mFaceEyeLidLowerRight", "mFaceRoot", 0.073, -0.036, 0.034),
        ("mFaceEar1Left", "mFaceRoot", 0.0, 0.080, 0.002),
        ("mFaceEar2Left", "mFaceEar1Left", -0.019, 0.018, 0.025),
        ("mFaceEar1Right", "mFaceRoot", 0.0, -0.080, 0.002),
        ("mFaceEar2Right", "mFaceEar1Right", -0.019, -0.018, 0.025),
        ("mFaceNoseLeft", "mFaceRoot", 0.086, 0.015, -0.004),
        ("mFaceNoseCenter", "mFaceRoot", 0.102, 0.0, 0.0),
        ("mFaceNoseRight", "mFaceRoot", 0.086, -0.015, -0.004),
        ("mFaceNoseBase", "mFaceRoot", 0.094, 0.0, -0.016),
        ("mFaceNoseBridge", "mFaceRoot", 0.091, 0.0, 0.020),
        ("mFaceCheekLowerLeft", "mFaceRoot", 0.050, 0.034, -0.031),
        ("mFaceCheekUpperLeft", "mFaceRoot", 0.070, 0.034, -0.005),
        ("mFaceCheekLowerRight", "mFaceRoot", 0.050, -0.034, -0.031),
        ("mFaceCheekUpperRight", "mFaceRoot", 0.070, -0.034, -0.005),
        ("mFaceJaw", "mFaceRoot", -0.001, 0.0, -0.015),
        ("mFaceChin", "mFaceJaw", 0.074, 0.0, -0.054),
        ("mFaceTeethLower", "mFaceJaw", 0.021, 0.0, -0.039),
        ("mFaceLipLowerLeft", "mFaceTeethLower", 0.045, 0.0, 0.0),
        ("mFaceLipLowerRight", "mFaceTeethLower", 0.045, 0.0, 0.0),
        ("mFaceLipLowerCenter", "mFaceTeethLower", 0.045, 0.0, 0.0),
        ("mFaceTongueBase", "mFaceTeethLower", 0.039, 0.0, 0.005),
        ("mFaceTongueTip", "mFaceTongueBase", 0.022, 0.0, 0.007),
        ("mFaceJawShaper", "mFaceRoot", 0.0, 0.0, 0.0),
        ("mFaceTeethUpper", "mFaceRoot", 0.020, 0.0, -0.030),
        ("mFaceLipUpperLeft", "mFaceTeethUpper", 0.045, 0.0, -0.003),
        ("mFaceLipUpperRight", "mFaceTeethUpper", 0.045, 0.0, -0.003),
        (
            "mFaceLipCornerLeft",
            "mFaceTeethUpper",
            0.028,
            -0.019,
            -0.010,
        ),
        (
            "mFaceLipCornerRight",
            "mFaceTeethUpper",
            0.028,
            0.019,
            -0.010,
        ),
        ("mFaceLipUpperCenter", "mFaceTeethUpper", 0.045, 0.0, -0.003),
        ("mFaceEyecornerInnerLeft", "mFaceRoot", 0.075, 0.017, 0.032),
        (
            "mFaceEyecornerInnerRight",
            "mFaceRoot",
            0.075,
            -0.017,
            0.032,
        ),
        // Left hand fingers
        ("mHandMiddle1Left", "mWristLeft", 0.013, 0.101, 0.015),
        (
            "mHandMiddle2Left",
            "mHandMiddle1Left",
            -0.001,
            0.040,
            -0.006,
        ),
        (
            "mHandMiddle3Left",
            "mHandMiddle2Left",
            -0.001,
            0.049,
            -0.008,
        ),
        ("mHandIndex1Left", "mWristLeft", 0.038, 0.097, 0.015),
        ("mHandIndex2Left", "mHandIndex1Left", 0.017, 0.036, -0.006),
        ("mHandIndex3Left", "mHandIndex2Left", 0.014, 0.032, -0.006),
        ("mHandRing1Left", "mWristLeft", -0.010, 0.099, 0.009),
        ("mHandRing2Left", "mHandRing1Left", -0.013, 0.038, -0.008),
        ("mHandRing3Left", "mHandRing2Left", -0.013, 0.040, -0.009),
        ("mHandPinky1Left", "mWristLeft", -0.031, 0.095, 0.003),
        ("mHandPinky2Left", "mHandPinky1Left", -0.024, 0.025, -0.006),
        ("mHandPinky3Left", "mHandPinky2Left", -0.015, 0.018, -0.004),
        ("mHandThumb1Left", "mWristLeft", 0.031, 0.026, 0.004),
        ("mHandThumb2Left", "mHandThumb1Left", 0.028, 0.032, -0.001),
        ("mHandThumb3Left", "mHandThumb2Left", 0.023, 0.031, -0.001),
        // Right hand fingers
        ("mHandMiddle1Right", "mWristRight", 0.013, -0.101, 0.015),
        (
            "mHandMiddle2Right",
            "mHandMiddle1Right",
            -0.001,
            -0.040,
            -0.006,
        ),
        (
            "mHandMiddle3Right",
            "mHandMiddle2Right",
            -0.001,
            -0.049,
            -0.008,
        ),
        ("mHandIndex1Right", "mWristRight", 0.038, -0.097, 0.015),
        (
            "mHandIndex2Right",
            "mHandIndex1Right",
            0.017,
            -0.036,
            -0.006,
        ),
        (
            "mHandIndex3Right",
            "mHandIndex2Right",
            0.014,
            -0.032,
            -0.006,
        ),
        ("mHandRing1Right", "mWristRight", -0.010, -0.099, 0.009),
        ("mHandRing2Right", "mHandRing1Right", -0.013, -0.038, -0.008),
        ("mHandRing3Right", "mHandRing2Right", -0.013, -0.040, -0.009),
        ("mHandPinky1Right", "mWristRight", -0.031, -0.095, 0.003),
        (
            "mHandPinky2Right",
            "mHandPinky1Right",
            -0.024,
            -0.025,
            -0.006,
        ),
        (
            "mHandPinky3Right",
            "mHandPinky2Right",
            -0.015,
            -0.018,
            -0.004,
        ),
        ("mHandThumb1Right", "mWristRight", 0.031, -0.026, 0.004),
        (
            "mHandThumb2Right",
            "mHandThumb1Right",
            0.028,
            -0.032,
            -0.001,
        ),
        (
            "mHandThumb3Right",
            "mHandThumb2Right",
            0.023,
            -0.031,
            -0.001,
        ),
        // Wings
        ("mWingsRoot", "mChest", -0.014, 0.0, 0.0),
        ("mWing1Left", "mWingsRoot", -0.099, 0.105, 0.181),
        ("mWing2Left", "mWing1Left", -0.168, 0.169, 0.067),
        ("mWing3Left", "mWing2Left", -0.181, 0.183, 0.0),
        ("mWing4Left", "mWing3Left", -0.171, 0.173, 0.0),
        ("mWing4FanLeft", "mWing3Left", -0.171, 0.173, 0.0),
        ("mWing1Right", "mWingsRoot", -0.099, -0.105, 0.181),
        ("mWing2Right", "mWing1Right", -0.168, -0.169, 0.067),
        ("mWing3Right", "mWing2Right", -0.181, -0.183, 0.0),
        ("mWing4Right", "mWing3Right", -0.171, -0.173, 0.0),
        ("mWing4FanRight", "mWing3Right", -0.171, -0.173, 0.0),
        // Tail
        ("mTail1", "mPelvis", -0.116, 0.0, 0.047),
        ("mTail2", "mTail1", -0.197, 0.0, 0.0),
        ("mTail3", "mTail2", -0.168, 0.0, 0.0),
        ("mTail4", "mTail3", -0.142, 0.0, 0.0),
        ("mTail5", "mTail4", -0.112, 0.0, 0.0),
        ("mTail6", "mTail5", -0.094, 0.0, 0.0),
        // Hind limbs
        ("mHindLimbsRoot", "mPelvis", -0.200, 0.0, 0.084),
        ("mHindLimb1Left", "mHindLimbsRoot", -0.204, 0.129, -0.125),
        ("mHindLimb2Left", "mHindLimb1Left", 0.002, -0.046, -0.491),
        ("mHindLimb3Left", "mHindLimb2Left", -0.030, -0.003, -0.468),
        ("mHindLimb4Left", "mHindLimb3Left", 0.112, 0.0, -0.061),
        ("mHindLimb1Right", "mHindLimbsRoot", -0.204, -0.129, -0.125),
        ("mHindLimb2Right", "mHindLimb1Right", 0.002, 0.046, -0.491),
        ("mHindLimb3Right", "mHindLimb2Right", -0.030, 0.003, -0.468),
        ("mHindLimb4Right", "mHindLimb3Right", 0.112, 0.0, -0.061),
    ];

    let mut world_pos: HashMap<String, [f32; 3]> = HashMap::with_capacity(SKELETON.len());
    for &(name, parent, lx, ly, lz) in SKELETON {
        let wp = if parent.is_empty() {
            [lx, ly, lz]
        } else if let Some(pp) = world_pos.get(parent) {
            [pp[0] + lx, pp[1] + ly, pp[2] + lz]
        } else {
            [lx, ly, lz]
        };
        world_pos.insert(name.to_string(), wp);
    }
    world_pos
}

fn are_all_inverse_bind_matrices_identity(matrices: &[[f32; 16]]) -> bool {
    if matrices.is_empty() {
        return false;
    }
    let identity = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let mut all_identity = true;
    for mat in matrices {
        for i in 0..16 {
            if (mat[i] - identity[i]).abs() > 0.001 {
                all_identity = false;
                break;
            }
        }
        if !all_identity {
            break;
        }
    }
    all_identity
}

pub fn to_mesh_skin_info(data: &ColladaSkinData) -> (MeshSkinInfo, Vec<VertexWeights>) {
    to_mesh_skin_info_with_axis(data, true)
}

pub fn to_mesh_skin_info_with_axis(
    data: &ColladaSkinData,
    y_up: bool,
) -> (MeshSkinInfo, Vec<VertexWeights>) {
    let normalized_names: Vec<String> = data
        .joint_names
        .iter()
        .map(|n| normalize_joint_name(n))
        .collect();

    let mut per_vertex_weights = Vec::with_capacity(data.vcounts.len());
    let mut pair_idx = 0usize;

    for &vc in &data.vcounts {
        let mut influences = Vec::new();
        for _ in 0..vc {
            if pair_idx >= data.vertex_joint_pairs.len() {
                break;
            }
            let (joint_idx, weight_idx) = data.vertex_joint_pairs[pair_idx];
            pair_idx += 1;

            let w = if weight_idx < data.weight_values.len() {
                data.weight_values[weight_idx]
            } else {
                0.0
            };

            if w > 0.0001 && joint_idx < 255 {
                influences.push(JointInfluence {
                    joint_index: joint_idx as u8,
                    weight: w,
                });
            }
        }

        influences.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        influences.truncate(4);

        if influences.is_empty() {
            influences.push(JointInfluence {
                joint_index: 0,
                weight: 1.0,
            });
        }

        let wsum: f32 = influences.iter().map(|i| i.weight).sum();
        if wsum > 0.0 {
            for inf in &mut influences {
                inf.weight /= wsum;
            }
        }

        per_vertex_weights.push(VertexWeights { influences });
    }

    let inverse_bind_matrices = {
        info!(
            "[COLLADA_SKIN] Computing SL skeleton inverse bind matrices for {} joints",
            normalized_names.len()
        );
        let skel = sl_skeleton_world_positions();
        let mut fixed: Vec<[f32; 16]> = Vec::with_capacity(normalized_names.len());
        let mut matched = 0usize;
        for name in &normalized_names {
            if let Some(wp) = skel.get(name.as_str()) {
                fixed.push([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -wp[0], -wp[1],
                    -wp[2], 1.0,
                ]);
                matched += 1;
            } else {
                warn!(
                    "[COLLADA_SKIN] Joint '{}' not in SL skeleton — using identity",
                    name
                );
                fixed.push([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ]);
            }
        }
        info!(
            "[COLLADA_SKIN] Matched {}/{} joints to SL skeleton",
            matched,
            normalized_names.len()
        );
        if !fixed.is_empty() {
            let m = &fixed[0];
            info!(
                "[COLLADA_SKIN] IBM[0] ({}): trans=({:.3},{:.3},{:.3})",
                normalized_names[0], m[12], m[13], m[14]
            );
        }
        if fixed.len() > 3 {
            let m = &fixed[3];
            info!(
                "[COLLADA_SKIN] IBM[3] ({}): trans=({:.3},{:.3},{:.3})",
                normalized_names[3], m[12], m[13], m[14]
            );
        }
        fixed
    };

    let bsm = &data.bind_shape_matrix;
    let bind_shape_transposed = if y_up {
        let converted = [
            bsm[0], -bsm[2], bsm[1], bsm[3], -bsm[8], bsm[10], -bsm[9], -bsm[11], bsm[4], -bsm[6],
            bsm[5], bsm[7], bsm[12], -bsm[14], bsm[13], bsm[15],
        ];
        [
            converted[0],
            converted[4],
            converted[8],
            converted[12],
            converted[1],
            converted[5],
            converted[9],
            converted[13],
            converted[2],
            converted[6],
            converted[10],
            converted[14],
            converted[3],
            converted[7],
            converted[11],
            converted[15],
        ]
    } else {
        [
            bsm[0], bsm[4], bsm[8], bsm[12], bsm[1], bsm[5], bsm[9], bsm[13], bsm[2], bsm[6],
            bsm[10], bsm[14], bsm[3], bsm[7], bsm[11], bsm[15],
        ]
    };

    info!("[COLLADA_SKIN] BSM raw: [{:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}]",
        bsm[0],bsm[1],bsm[2],bsm[3], bsm[4],bsm[5],bsm[6],bsm[7],
        bsm[8],bsm[9],bsm[10],bsm[11], bsm[12],bsm[13],bsm[14],bsm[15]);
    info!("[COLLADA_SKIN] BSM out: [{:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}, {:.3},{:.3},{:.3},{:.3}]",
        bind_shape_transposed[0],bind_shape_transposed[1],bind_shape_transposed[2],bind_shape_transposed[3],
        bind_shape_transposed[4],bind_shape_transposed[5],bind_shape_transposed[6],bind_shape_transposed[7],
        bind_shape_transposed[8],bind_shape_transposed[9],bind_shape_transposed[10],bind_shape_transposed[11],
        bind_shape_transposed[12],bind_shape_transposed[13],bind_shape_transposed[14],bind_shape_transposed[15]);

    let skin_info = MeshSkinInfo {
        joint_names: normalized_names,
        inverse_bind_matrices,
        bind_shape_matrix: bind_shape_transposed,
        pelvis_offset: 0.0,
    };

    (skin_info, per_vertex_weights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_known_sl_names() {
        assert_eq!(normalize_joint_name("mPelvis"), "mPelvis");
        assert_eq!(normalize_joint_name("mChest"), "mChest");
        assert_eq!(normalize_joint_name("mWristLeft"), "mWristLeft");
    }

    #[test]
    fn test_normalize_collision_volumes() {
        assert_eq!(normalize_joint_name("PELVIS"), "mPelvis");
        assert_eq!(normalize_joint_name("L_UPPER_ARM"), "mShoulderLeft");
        assert_eq!(normalize_joint_name("R_FOOT"), "mFootRight");
    }

    #[test]
    fn test_normalize_avastar_names() {
        assert_eq!(normalize_joint_name("SHOULDER_LEFT"), "mShoulderLeft");
        assert_eq!(normalize_joint_name("ELBOW_RIGHT"), "mElbowRight");
        assert_eq!(normalize_joint_name("TAIL1"), "mTail1");
    }

    #[test]
    fn test_parse_minimal_skin() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_controllers>
    <controller id="Armature_Cube-skin" name="Armature">
      <skin source="#Cube-mesh">
        <bind_shape_matrix>1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</bind_shape_matrix>
        <source id="joints">
          <Name_array id="joints-array" count="2">mPelvis mTorso</Name_array>
        </source>
        <source id="matrices">
          <float_array id="matrices-array" count="32">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</float_array>
        </source>
        <source id="weights">
          <float_array id="weights-array" count="2">0.7 0.3</float_array>
        </source>
        <joints>
          <input semantic="JOINT" source="#joints"/>
          <input semantic="INV_BIND_MATRIX" source="#matrices"/>
        </joints>
        <vertex_weights count="2">
          <input semantic="JOINT" offset="0" source="#joints"/>
          <input semantic="WEIGHT" offset="1" source="#weights"/>
          <vcount>2 1</vcount>
          <v>0 0 1 1 0 0</v>
        </vertex_weights>
      </skin>
    </controller>
  </library_controllers>
</COLLADA>"##;

        let skin = parse_collada_skin(dae).expect("parse failed");
        assert_eq!(skin.joint_names.len(), 2);
        assert_eq!(skin.joint_names[0], "mPelvis");
        assert_eq!(skin.joint_names[1], "mTorso");
        assert_eq!(skin.inverse_bind_matrices.len(), 2);
        assert_eq!(skin.vcounts.len(), 2);
        assert_eq!(skin.vcounts[0], 2);
        assert_eq!(skin.vcounts[1], 1);

        let (info, weights) = to_mesh_skin_info(&skin);
        assert_eq!(info.joint_names.len(), 2);
        assert_eq!(weights.len(), 2);
        assert_eq!(weights[0].influences.len(), 2);
        assert_eq!(weights[1].influences.len(), 1);
        let w0_sum: f32 = weights[0].influences.iter().map(|i| i.weight).sum();
        assert!((w0_sum - 1.0).abs() < 0.01);

        // SL viewer reads translation from mMatrix[3][0..2] = flat[12..14]
        // mPelvis world pos = (0, 0, 1.067) → flat[12]=0.0, flat[13]=0.0, flat[14]=-1.067
        let pelvis_mat = &info.inverse_bind_matrices[0];
        assert!(
            (pelvis_mat[12] - 0.0).abs() < 0.01,
            "pelvis inv_bind X: {}",
            pelvis_mat[12]
        );
        assert!(
            (pelvis_mat[13] - 0.0).abs() < 0.01,
            "pelvis inv_bind Y: {}",
            pelvis_mat[13]
        );
        assert!(
            (pelvis_mat[14] - (-1.067)).abs() < 0.01,
            "pelvis inv_bind Z: {} expected -1.067",
            pelvis_mat[14]
        );

        // mTorso world pos: mPelvis(0,0,1.067) + mSpine1(0,0,0.084) + mSpine2(0,0,-0.084) + mTorso(0,0,0.084) = (0,0,1.151)
        let torso_mat = &info.inverse_bind_matrices[1];
        assert!(
            (torso_mat[14] - (-1.151)).abs() < 0.01,
            "torso inv_bind Z: {} expected -1.151",
            torso_mat[14]
        );
    }

    #[test]
    fn test_blender_matrices_replaced_with_sl_skeleton() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_controllers>
    <controller id="skin" name="Armature">
      <skin source="#mesh">
        <bind_shape_matrix>1 0 0 0 0 1 0 0 0 0 1 0.85 0 0 0 1</bind_shape_matrix>
        <source id="joints">
          <Name_array id="j" count="2">mPelvis mChest</Name_array>
        </source>
        <source id="matrices">
          <float_array id="m" count="32">1 0 0 0 0 0 1 -0.976 0 -1 0 0 0 0 0 1 1 0 0 0 0 0.04 1 -1.41 0 -1 0.04 0 0 0 0 1</float_array>
        </source>
        <source id="weights">
          <float_array id="w" count="1">1.0</float_array>
        </source>
        <joints>
          <input semantic="JOINT" source="#joints"/>
          <input semantic="INV_BIND_MATRIX" source="#matrices"/>
        </joints>
        <vertex_weights count="1">
          <input semantic="JOINT" offset="0" source="#joints"/>
          <input semantic="WEIGHT" offset="1" source="#weights"/>
          <vcount>1</vcount>
          <v>0 0</v>
        </vertex_weights>
      </skin>
    </controller>
  </library_controllers>
</COLLADA>"##;

        let skin = parse_collada_skin(dae).expect("parse failed");
        let (info, _) = to_mesh_skin_info(&skin);
        assert!(
            (info.inverse_bind_matrices[0][14] - (-1.067)).abs() < 0.01,
            "pelvis Z: {}",
            info.inverse_bind_matrices[0][14]
        );
        assert!(
            (info.inverse_bind_matrices[1][14] - (-1.356)).abs() < 0.01,
            "chest Z: {}",
            info.inverse_bind_matrices[1][14]
        );
        assert!(
            (info.inverse_bind_matrices[0][12] - 0.0).abs() < 0.01,
            "pelvis X should be 0"
        );
        assert!(
            (info.inverse_bind_matrices[0][13] - 0.0).abs() < 0.01,
            "pelvis Y should be 0"
        );
    }

    #[test]
    fn test_skeleton_world_positions() {
        let skel = sl_skeleton_world_positions();
        let pelvis = skel.get("mPelvis").unwrap();
        assert!(
            (pelvis[2] - 1.067).abs() < 0.001,
            "mPelvis Z: {}",
            pelvis[2]
        );

        let chest = skel.get("mChest").unwrap();
        assert!((chest[2] - 1.356).abs() < 0.001, "mChest Z: {}", chest[2]);

        let head = skel.get("mHead").unwrap();
        assert!(head[2] > 1.6, "mHead Z should be >1.6, got {}", head[2]);

        let ankle_left = skel.get("mAnkleLeft").unwrap();
        assert!(
            ankle_left[2] < 0.1,
            "mAnkleLeft Z should be near ground, got {}",
            ankle_left[2]
        );
    }
}
