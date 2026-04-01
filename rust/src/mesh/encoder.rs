use anyhow::{bail, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JointInfluence {
    pub joint_index: u8,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct VertexWeights {
    pub influences: Vec<JointInfluence>,
}

#[derive(Debug, Clone)]
pub struct MeshSkinInfo {
    pub joint_names: Vec<String>,
    pub inverse_bind_matrices: Vec<[f32; 16]>,
    pub bind_shape_matrix: [f32; 16],
    pub pelvis_offset: f32,
}

#[derive(Debug, Clone)]
pub struct MeshFace {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub joint_weights: Option<Vec<VertexWeights>>,
    pub original_position_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct MeshGeometry {
    pub faces: Vec<MeshFace>,
    pub skin_info: Option<MeshSkinInfo>,
}

enum LlsdBinValue {
    Integer(i32),
    Real(f64),
    Binary(Vec<u8>),
    LlsdString(String),
    Bool(bool),
    Map(Vec<(String, LlsdBinValue)>),
    Array(Vec<LlsdBinValue>),
}

fn write_llsd_value(buf: &mut Vec<u8>, val: &LlsdBinValue) {
    match val {
        LlsdBinValue::Integer(v) => {
            buf.push(b'i');
            buf.extend_from_slice(&v.to_be_bytes());
        }
        LlsdBinValue::Real(v) => {
            buf.push(b'r');
            buf.extend_from_slice(&v.to_be_bytes());
        }
        LlsdBinValue::Binary(data) => {
            buf.push(b'b');
            buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
            buf.extend_from_slice(data);
        }
        LlsdBinValue::LlsdString(s) => {
            buf.push(b's');
            buf.extend_from_slice(&(s.len() as u32).to_be_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        LlsdBinValue::Bool(v) => {
            buf.push(if *v { b'1' } else { b'0' });
        }
        LlsdBinValue::Map(entries) => {
            buf.push(b'{');
            buf.extend_from_slice(&(entries.len() as u32).to_be_bytes());
            for (key, value) in entries {
                buf.push(b'k');
                buf.extend_from_slice(&(key.len() as u32).to_be_bytes());
                buf.extend_from_slice(key.as_bytes());
                write_llsd_value(buf, value);
            }
            buf.push(b'}');
        }
        LlsdBinValue::Array(items) => {
            buf.push(b'[');
            buf.extend_from_slice(&(items.len() as u32).to_be_bytes());
            for item in items {
                write_llsd_value(buf, item);
            }
            buf.push(b']');
        }
    }
}

fn serialize_llsd(val: &LlsdBinValue) -> Vec<u8> {
    let mut buf = Vec::new();
    write_llsd_value(&mut buf, val);
    buf
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn compute_domain_3d(verts: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for v in verts {
        for i in 0..3 {
            if v[i] < min[i] { min[i] = v[i]; }
            if v[i] > max[i] { max[i] = v[i]; }
        }
    }
    for i in 0..3 {
        if (max[i] - min[i]).abs() < 1e-6 {
            min[i] -= 0.5;
            max[i] += 0.5;
        }
    }
    (min, max)
}

fn compute_domain_2d(coords: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut min = [f32::MAX; 2];
    let mut max = [f32::MIN; 2];
    for c in coords {
        for i in 0..2 {
            if c[i] < min[i] { min[i] = c[i]; }
            if c[i] > max[i] { max[i] = c[i]; }
        }
    }
    for i in 0..2 {
        if (max[i] - min[i]).abs() < 1e-6 {
            min[i] -= 0.5;
            max[i] += 0.5;
        }
    }
    (min, max)
}

fn quantize_f32(value: f32, domain_min: f32, domain_max: f32) -> u16 {
    let range = domain_max - domain_min;
    if range.abs() < 1e-12 {
        return 0;
    }
    let norm = ((value - domain_min) / range).clamp(0.0, 1.0);
    (norm * 65535.0) as u16
}

fn quantize_vec3_to_bytes(verts: &[[f32; 3]], min: &[f32; 3], max: &[f32; 3]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(verts.len() * 6);
    for v in verts {
        for i in 0..3 {
            buf.extend_from_slice(&quantize_f32(v[i], min[i], max[i]).to_le_bytes());
        }
    }
    buf
}

fn quantize_vec2_to_bytes(coords: &[[f32; 2]], min: &[f32; 2], max: &[f32; 2]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(coords.len() * 4);
    for c in coords {
        for i in 0..2 {
            buf.extend_from_slice(&quantize_f32(c[i], min[i], max[i]).to_le_bytes());
        }
    }
    buf
}

fn indices_to_u16_bytes(indices: &[u32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(indices.len() * 2);
    for &idx in indices {
        buf.extend_from_slice(&(idx as u16).to_le_bytes());
    }
    buf
}

fn make_domain_3d_llsd(min: &[f32; 3], max: &[f32; 3]) -> LlsdBinValue {
    LlsdBinValue::Map(vec![
        ("Min".into(), LlsdBinValue::Array(vec![
            LlsdBinValue::Real(min[0] as f64),
            LlsdBinValue::Real(min[1] as f64),
            LlsdBinValue::Real(min[2] as f64),
        ])),
        ("Max".into(), LlsdBinValue::Array(vec![
            LlsdBinValue::Real(max[0] as f64),
            LlsdBinValue::Real(max[1] as f64),
            LlsdBinValue::Real(max[2] as f64),
        ])),
    ])
}

fn make_domain_2d_llsd(min: &[f32; 2], max: &[f32; 2]) -> LlsdBinValue {
    LlsdBinValue::Map(vec![
        ("Min".into(), LlsdBinValue::Array(vec![
            LlsdBinValue::Real(min[0] as f64),
            LlsdBinValue::Real(min[1] as f64),
        ])),
        ("Max".into(), LlsdBinValue::Array(vec![
            LlsdBinValue::Real(max[0] as f64),
            LlsdBinValue::Real(max[1] as f64),
        ])),
    ])
}

fn encode_face_llsd(face: &MeshFace) -> Result<LlsdBinValue> {
    if face.positions.is_empty() {
        bail!("Face has no positions");
    }
    if face.positions.len() > 65535 {
        bail!("Face has {} vertices, max 65535", face.positions.len());
    }
    for &idx in &face.indices {
        if idx as usize >= face.positions.len() {
            bail!("Index {} out of range (vertex count: {})", idx, face.positions.len());
        }
    }

    let (pos_min, pos_max) = compute_domain_3d(&face.positions);
    let norm_min = [-1.0f32, -1.0, -1.0];
    let norm_max = [1.0f32, 1.0, 1.0];
    let (tc_min, tc_max) = if face.tex_coords.is_empty() {
        ([0.0f32, 0.0], [1.0f32, 1.0])
    } else {
        compute_domain_2d(&face.tex_coords)
    };

    let pos_bytes = quantize_vec3_to_bytes(&face.positions, &pos_min, &pos_max);
    let norm_bytes = if face.normals.len() == face.positions.len() {
        quantize_vec3_to_bytes(&face.normals, &norm_min, &norm_max)
    } else {
        tracing::warn!("[MESH_ENCODE] Normals mismatch: {} normals vs {} positions — using defaults",
            face.normals.len(), face.positions.len());
        let default_normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; face.positions.len()];
        quantize_vec3_to_bytes(&default_normals, &norm_min, &norm_max)
    };
    tracing::info!("[MESH_ENCODE] face: {} verts, {} norms, {} tcs, {} indices, pos_domain=[{:.3},{:.3},{:.3}]-[{:.3},{:.3},{:.3}]",
        face.positions.len(), face.normals.len(), face.tex_coords.len(), face.indices.len(),
        pos_min[0], pos_min[1], pos_min[2], pos_max[0], pos_max[1], pos_max[2]);
    let tc_bytes = if face.tex_coords.len() == face.positions.len() {
        quantize_vec2_to_bytes(&face.tex_coords, &tc_min, &tc_max)
    } else {
        tracing::warn!("[MESH_ENCODE] TexCoords mismatch: {} uvs vs {} positions — using defaults",
            face.tex_coords.len(), face.positions.len());
        let default_uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; face.positions.len()];
        quantize_vec2_to_bytes(&default_uvs, &tc_min, &tc_max)
    };
    let idx_bytes = indices_to_u16_bytes(&face.indices);

    let mut entries = Vec::new();
    entries.push(("Position".into(), LlsdBinValue::Binary(pos_bytes)));
    entries.push(("PositionDomain".into(), make_domain_3d_llsd(&pos_min, &pos_max)));
    entries.push(("Normal".into(), LlsdBinValue::Binary(norm_bytes)));
    entries.push(("TexCoord0".into(), LlsdBinValue::Binary(tc_bytes)));
    entries.push(("TexCoord0Domain".into(), make_domain_2d_llsd(&tc_min, &tc_max)));
    entries.push(("TriangleList".into(), LlsdBinValue::Binary(idx_bytes)));

    if let Some(weights_data) = encode_face_weights(face) {
        entries.push(("Weights".into(), LlsdBinValue::Binary(weights_data)));
    }

    Ok(LlsdBinValue::Map(entries))
}

fn encode_lod_section(geometry: &MeshGeometry) -> Result<Vec<u8>> {
    let mut face_values = Vec::new();
    for face in &geometry.faces {
        face_values.push(encode_face_llsd(face)?);
    }
    let array = LlsdBinValue::Array(face_values);
    let serialized = serialize_llsd(&array);
    zlib_compress(&serialized)
}

fn encode_skin_block(skin: &MeshSkinInfo) -> Result<Vec<u8>> {
    let mut joint_names_arr = Vec::new();
    for name in &skin.joint_names {
        joint_names_arr.push(LlsdBinValue::LlsdString(name.clone()));
    }

    let mut inv_bind_arr = Vec::new();
    for mat in &skin.inverse_bind_matrices {
        let vals: Vec<LlsdBinValue> = mat.iter().map(|&v| LlsdBinValue::Real(v as f64)).collect();
        inv_bind_arr.push(LlsdBinValue::Array(vals));
    }

    let bind_shape: Vec<LlsdBinValue> = skin.bind_shape_matrix.iter()
        .map(|&v| LlsdBinValue::Real(v as f64))
        .collect();

    let entries = vec![
        ("joint_names".into(), LlsdBinValue::Array(joint_names_arr)),
        ("inverse_bind_matrix".into(), LlsdBinValue::Array(inv_bind_arr)),
        ("bind_shape_matrix".into(), LlsdBinValue::Array(bind_shape)),
        ("pelvis_offset".into(), LlsdBinValue::Real(skin.pelvis_offset as f64)),
    ];

    let map = LlsdBinValue::Map(entries);
    let serialized = serialize_llsd(&map);
    zlib_compress(&serialized)
}

fn encode_face_weights(face: &MeshFace) -> Option<Vec<u8>> {
    let weights = face.joint_weights.as_ref()?;
    if weights.is_empty() {
        return None;
    }

    let mut buf = Vec::with_capacity(weights.len() * 10);
    for vw in weights {
        let mut influences: Vec<(u8, f32)> = vw.influences.iter()
            .map(|ji| (ji.joint_index, ji.weight))
            .collect();
        influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        influences.truncate(4);

        let wsum: f32 = influences.iter().map(|(_, w)| w).sum();
        let norm = if wsum > 0.0 { 1.0 / wsum } else { 1.0 };

        let count = influences.len();
        for (idx, weight) in &influences {
            buf.push(*idx);
            let w16 = ((*weight * norm) * 65535.0).clamp(0.0, 65535.0) as u16;
            buf.extend_from_slice(&w16.to_le_bytes());
        }
        if count < 4 {
            buf.push(0xFF);
        }
    }

    Some(buf)
}

fn encode_physics_convex(geometry: &MeshGeometry) -> Result<Vec<u8>> {
    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    for face in &geometry.faces {
        all_positions.extend_from_slice(&face.positions);
    }
    if all_positions.is_empty() {
        bail!("No positions for physics convex");
    }

    let (aabb_min, aabb_max) = compute_domain_3d(&all_positions);

    let hull_verts: [[f32; 3]; 8] = [
        [aabb_min[0], aabb_min[1], aabb_min[2]],
        [aabb_max[0], aabb_min[1], aabb_min[2]],
        [aabb_min[0], aabb_max[1], aabb_min[2]],
        [aabb_max[0], aabb_max[1], aabb_min[2]],
        [aabb_min[0], aabb_min[1], aabb_max[2]],
        [aabb_max[0], aabb_min[1], aabb_max[2]],
        [aabb_min[0], aabb_max[1], aabb_max[2]],
        [aabb_max[0], aabb_max[1], aabb_max[2]],
    ];

    let positions_bytes = quantize_vec3_to_bytes(&hull_verts, &aabb_min, &aabb_max);

    let mut min_binary = Vec::with_capacity(12);
    for i in 0..3 {
        min_binary.extend_from_slice(&aabb_min[i].to_le_bytes());
    }
    let mut max_binary = Vec::with_capacity(12);
    for i in 0..3 {
        max_binary.extend_from_slice(&aabb_max[i].to_le_bytes());
    }

    let entries = vec![
        ("Min".into(), LlsdBinValue::Binary(min_binary)),
        ("Max".into(), LlsdBinValue::Binary(max_binary)),
        ("HullList".into(), LlsdBinValue::Binary(vec![8u8])),
        ("Positions".into(), LlsdBinValue::Binary(positions_bytes.clone())),
        ("BoundingVerts".into(), LlsdBinValue::Binary(positions_bytes)),
    ];

    let map = LlsdBinValue::Map(entries);
    let serialized = serialize_llsd(&map);
    zlib_compress(&serialized)
}

pub const MAX_MESH_FACES: usize = 8;

fn merge_faces_to_limit(faces: &[MeshFace], max_faces: usize) -> Vec<MeshFace> {
    if faces.len() <= max_faces {
        return faces.to_vec();
    }

    tracing::info!("[MESH_ENCODE] Merging {} faces down to {} (SL max)", faces.len(), max_faces);

    let mut merged: Vec<MeshFace> = Vec::with_capacity(max_faces);
    for _ in 0..max_faces {
        merged.push(MeshFace {
            positions: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            indices: Vec::new(),
            joint_weights: None,
            original_position_indices: None,
        });
    }

    for (i, face) in faces.iter().enumerate() {
        let target = i % max_faces;
        let base_vert = merged[target].positions.len() as u32;

        merged[target].positions.extend_from_slice(&face.positions);
        merged[target].normals.extend_from_slice(&face.normals);
        merged[target].tex_coords.extend_from_slice(&face.tex_coords);

        for &idx in &face.indices {
            merged[target].indices.push(base_vert + idx);
        }

        if let Some(ref weights) = face.joint_weights {
            let target_weights = merged[target].joint_weights.get_or_insert_with(Vec::new);
            target_weights.extend_from_slice(weights);
        }
    }

    merged.retain(|f| !f.positions.is_empty());

    for face in &merged {
        if face.positions.len() > 65535 {
            tracing::warn!("[MESH_ENCODE] Merged face has {} verts (>65535) — will be split", face.positions.len());
        }
    }

    tracing::info!("[MESH_ENCODE] After merge: {} faces, verts per face: {:?}",
        merged.len(), merged.iter().map(|f| f.positions.len()).collect::<Vec<_>>());

    merged
}

fn split_oversized_face(face: &MeshFace) -> Vec<MeshFace> {
    if face.positions.len() <= 65535 {
        return vec![face.clone()];
    }

    let max_verts = 65000usize;
    let mut result = Vec::new();
    let mut current = MeshFace {
        positions: Vec::new(),
        normals: Vec::new(),
        tex_coords: Vec::new(),
        indices: Vec::new(),
        joint_weights: None,
        original_position_indices: None,
    };

    let tris = face.indices.len() / 3;
    let mut vert_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

    for t in 0..tris {
        let i0 = face.indices[t * 3] as usize;
        let i1 = face.indices[t * 3 + 1] as usize;
        let i2 = face.indices[t * 3 + 2] as usize;

        let needed = [i0, i1, i2].iter()
            .filter(|&&idx| !vert_map.contains_key(&(idx as u32)))
            .count();

        if current.positions.len() + needed > max_verts && !current.positions.is_empty() {
            result.push(current);
            current = MeshFace {
                positions: Vec::new(),
                normals: Vec::new(),
                tex_coords: Vec::new(),
                indices: Vec::new(),
                joint_weights: None,
                original_position_indices: None,
            };
            vert_map.clear();
        }

        let mut map_vert = |idx: usize| -> u32 {
            let key = idx as u32;
            if let Some(&mapped) = vert_map.get(&key) {
                return mapped;
            }
            let new_idx = current.positions.len() as u32;
            current.positions.push(face.positions[idx]);
            if idx < face.normals.len() {
                current.normals.push(face.normals[idx]);
            }
            if idx < face.tex_coords.len() {
                current.tex_coords.push(face.tex_coords[idx]);
            }
            vert_map.insert(key, new_idx);
            new_idx
        };

        let mi0 = map_vert(i0);
        let mi1 = map_vert(i1);
        let mi2 = map_vert(i2);
        current.indices.extend_from_slice(&[mi0, mi1, mi2]);
    }

    if !current.positions.is_empty() {
        result.push(current);
    }

    result
}

pub fn encode_mesh_asset(geometry: &MeshGeometry) -> Result<Vec<u8>> {
    if geometry.faces.is_empty() {
        bail!("MeshGeometry has no faces");
    }
    for (i, face) in geometry.faces.iter().enumerate() {
        if face.positions.is_empty() {
            bail!("Face {} has no positions", i);
        }
        if face.positions.len() > 65535 {
            bail!("Face {} has {} vertices (max 65535)", i, face.positions.len());
        }
    }

    let lod_data = encode_lod_section(geometry)?;
    let physics_data = encode_physics_convex(geometry)?;

    let lod_offset = 0i32;
    let lod_size = lod_data.len() as i32;
    let phys_offset = lod_data.len() as i32;
    let phys_size = physics_data.len() as i32;

    let skin_data = if let Some(skin) = &geometry.skin_info {
        Some(encode_skin_block(skin)?)
    } else {
        None
    };

    let skin_offset = (lod_data.len() + physics_data.len()) as i32;
    let skin_size = skin_data.as_ref().map(|d| d.len() as i32).unwrap_or(0);

    let mut header_entries = vec![
        ("high_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(lod_offset)),
            ("size".into(), LlsdBinValue::Integer(lod_size)),
        ])),
        ("medium_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(lod_offset)),
            ("size".into(), LlsdBinValue::Integer(lod_size)),
        ])),
        ("low_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(lod_offset)),
            ("size".into(), LlsdBinValue::Integer(lod_size)),
        ])),
        ("lowest_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(lod_offset)),
            ("size".into(), LlsdBinValue::Integer(lod_size)),
        ])),
        ("physics_convex".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(phys_offset)),
            ("size".into(), LlsdBinValue::Integer(phys_size)),
        ])),
    ];

    if skin_data.is_some() {
        header_entries.push(("skin".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(skin_offset)),
            ("size".into(), LlsdBinValue::Integer(skin_size)),
        ])));
    }

    let header = LlsdBinValue::Map(header_entries);
    let header_bytes = serialize_llsd(&header);

    let mut result = Vec::with_capacity(
        header_bytes.len() + lod_data.len() + physics_data.len() + skin_size as usize
    );
    result.extend_from_slice(&header_bytes);
    result.extend_from_slice(&lod_data);
    result.extend_from_slice(&physics_data);
    if let Some(sd) = &skin_data {
        result.extend_from_slice(sd);
    }

    Ok(result)
}

