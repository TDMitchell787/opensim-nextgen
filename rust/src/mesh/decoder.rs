use anyhow::{anyhow, bail, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;
use tracing::warn;

use super::encoder::{JointInfluence, MeshFace, MeshGeometry, MeshSkinInfo, VertexWeights};
use super::types::MeshAssetHeader;

pub fn decode_lod_geometry(data: &[u8], header: &MeshAssetHeader) -> Result<MeshGeometry> {
    let block_ref = header
        .high_lod
        .as_ref()
        .ok_or_else(|| anyhow!("No high_lod block in mesh header"))?;
    decode_lod_block(data, header.data_start, block_ref, data, header)
}

pub fn decode_lod_geometry_level(
    data: &[u8],
    header: &MeshAssetHeader,
    level: LodLevel,
) -> Result<MeshGeometry> {
    let block_ref = match level {
        LodLevel::High => header.high_lod.as_ref(),
        LodLevel::Medium => header.medium_lod.as_ref(),
        LodLevel::Low => header.low_lod.as_ref(),
        LodLevel::Lowest => header.lowest_lod.as_ref(),
    }
    .ok_or_else(|| anyhow!("No {:?} LOD block in mesh header", level))?;
    decode_lod_block(data, header.data_start, block_ref, data, header)
}

#[derive(Debug, Clone, Copy)]
pub enum LodLevel {
    High,
    Medium,
    Low,
    Lowest,
}

fn decode_lod_block(
    _full_data: &[u8],
    data_start: usize,
    block_ref: &super::types::BlockRef,
    asset_data: &[u8],
    header: &MeshAssetHeader,
) -> Result<MeshGeometry> {
    let abs_offset = data_start + block_ref.offset;
    if abs_offset + block_ref.size > asset_data.len() {
        bail!(
            "LOD block out of bounds: offset={} size={} datalen={}",
            abs_offset,
            block_ref.size,
            asset_data.len()
        );
    }

    let compressed = &asset_data[abs_offset..abs_offset + block_ref.size];
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| anyhow!("LOD zlib decompression failed: {}", e))?;

    let faces = parse_lod_llsd(&decompressed)?;

    let skin_info = decode_skin_block(asset_data, header)?;

    Ok(MeshGeometry { faces, skin_info })
}

fn parse_lod_llsd(data: &[u8]) -> Result<Vec<MeshFace>> {
    let mut pos = 0;

    if pos >= data.len() || data[pos] != b'[' {
        bail!(
            "LOD data doesn't start with LLSD array marker '[', got 0x{:02x} at {}",
            data.get(pos).copied().unwrap_or(0),
            pos
        );
    }
    pos += 1;

    if pos + 4 > data.len() {
        bail!("Truncated face count");
    }
    let face_count =
        u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;

    let mut faces = Vec::with_capacity(face_count);
    for i in 0..face_count {
        let face = parse_face_map(data, &mut pos)
            .map_err(|e| anyhow!("Failed to parse face {}: {}", i, e))?;
        faces.push(face);
    }

    Ok(faces)
}