#[derive(Debug, Clone)]
pub struct MultiLodGeometry {
    pub high: MeshGeometry,
    pub medium: MeshGeometry,
    pub low: MeshGeometry,
    pub lowest: MeshGeometry,
}

pub fn encode_mesh_asset_multi_lod(lods: &MultiLodGeometry) -> Result<Vec<u8>> {
    for (name, geom) in [("high", &lods.high), ("medium", &lods.medium),
                          ("low", &lods.low), ("lowest", &lods.lowest)] {
        if geom.faces.is_empty() {
            bail!("{} LOD has no faces", name);
        }
        for (i, face) in geom.faces.iter().enumerate() {
            if face.positions.is_empty() {
                bail!("{} LOD face {} has no positions", name, i);
            }
            if face.positions.len() > 65535 {
                bail!("{} LOD face {} has {} vertices (max 65535)", name, i, face.positions.len());
            }
        }
    }

    let high_data = encode_lod_section(&lods.high)?;
    let medium_data = encode_lod_section(&lods.medium)?;
    let low_data = encode_lod_section(&lods.low)?;
    let lowest_data = encode_lod_section(&lods.lowest)?;
    let physics_data = encode_physics_convex(&lods.high)?;

    let skin_data = if let Some(skin) = &lods.high.skin_info {
        Some(encode_skin_block(skin)?)
    } else {
        None
    };

    let mut offset = 0i32;
    let high_offset = offset;
    let high_size = high_data.len() as i32;
    offset += high_size;

    let medium_offset = offset;
    let medium_size = medium_data.len() as i32;
    offset += medium_size;

    let low_offset = offset;
    let low_size = low_data.len() as i32;
    offset += low_size;

    let lowest_offset = offset;
    let lowest_size = lowest_data.len() as i32;
    offset += lowest_size;

    let phys_offset = offset;
    let phys_size = physics_data.len() as i32;
    offset += phys_size;

    let skin_offset = offset;
    let skin_size = skin_data.as_ref().map(|d| d.len() as i32).unwrap_or(0);

    let mut header_entries = vec![
        ("high_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(high_offset)),
            ("size".into(), LlsdBinValue::Integer(high_size)),
        ])),
        ("medium_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(medium_offset)),
            ("size".into(), LlsdBinValue::Integer(medium_size)),
        ])),
        ("low_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(low_offset)),
            ("size".into(), LlsdBinValue::Integer(low_size)),
        ])),
        ("lowest_lod".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(lowest_offset)),
            ("size".into(), LlsdBinValue::Integer(lowest_size)),
        ])),
        ("physics_convex".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(phys_offset)),
            ("size".into(), LlsdBinValue::Integer(phys_size)),
        ])),
    ];

    if skin_data.is_some() {
        header_entries.push(("skin".into(), LlsdBinValue::Map(vec![
            ("offset".into(), LlsdBinValue::Integer(skin_offset)),
            ("size".into(), LlsdBinValue::Integer(skin_size)),
        ])));
    }

    let header = LlsdBinValue::Map(header_entries);
    let header_bytes = serialize_llsd(&header);

    let total_size = header_bytes.len() + high_data.len() + medium_data.len()
        + low_data.len() + lowest_data.len() + physics_data.len() + skin_size as usize;
    let mut result = Vec::with_capacity(total_size);
    result.extend_from_slice(&header_bytes);
    result.extend_from_slice(&high_data);
    result.extend_from_slice(&medium_data);
    result.extend_from_slice(&low_data);
    result.extend_from_slice(&lowest_data);
    result.extend_from_slice(&physics_data);
    if let Some(sd) = &skin_data {
        result.extend_from_slice(sd);
    }

    Ok(result)
}

pub fn build_mesh_extra_params(asset_uuid: &Uuid) -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    buf.push(1u8);
    buf.extend_from_slice(&0x0030u16.to_le_bytes());
    buf.extend_from_slice(&17u32.to_le_bytes());
    buf.extend_from_slice(asset_uuid.as_bytes());
    buf.push(5u8);
    buf
}

pub fn generate_box_mesh(width: f32, height: f32, depth: f32) -> MeshGeometry {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let hd = depth / 2.0;
    let positions = vec![
        [-hw, -hh, -hd], [ hw, -hh, -hd], [ hw,  hh, -hd], [-hw,  hh, -hd],
        [-hw, -hh,  hd], [ hw, -hh,  hd], [ hw,  hh,  hd], [-hw,  hh,  hd],
    ];
    let normals = vec![
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        [0.0, 0.0,  1.0], [0.0, 0.0,  1.0], [0.0, 0.0,  1.0], [0.0, 0.0,  1.0],
    ];
    let tex_coords = vec![
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3,
        4, 6, 5, 4, 7, 6,
        0, 4, 5, 0, 5, 1,
        2, 6, 7, 2, 7, 3,
        0, 3, 7, 0, 7, 4,
        1, 5, 6, 1, 6, 2,
    ];
    MeshGeometry {
        faces: vec![MeshFace { positions, normals, tex_coords, indices, joint_weights: None, original_position_indices: None }],
        skin_info: None,
    }
}