fn parse_face_map(data: &[u8], pos: &mut usize) -> Result<MeshFace> {
    if *pos >= data.len() || data[*pos] != b'{' {
        bail!(
            "Expected map marker '{{' at {}, got 0x{:02x}",
            *pos,
            data.get(*pos).copied().unwrap_or(0)
        );
    }
    *pos += 1;

    if *pos + 4 > data.len() {
        bail!("Truncated face map entry count");
    }
    let entry_count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut position_bytes: Vec<u8> = Vec::new();
    let mut position_domain: Option<([f64; 3], [f64; 3])> = None;
    let mut normal_bytes: Vec<u8> = Vec::new();
    let mut texcoord_bytes: Vec<u8> = Vec::new();
    let mut texcoord_domain: Option<([f64; 2], [f64; 2])> = None;
    let mut triangle_bytes: Vec<u8> = Vec::new();
    let mut weight_bytes: Vec<u8> = Vec::new();

    for _ in 0..entry_count {
        if *pos >= data.len() {
            break;
        }
        let key = read_llsd_key(data, pos)?;
        match key.as_str() {
            "Position" => {
                position_bytes = read_llsd_binary(data, pos)?;
            }
            "PositionDomain" => {
                position_domain = Some(read_domain_3d(data, pos)?);
            }
            "Normal" => {
                normal_bytes = read_llsd_binary(data, pos)?;
            }
            "TexCoord0" => {
                texcoord_bytes = read_llsd_binary(data, pos)?;
            }
            "TexCoord0Domain" => {
                texcoord_domain = Some(read_domain_2d(data, pos)?);
            }
            "TriangleList" => {
                triangle_bytes = read_llsd_binary(data, pos)?;
            }
            "Weights" => {
                weight_bytes = read_llsd_binary(data, pos)?;
            }
            _ => {
                skip_llsd_value(data, pos)?;
            }
        }
    }

    if *pos < data.len() && data[*pos] == b'}' {
        *pos += 1;
    }

    let (pos_min, pos_max) = position_domain.unwrap_or(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));

    let vertex_count = position_bytes.len() / 6;
    if vertex_count == 0 {
        bail!("Face has no position data");
    }

    let positions = dequantize_vec3(&position_bytes, &pos_min, &pos_max);

    let normals = if normal_bytes.len() == vertex_count * 6 {
        let norm_min = [-1.0f64, -1.0, -1.0];
        let norm_max = [1.0f64, 1.0, 1.0];
        dequantize_vec3(&normal_bytes, &norm_min, &norm_max)
    } else {
        vec![[0.0f32, 0.0, 1.0]; vertex_count]
    };

    let (tc_min, tc_max) = texcoord_domain.unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let tex_coords = if texcoord_bytes.len() == vertex_count * 4 {
        dequantize_vec2(&texcoord_bytes, &tc_min, &tc_max)
    } else {
        vec![[0.0f32, 0.0]; vertex_count]
    };

    let indices = decode_triangle_indices(&triangle_bytes);

    let joint_weights = if !weight_bytes.is_empty() {
        Some(decode_vertex_weights(&weight_bytes, vertex_count))
    } else {
        None
    };

    Ok(MeshFace {
        positions,
        normals,
        tex_coords,
        indices,
        joint_weights,
        original_position_indices: None,
    })
}

fn dequantize_vec3(data: &[u8], min: &[f64; 3], max: &[f64; 3]) -> Vec<[f32; 3]> {
    let count = data.len() / 6;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let base = i * 6;
        let ux = u16::from_le_bytes([data[base], data[base + 1]]);
        let uy = u16::from_le_bytes([data[base + 2], data[base + 3]]);
        let uz = u16::from_le_bytes([data[base + 4], data[base + 5]]);
        let x = min[0] + (ux as f64 / 65535.0) * (max[0] - min[0]);
        let y = min[1] + (uy as f64 / 65535.0) * (max[1] - min[1]);
        let z = min[2] + (uz as f64 / 65535.0) * (max[2] - min[2]);
        result.push([x as f32, y as f32, z as f32]);
    }
    result
}

fn dequantize_vec2(data: &[u8], min: &[f64; 2], max: &[f64; 2]) -> Vec<[f32; 2]> {
    let count = data.len() / 4;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let base = i * 4;
        let uu = u16::from_le_bytes([data[base], data[base + 1]]);
        let uv = u16::from_le_bytes([data[base + 2], data[base + 3]]);
        let u = min[0] + (uu as f64 / 65535.0) * (max[0] - min[0]);
        let v = min[1] + (uv as f64 / 65535.0) * (max[1] - min[1]);
        result.push([u as f32, v as f32]);
    }
    result
}

fn decode_triangle_indices(data: &[u8]) -> Vec<u32> {
    let count = data.len() / 2;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let base = i * 2;
        let idx = u16::from_le_bytes([data[base], data[base + 1]]);
        result.push(idx as u32);
    }
    result
}

fn decode_vertex_weights(data: &[u8], vertex_count: usize) -> Vec<VertexWeights> {
    let mut weights = Vec::with_capacity(vertex_count);
    let mut pos = 0;

    for _ in 0..vertex_count {
        let mut influences = Vec::new();
        while pos < data.len() {
            let joint_index = data[pos];
            if joint_index == 0xFF {
                pos += 1;
                break;
            }
            if pos + 3 > data.len() {
                break;
            }
            let w16 = u16::from_le_bytes([data[pos + 1], data[pos + 2]]);
            let weight = w16 as f32 / 65535.0;
            influences.push(JointInfluence {
                joint_index,
                weight,
            });
            pos += 3;
            if influences.len() >= 4 {
                break;
            }
        }
        weights.push(VertexWeights { influences });
    }

    if weights.len() < vertex_count {
        weights.resize_with(vertex_count, || VertexWeights {
            influences: Vec::new(),
        });
    }

    weights
}

fn decode_skin_block(data: &[u8], header: &MeshAssetHeader) -> Result<Option<MeshSkinInfo>> {
    let block_ref = match &header.skin {
        Some(br) => br,
        None => return Ok(None),
    };

    let abs_offset = header.data_start + block_ref.offset;
    if abs_offset + block_ref.size > data.len() {
        warn!(
            "Skin block out of bounds: offset={} size={} datalen={}",
            abs_offset,
            block_ref.size,
            data.len()
        );
        return Ok(None);
    }

    let compressed = &data[abs_offset..abs_offset + block_ref.size];
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    if let Err(e) = decoder.read_to_end(&mut decompressed) {
        warn!("Skin block zlib decompression failed: {}", e);
        return Ok(None);
    }

    parse_skin_llsd(&decompressed)
}

fn parse_skin_llsd(data: &[u8]) -> Result<Option<MeshSkinInfo>> {
    let mut pos = 0;

    if pos >= data.len() || data[pos] != b'{' {
        bail!("Skin data doesn't start with LLSD map marker");
    }
    pos += 1;

    if pos + 4 > data.len() {
        bail!("Truncated skin map count");
    }
    let entry_count =
        u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;

    let mut joint_names: Vec<String> = Vec::new();
    let mut inverse_bind_matrices: Vec<[f32; 16]> = Vec::new();
    let mut bind_shape_matrix = [0.0f32; 16];
    let mut pelvis_offset = 0.0f32;

    for _ in 0..entry_count {
        if pos >= data.len() {
            break;
        }
        let key = read_llsd_key(data, &mut pos)?;
        match key.as_str() {
            "joint_names" => {
                joint_names = read_llsd_string_array(data, &mut pos)?;
            }
            "inverse_bind_matrix" => {
                inverse_bind_matrices = read_llsd_matrix_array(data, &mut pos)?;
            }
            "bind_shape_matrix" => {
                let vals = read_llsd_real_array(data, &mut pos)?;
                if vals.len() >= 16 {
                    for i in 0..16 {
                        bind_shape_matrix[i] = vals[i] as f32;
                    }
                } else {
                    skip_remaining_value(data, &mut pos, &vals);
                }
            }
            "pelvis_offset" => {
                pelvis_offset = read_llsd_real(data, &mut pos)? as f32;
            }
            _ => {
                skip_llsd_value(data, &mut pos)?;
            }
        }
    }

    if joint_names.is_empty() {
        return Ok(None);
    }

    Ok(Some(MeshSkinInfo {
        joint_names,
        inverse_bind_matrices,
        bind_shape_matrix,
        pelvis_offset,
    }))
}

fn skip_remaining_value(_data: &[u8], _pos: &mut usize, _vals: &[f64]) {}