pub fn generate_cylinder_mesh(radius: f32, height: f32, segments: u32) -> MeshGeometry {
    let seg = segments.max(6);
    let hh = height / 2.0;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();
    let mut indices = Vec::new();

    for i in 0..seg {
        let angle = (i as f32 / seg as f32) * std::f32::consts::TAU;
        let next_angle = ((i + 1) as f32 / seg as f32) * std::f32::consts::TAU;
        let (s0, c0) = (angle.sin(), angle.cos());
        let (s1, c1) = (next_angle.sin(), next_angle.cos());
        let base = positions.len() as u32;
        positions.extend_from_slice(&[
            [c0 * radius, s0 * radius, -hh],
            [c1 * radius, s1 * radius, -hh],
            [c1 * radius, s1 * radius,  hh],
            [c0 * radius, s0 * radius,  hh],
        ]);
        let nx = (c0 + c1) / 2.0;
        let ny = (s0 + s1) / 2.0;
        let nl = (nx * nx + ny * ny).sqrt().max(1e-6);
        normals.extend_from_slice(&[[nx/nl, ny/nl, 0.0]; 4]);
        let u0 = i as f32 / seg as f32;
        let u1 = (i + 1) as f32 / seg as f32;
        tex_coords.extend_from_slice(&[[u0, 0.0], [u1, 0.0], [u1, 1.0], [u0, 1.0]]);
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }

    let cap_base = positions.len() as u32;
    positions.push([0.0, 0.0, hh]);
    normals.push([0.0, 0.0, 1.0]);
    tex_coords.push([0.5, 0.5]);
    for i in 0..seg {
        let angle = (i as f32 / seg as f32) * std::f32::consts::TAU;
        positions.push([angle.cos() * radius, angle.sin() * radius, hh]);
        normals.push([0.0, 0.0, 1.0]);
        tex_coords.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }
    for i in 0..seg {
        indices.extend_from_slice(&[cap_base, cap_base + 1 + i, cap_base + 1 + (i + 1) % seg]);
    }

    let bot_base = positions.len() as u32;
    positions.push([0.0, 0.0, -hh]);
    normals.push([0.0, 0.0, -1.0]);
    tex_coords.push([0.5, 0.5]);
    for i in 0..seg {
        let angle = (i as f32 / seg as f32) * std::f32::consts::TAU;
        positions.push([angle.cos() * radius, angle.sin() * radius, -hh]);
        normals.push([0.0, 0.0, -1.0]);
        tex_coords.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }
    for i in 0..seg {
        indices.extend_from_slice(&[bot_base, bot_base + 1 + (i + 1) % seg, bot_base + 1 + i]);
    }

    MeshGeometry {
        faces: vec![MeshFace { positions, normals, tex_coords, indices, joint_weights: None, original_position_indices: None }],
        skin_info: None,
    }
}

pub fn generate_sphere_mesh(radius: f32, lat_segments: u32, lon_segments: u32) -> MeshGeometry {
    let lat = lat_segments.max(4);
    let lon = lon_segments.max(6);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=lat {
        let theta = std::f32::consts::PI * (i as f32 / lat as f32);
        let st = theta.sin();
        let ct = theta.cos();
        for j in 0..=lon {
            let phi = std::f32::consts::TAU * (j as f32 / lon as f32);
            let sp = phi.sin();
            let cp = phi.cos();
            let nx = st * cp;
            let ny = st * sp;
            let nz = ct;
            positions.push([nx * radius, ny * radius, nz * radius]);
            normals.push([nx, ny, nz]);
            tex_coords.push([j as f32 / lon as f32, i as f32 / lat as f32]);
        }
    }

    for i in 0..lat {
        for j in 0..lon {
            let a = i * (lon + 1) + j;
            let b = a + lon + 1;
            indices.extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
        }
    }

    MeshGeometry {
        faces: vec![MeshFace { positions, normals, tex_coords, indices, joint_weights: None, original_position_indices: None }],
        skin_info: None,
    }
}

pub fn generate_torus_mesh(major_radius: f32, minor_radius: f32, ring_segments: u32, tube_segments: u32) -> MeshGeometry {
    let rings = ring_segments.max(6);
    let tubes = tube_segments.max(6);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tex_coords = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=rings {
        let theta = std::f32::consts::TAU * (i as f32 / rings as f32);
        let ct = theta.cos();
        let st = theta.sin();
        for j in 0..=tubes {
            let phi = std::f32::consts::TAU * (j as f32 / tubes as f32);
            let cp = phi.cos();
            let sp = phi.sin();
            let cx = (major_radius + minor_radius * cp) * ct;
            let cy = (major_radius + minor_radius * cp) * st;
            let cz = minor_radius * sp;
            positions.push([cx, cy, cz]);
            normals.push([cp * ct, cp * st, sp]);
            tex_coords.push([i as f32 / rings as f32, j as f32 / tubes as f32]);
        }
    }

    for i in 0..rings {
        for j in 0..tubes {
            let a = i * (tubes + 1) + j;
            let b = a + tubes + 1;
            indices.extend_from_slice(&[a, b, a + 1, b, b + 1, a + 1]);
        }
    }

    MeshGeometry {
        faces: vec![MeshFace { positions, normals, tex_coords, indices, joint_weights: None, original_position_indices: None }],
        skin_info: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::parser;

    fn make_unit_cube() -> MeshGeometry {
        let positions = vec![
            [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5],
            [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
        ];
        let normals = vec![
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            [0.0, 0.0,  1.0], [0.0, 0.0,  1.0], [0.0, 0.0,  1.0], [0.0, 0.0,  1.0],
        ];
        let tex_coords = vec![
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ];
        let indices = vec![
            0, 1, 2, 0, 2, 3,
            4, 6, 5, 4, 7, 6,
            0, 4, 5, 0, 5, 1,
            2, 6, 7, 2, 7, 3,
            0, 3, 7, 0, 7, 4,
            1, 5, 6, 1, 6, 2,
        ];

        MeshGeometry {
            faces: vec![MeshFace { positions, normals, tex_coords, indices, joint_weights: None, original_position_indices: None }],
            skin_info: None,
        }
    }

    #[test]
    fn test_quantize_known_values() {
        assert_eq!(quantize_f32(0.0, 0.0, 1.0), 0);
        assert_eq!(quantize_f32(1.0, 0.0, 1.0), 65535);
        assert_eq!(quantize_f32(0.5, 0.0, 1.0), 32767);
        let dequant: f32 = 0.0 + (32767.0 / 65535.0) * (1.0 - 0.0);
        assert!((dequant - 0.5_f32).abs() < 1.0_f32 / 65535.0 + 1e-6);
    }

    #[test]
    fn test_domain_collapse_expansion() {
        let verts = vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]];
        let (min, max) = compute_domain_3d(&verts);
        for i in 0..3 {
            assert!((max[i] - min[i]) >= 0.9, "Domain not expanded for axis {}", i);
        }
    }

    #[test]
    fn test_round_trip_header_parse() {
        let geom = make_unit_cube();
        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parser::parse_mesh_header(&blob).expect("header parse failed");
        assert!(header.high_lod.is_some(), "missing high_lod");
        assert!(header.medium_lod.is_some(), "missing medium_lod");
        assert!(header.low_lod.is_some(), "missing low_lod");
        assert!(header.lowest_lod.is_some(), "missing lowest_lod");
        assert!(header.physics_convex.is_some(), "missing physics_convex");
    }

    #[test]
    fn test_round_trip_physics_convex() {
        let geom = make_unit_cube();
        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parser::parse_mesh_header(&blob).expect("header parse failed");
        let convex = parser::extract_physics_convex(&blob, &header).expect("physics parse failed");

        assert_eq!(convex.hulls.len(), 1, "expected 1 hull");
        assert_eq!(convex.hulls[0].len(), 8, "expected 8 hull vertices");

        for v in &convex.hulls[0] {
            for i in 0..3 {
                assert!(v[i] >= -0.6 && v[i] <= 0.6,
                    "vertex component {} out of range: {}", i, v[i]);
            }
        }
    }

    #[test]
    fn test_extra_params_round_trip() {
        let uuid = Uuid::new_v4();
        let params = build_mesh_extra_params(&uuid);
        let extracted = crate::mesh::extract_mesh_asset_uuid(&params);
        assert_eq!(extracted, Some(uuid));
    }

    #[test]
    fn test_empty_geometry_error() {
        let geom = MeshGeometry { faces: vec![], skin_info: None };
        assert!(encode_mesh_asset(&geom).is_err());
    }

    #[test]
    fn test_multi_face() {
        let face1 = MeshFace {
            positions: vec![[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            normals: vec![[0.577, 0.577, 0.577]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };
        let face2 = MeshFace {
            positions: vec![[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]],
            normals: vec![[-0.577, -0.577, -0.577]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };
        let geom = MeshGeometry { faces: vec![face1, face2], skin_info: None };
        let blob = encode_mesh_asset(&geom).expect("multi-face encode failed");
        let header = parser::parse_mesh_header(&blob).expect("multi-face header parse failed");
        assert!(header.high_lod.is_some());
        assert!(header.physics_convex.is_some());
    }

    #[test]
    fn test_multi_lod_encode() {
        let geom = make_unit_cube();
        let lods = MultiLodGeometry {
            high: geom.clone(),
            medium: geom.clone(),
            low: geom.clone(),
            lowest: geom.clone(),
        };
        let blob = encode_mesh_asset_multi_lod(&lods).expect("multi-lod encode failed");
        let header = parser::parse_mesh_header(&blob).expect("header parse failed");

        let h = header.high_lod.as_ref().unwrap();
        let m = header.medium_lod.as_ref().unwrap();
        let l = header.low_lod.as_ref().unwrap();
        let lo = header.lowest_lod.as_ref().unwrap();

        assert!(h.offset == 0, "high_lod offset should be 0");
        assert!(m.offset > 0, "medium_lod should have distinct offset");
        assert!(l.offset > m.offset, "low_lod should come after medium");
        assert!(lo.offset > l.offset, "lowest_lod should come after low");

        assert!(h.size > 0);
        assert!(m.size > 0);
        assert!(l.size > 0);
        assert!(lo.size > 0);
    }

    #[test]
    fn test_multi_lod_distinct_offsets() {
        let geom = make_unit_cube();
        let small_face = MeshFace {
            positions: vec![[-0.5, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 0.5]],
            normals: vec![[0.577, 0.577, 0.577]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };
        let small_geom = MeshGeometry { faces: vec![small_face], skin_info: None };

        let lods = MultiLodGeometry {
            high: geom.clone(),
            medium: geom.clone(),
            low: small_geom.clone(),
            lowest: small_geom,
        };
        let blob = encode_mesh_asset_multi_lod(&lods).expect("encode failed");
        let header = parser::parse_mesh_header(&blob).expect("header parse failed");

        let h = header.high_lod.as_ref().unwrap();
        let l = header.low_lod.as_ref().unwrap();
        assert!(h.size > l.size, "high LOD should be larger than low LOD");
    }

    #[test]
    fn test_llsd_binary_serialization() {
        let val = LlsdBinValue::Map(vec![
            ("key".into(), LlsdBinValue::Integer(42)),
        ]);
        let bytes = serialize_llsd(&val);
        assert_eq!(bytes[0], b'{');
        let count = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        assert_eq!(count, 1);
        assert_eq!(bytes[5], b'k');
        let key_len = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        assert_eq!(key_len, 3);
        assert_eq!(&bytes[10..13], b"key");
        assert_eq!(bytes[13], b'i');
        let val_i = i32::from_be_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
        assert_eq!(val_i, 42);
        assert_eq!(bytes[18], b'}');
    }
}