fn read_llsd_key(data: &[u8], pos: &mut usize) -> Result<String> {
    if *pos >= data.len() || data[*pos] != b'k' {
        bail!(
            "Expected LLSD key marker 'k' at {}, got 0x{:02x}",
            *pos,
            data.get(*pos).copied().unwrap_or(0)
        );
    }
    *pos += 1;

    if *pos + 4 > data.len() {
        bail!("Not enough bytes for key length at {}", *pos);
    }
    let len =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    if *pos + len > data.len() {
        bail!("Key length {} exceeds data at {}", len, *pos);
    }
    let key = String::from_utf8_lossy(&data[*pos..*pos + len]).to_string();
    *pos += len;
    Ok(key)
}

fn read_llsd_binary(data: &[u8], pos: &mut usize) -> Result<Vec<u8>> {
    if *pos >= data.len() || data[*pos] != b'b' {
        bail!(
            "Expected binary marker 'b' at {}, got 0x{:02x}",
            *pos,
            data.get(*pos).copied().unwrap_or(0)
        );
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Not enough bytes for binary length at {}", *pos);
    }
    let len =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if *pos + len > data.len() {
        bail!("Binary data length {} exceeds data at {}", len, *pos);
    }
    let result = data[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(result)
}

fn read_llsd_real(data: &[u8], pos: &mut usize) -> Result<f64> {
    if *pos >= data.len() {
        bail!("No data for real at {}", *pos);
    }
    if data[*pos] == b'r' {
        *pos += 1;
        if *pos + 8 > data.len() {
            bail!("Not enough bytes for f64 at {}", *pos);
        }
        let val = f64::from_be_bytes(data[*pos..*pos + 8].try_into()?);
        *pos += 8;
        Ok(val)
    } else if data[*pos] == b'i' {
        *pos += 1;
        if *pos + 4 > data.len() {
            bail!("Not enough bytes for integer-as-real at {}", *pos);
        }
        let val = i32::from_be_bytes(data[*pos..*pos + 4].try_into()?);
        *pos += 4;
        Ok(val as f64)
    } else {
        skip_llsd_value(data, pos)?;
        Ok(0.0)
    }
}

fn read_llsd_string_array(data: &[u8], pos: &mut usize) -> Result<Vec<String>> {
    if *pos >= data.len() || data[*pos] != b'[' {
        bail!("Expected array marker '[' at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated array count at {}", *pos);
    }
    let count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if *pos >= data.len() {
            break;
        }
        if data[*pos] == b's' {
            *pos += 1;
            if *pos + 4 > data.len() {
                break;
            }
            let len =
                u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                    as usize;
            *pos += 4;
            if *pos + len > data.len() {
                break;
            }
            let s = String::from_utf8_lossy(&data[*pos..*pos + len]).to_string();
            *pos += len;
            result.push(s);
        } else {
            skip_llsd_value(data, pos)?;
            result.push(String::new());
        }
    }

    if *pos < data.len() && data[*pos] == b']' {
        *pos += 1;
    }

    Ok(result)
}

fn read_llsd_real_array(data: &[u8], pos: &mut usize) -> Result<Vec<f64>> {
    if *pos >= data.len() || data[*pos] != b'[' {
        bail!("Expected array marker '[' at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated array count at {}", *pos);
    }
    let count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if *pos >= data.len() || data[*pos] == b']' {
            break;
        }
        result.push(read_llsd_real(data, pos)?);
    }

    if *pos < data.len() && data[*pos] == b']' {
        *pos += 1;
    }

    Ok(result)
}

fn read_llsd_matrix_array(data: &[u8], pos: &mut usize) -> Result<Vec<[f32; 16]>> {
    if *pos >= data.len() || data[*pos] != b'[' {
        bail!("Expected array marker '[' at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated array count at {}", *pos);
    }
    let count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if *pos >= data.len() || data[*pos] == b']' {
            break;
        }
        let vals = read_llsd_real_array(data, pos)?;
        let mut mat = [0.0f32; 16];
        for i in 0..16.min(vals.len()) {
            mat[i] = vals[i] as f32;
        }
        result.push(mat);
    }

    if *pos < data.len() && data[*pos] == b']' {
        *pos += 1;
    }

    Ok(result)
}

fn read_domain_3d(data: &[u8], pos: &mut usize) -> Result<([f64; 3], [f64; 3])> {
    if *pos >= data.len() || data[*pos] != b'{' {
        bail!("Expected domain map marker at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated domain map count");
    }
    let entry_count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut min = [0.0f64; 3];
    let mut max = [1.0f64; 3];

    for _ in 0..entry_count {
        if *pos >= data.len() {
            break;
        }
        let key = read_llsd_key(data, pos)?;
        let vals = read_llsd_real_array(data, pos)?;
        match key.as_str() {
            "Min" => {
                for i in 0..3.min(vals.len()) {
                    min[i] = vals[i];
                }
            }
            "Max" => {
                for i in 0..3.min(vals.len()) {
                    max[i] = vals[i];
                }
            }
            _ => {}
        }
    }

    if *pos < data.len() && data[*pos] == b'}' {
        *pos += 1;
    }

    Ok((min, max))
}

fn read_domain_2d(data: &[u8], pos: &mut usize) -> Result<([f64; 2], [f64; 2])> {
    if *pos >= data.len() || data[*pos] != b'{' {
        bail!("Expected domain map marker at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated domain map count");
    }
    let entry_count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut min = [0.0f64; 2];
    let mut max = [1.0f64; 2];

    for _ in 0..entry_count {
        if *pos >= data.len() {
            break;
        }
        let key = read_llsd_key(data, pos)?;
        let vals = read_llsd_real_array(data, pos)?;
        match key.as_str() {
            "Min" => {
                for i in 0..2.min(vals.len()) {
                    min[i] = vals[i];
                }
            }
            "Max" => {
                for i in 0..2.min(vals.len()) {
                    max[i] = vals[i];
                }
            }
            _ => {}
        }
    }

    if *pos < data.len() && data[*pos] == b'}' {
        *pos += 1;
    }

    Ok((min, max))
}

fn skip_llsd_value(data: &[u8], pos: &mut usize) -> Result<()> {
    if *pos >= data.len() {
        bail!("No data to skip at {}", *pos);
    }
    match data[*pos] {
        0x00 | b'!' | b'1' | b'0' => {
            *pos += 1;
        }
        b'i' => {
            *pos += 5;
        }
        b'r' => {
            *pos += 9;
        }
        b'u' => {
            *pos += 17;
        }
        b'd' => {
            *pos += 9;
        }
        b's' | b'b' | b'l' => {
            *pos += 1;
            if *pos + 4 > data.len() {
                bail!("Truncated length at {}", *pos);
            }
            let len =
                u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                    as usize;
            *pos += 4 + len;
        }
        b'{' => {
            *pos += 1;
            if *pos + 4 > data.len() {
                bail!("Truncated map count at {}", *pos);
            }
            let count =
                u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                    as usize;
            *pos += 4;
            for _ in 0..count {
                if *pos >= data.len() {
                    break;
                }
                read_llsd_key(data, pos)?;
                skip_llsd_value(data, pos)?;
            }
            if *pos < data.len() && data[*pos] == b'}' {
                *pos += 1;
            }
        }
        b'[' => {
            *pos += 1;
            if *pos + 4 > data.len() {
                bail!("Truncated array count at {}", *pos);
            }
            let count =
                u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                    as usize;
            *pos += 4;
            for _ in 0..count {
                if *pos >= data.len() {
                    break;
                }
                skip_llsd_value(data, pos)?;
            }
            if *pos < data.len() && data[*pos] == b']' {
                *pos += 1;
            }
        }
        other => {
            bail!("Unknown LLSD type marker: 0x{:02x} at {}", other, *pos);
        }
    }
    if *pos > data.len() {
        bail!("Skipped past end of data at {}", *pos);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::encoder::{
        encode_mesh_asset, JointInfluence, MeshFace, MeshGeometry, MeshSkinInfo, VertexWeights,
    };
    use crate::mesh::parser::parse_mesh_header;

    fn make_test_face() -> MeshFace {
        MeshFace {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            normals: vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.5, 0.5]],
            indices: vec![0, 1, 2, 0, 2, 3],
            joint_weights: None,
            original_position_indices: None,
        }
    }

    fn make_skinned_face() -> (MeshFace, MeshSkinInfo) {
        let face = MeshFace {
            positions: vec![
                [-0.5, -0.5, 0.0],
                [0.5, -0.5, 0.0],
                [0.5, 0.5, 0.0],
                [-0.5, 0.5, 0.0],
            ],
            normals: vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            joint_weights: Some(vec![
                VertexWeights {
                    influences: vec![
                        JointInfluence {
                            joint_index: 0,
                            weight: 0.8,
                        },
                        JointInfluence {
                            joint_index: 1,
                            weight: 0.2,
                        },
                    ],
                },
                VertexWeights {
                    influences: vec![
                        JointInfluence {
                            joint_index: 0,
                            weight: 0.5,
                        },
                        JointInfluence {
                            joint_index: 1,
                            weight: 0.5,
                        },
                    ],
                },
                VertexWeights {
                    influences: vec![
                        JointInfluence {
                            joint_index: 1,
                            weight: 0.9,
                        },
                        JointInfluence {
                            joint_index: 2,
                            weight: 0.1,
                        },
                    ],
                },
                VertexWeights {
                    influences: vec![JointInfluence {
                        joint_index: 1,
                        weight: 1.0,
                    }],
                },
            ]),
            original_position_indices: None,
        };

        let skin = MeshSkinInfo {
            joint_names: vec!["mPelvis".into(), "mTorso".into(), "mChest".into()],
            inverse_bind_matrices: vec![
                [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
                [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.0, 1.0,
                ],
                [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0,
                ],
            ],
            bind_shape_matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            pelvis_offset: 0.0,
        };

        (face, skin)
    }

    #[test]
    fn test_round_trip_simple_face() {
        let geom = MeshGeometry {
            faces: vec![make_test_face()],
            skin_info: None,
        };

        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parse_mesh_header(&blob).expect("header parse failed");
        let decoded = decode_lod_geometry(&blob, &header).expect("decode failed");

        assert_eq!(decoded.faces.len(), 1, "face count mismatch");
        let orig = &geom.faces[0];
        let dec = &decoded.faces[0];

        assert_eq!(
            dec.positions.len(),
            orig.positions.len(),
            "vertex count mismatch"
        );
        assert_eq!(
            dec.normals.len(),
            orig.normals.len(),
            "normal count mismatch"
        );
        assert_eq!(
            dec.tex_coords.len(),
            orig.tex_coords.len(),
            "texcoord count mismatch"
        );
        assert_eq!(
            dec.indices.len(),
            orig.indices.len(),
            "index count mismatch"
        );

        for i in 0..orig.positions.len() {
            for axis in 0..3 {
                let diff = (dec.positions[i][axis] - orig.positions[i][axis]).abs();
                assert!(
                    diff < 0.01,
                    "position[{}][{}] diff={} (orig={}, dec={})",
                    i,
                    axis,
                    diff,
                    orig.positions[i][axis],
                    dec.positions[i][axis]
                );
            }
        }

        for i in 0..orig.tex_coords.len() {
            for axis in 0..2 {
                let diff = (dec.tex_coords[i][axis] - orig.tex_coords[i][axis]).abs();
                assert!(
                    diff < 0.01,
                    "texcoord[{}][{}] diff={} (orig={}, dec={})",
                    i,
                    axis,
                    diff,
                    orig.tex_coords[i][axis],
                    dec.tex_coords[i][axis]
                );
            }
        }

        for i in 0..orig.indices.len() {
            assert_eq!(dec.indices[i], orig.indices[i], "index[{}] mismatch", i);
        }
    }

    #[test]
    fn test_round_trip_multi_face() {
        let face1 = MeshFace {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };
        let face2 = MeshFace {
            positions: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };

        let geom = MeshGeometry {
            faces: vec![face1, face2],
            skin_info: None,
        };
        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parse_mesh_header(&blob).expect("header parse failed");
        let decoded = decode_lod_geometry(&blob, &header).expect("decode failed");

        assert_eq!(decoded.faces.len(), 2, "expected 2 faces");
        assert_eq!(decoded.faces[0].positions.len(), 3);
        assert_eq!(decoded.faces[1].positions.len(), 3);

        let diff = (decoded.faces[1].positions[0][0] - 2.0).abs();
        assert!(
            diff < 0.01,
            "face2 position X should be ~2.0, got {}",
            decoded.faces[1].positions[0][0]
        );
    }

    #[test]
    fn test_round_trip_with_skin() {
        let (face, skin) = make_skinned_face();
        let geom = MeshGeometry {
            faces: vec![face],
            skin_info: Some(skin.clone()),
        };

        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parse_mesh_header(&blob).expect("header parse failed");
        let decoded = decode_lod_geometry(&blob, &header).expect("decode failed");

        assert_eq!(decoded.faces.len(), 1);
        assert_eq!(decoded.faces[0].positions.len(), 4);

        let dec_skin = decoded
            .skin_info
            .as_ref()
            .expect("skin_info should be present");
        assert_eq!(dec_skin.joint_names, vec!["mPelvis", "mTorso", "mChest"]);
        assert_eq!(dec_skin.inverse_bind_matrices.len(), 3);

        for i in 0..16 {
            let diff = (dec_skin.bind_shape_matrix[i] - skin.bind_shape_matrix[i]).abs();
            assert!(diff < 0.001, "bind_shape_matrix[{}] diff={}", i, diff);
        }

        let dec_weights = decoded.faces[0]
            .joint_weights
            .as_ref()
            .expect("weights should be present");
        assert_eq!(dec_weights.len(), 4);
        assert_eq!(dec_weights[0].influences[0].joint_index, 0);
        assert!(
            dec_weights[0].influences[0].weight > 0.7,
            "first vertex should be ~0.8 on joint 0"
        );
        assert_eq!(dec_weights[3].influences[0].joint_index, 1);
        assert!(
            dec_weights[3].influences[0].weight > 0.95,
            "last vertex should be ~1.0 on joint 1"
        );
    }

    #[test]
    fn test_round_trip_preserves_normals() {
        let face = MeshFace {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![
                [0.577, 0.577, 0.577],
                [-0.707, 0.707, 0.0],
                [0.0, -1.0, 0.0],
            ],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };

        let geom = MeshGeometry {
            faces: vec![face],
            skin_info: None,
        };
        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parse_mesh_header(&blob).expect("header parse failed");
        let decoded = decode_lod_geometry(&blob, &header).expect("decode failed");

        let orig_n = &geom.faces[0].normals;
        let dec_n = &decoded.faces[0].normals;
        for i in 0..3 {
            for axis in 0..3 {
                let diff = (dec_n[i][axis] - orig_n[i][axis]).abs();
                assert!(diff < 0.01, "normal[{}][{}] diff={}", i, axis, diff);
            }
        }
    }

    #[test]
    fn test_decode_lod_levels() {
        let geom = MeshGeometry {
            faces: vec![make_test_face()],
            skin_info: None,
        };

        let blob = encode_mesh_asset(&geom).expect("encode failed");
        let header = parse_mesh_header(&blob).expect("header parse failed");

        for level in [
            LodLevel::High,
            LodLevel::Medium,
            LodLevel::Low,
            LodLevel::Lowest,
        ] {
            let decoded = decode_lod_geometry_level(&blob, &header, level)
                .expect(&format!("decode {:?} failed", level));
            assert_eq!(decoded.faces.len(), 1);
            assert_eq!(decoded.faces[0].positions.len(), 4);
        }
    }
}
