use anyhow::{anyhow, bail, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;
use tracing::warn;

use super::types::{BlockRef, ConvexPhysicsData, MeshAssetHeader};

pub fn parse_mesh_header(data: &[u8]) -> Result<MeshAssetHeader> {
    if data.is_empty() {
        bail!("Empty mesh data");
    }

    let mut header = MeshAssetHeader::default();
    let mut pos = 0;

    if pos >= data.len() || data[pos] != b'{' {
        bail!("Mesh header doesn't start with LLSD binary map marker");
    }
    pos += 1;
    if pos + 4 > data.len() {
        bail!("Truncated map count");
    }
    let entry_count =
        u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;

    for _ in 0..entry_count {
        if pos >= data.len() {
            break;
        }

        let key = read_llsd_key(data, &mut pos)?;

        if pos >= data.len() {
            bail!("No value after key '{}'", key);
        }

        let block_ref = read_llsd_block_ref(data, &mut pos)?;

        match key.as_str() {
            "physics_convex" => header.physics_convex = Some(block_ref),
            "physics_mesh" => header.physics_mesh = Some(block_ref),
            "high_lod" => header.high_lod = Some(block_ref),
            "medium_lod" => header.medium_lod = Some(block_ref),
            "low_lod" => header.low_lod = Some(block_ref),
            "lowest_lod" => header.lowest_lod = Some(block_ref),
            "skin" => header.skin = Some(block_ref),
            _ => {}
        }
    }

    if pos < data.len() && data[pos] == b'}' {
        pos += 1;
    }
    header.data_start = pos;

    Ok(header)
}

fn read_llsd_key(data: &[u8], pos: &mut usize) -> Result<String> {
    if *pos >= data.len() || data[*pos] != b'k' {
        bail!("Expected LLSD key marker 'k' at {}", *pos);
    }
    *pos += 1;

    if *pos + 4 > data.len() {
        bail!("Not enough bytes for key length");
    }
    let len =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    if *pos + len > data.len() {
        bail!("Key length {} exceeds data", len);
    }
    let key = String::from_utf8_lossy(&data[*pos..*pos + len]).to_string();
    *pos += len;
    Ok(key)
}

fn read_llsd_block_ref(data: &[u8], pos: &mut usize) -> Result<BlockRef> {
    if *pos >= data.len() || data[*pos] != b'{' {
        return Err(anyhow!("Expected map for block ref at {}", *pos));
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Truncated inner map count");
    }
    let inner_count =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;

    let mut offset = 0usize;
    let mut size = 0usize;

    for _ in 0..inner_count {
        if *pos >= data.len() {
            break;
        }

        let inner_key = read_llsd_key(data, pos)?;
        let value = read_llsd_integer(data, pos)?;

        match inner_key.as_str() {
            "offset" => offset = value as usize,
            "size" => size = value as usize,
            _ => {}
        }
    }
    if *pos < data.len() && data[*pos] == b'}' {
        *pos += 1;
    }

    Ok(BlockRef { offset, size })
}

fn read_llsd_integer(data: &[u8], pos: &mut usize) -> Result<i32> {
    if *pos >= data.len() {
        bail!("No data for integer");
    }
    match data[*pos] {
        b'i' => {
            *pos += 1;
            if *pos + 4 > data.len() {
                bail!("Not enough bytes for integer");
            }
            let val =
                i32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
            *pos += 4;
            Ok(val)
        }
        _ => {
            skip_llsd_value(data, pos)?;
            Ok(0)
        }
    }
}

fn skip_llsd_value(data: &[u8], pos: &mut usize) -> Result<()> {
    if *pos >= data.len() {
        bail!("No data to skip");
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
        b's' | b'b' | b'l' => {
            *pos += 1;
            if *pos + 4 > data.len() {
                bail!("Truncated");
            }
            let len =
                u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                    as usize;
            *pos += 4 + len;
        }
        b'{' => {
            *pos += 1;
            if *pos + 4 <= data.len() {
                *pos += 4;
            }
            while *pos < data.len() && data[*pos] != b'}' {
                read_llsd_key(data, pos)?;
                skip_llsd_value(data, pos)?;
            }
            if *pos < data.len() {
                *pos += 1;
            }
        }
        b'[' => {
            *pos += 1;
            if *pos + 4 <= data.len() {
                *pos += 4;
            }
            while *pos < data.len() && data[*pos] != b']' {
                skip_llsd_value(data, pos)?;
            }
            if *pos < data.len() {
                *pos += 1;
            }
        }
        b'd' => {
            *pos += 9;
        }
        other => {
            bail!("Unknown LLSD type marker: 0x{:02x} at {}", other, *pos);
        }
    }
    if *pos > data.len() {
        bail!("Skipped past end of data");
    }
    Ok(())
}

pub fn extract_physics_convex(data: &[u8], header: &MeshAssetHeader) -> Result<ConvexPhysicsData> {
    let block_ref = header
        .physics_convex
        .as_ref()
        .ok_or_else(|| anyhow!("No physics_convex block in mesh header"))?;

    let abs_offset = header.data_start + block_ref.offset;
    if abs_offset + block_ref.size > data.len() {
        bail!(
            "physics_convex block out of bounds: abs_offset={} size={} datalen={}",
            abs_offset,
            block_ref.size,
            data.len()
        );
    }

    let compressed = &data[abs_offset..abs_offset + block_ref.size];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| anyhow!("zlib decompression failed: {}", e))?;

    parse_physics_convex_llsd(&decompressed)
}

fn parse_physics_convex_llsd(data: &[u8]) -> Result<ConvexPhysicsData> {
    let mut pos = 0;

    if pos >= data.len() || data[pos] != b'{' {
        bail!("physics_convex inner LLSD doesn't start with map marker");
    }
    pos += 1;
    if pos + 4 > data.len() {
        bail!("Truncated physics_convex map count");
    }
    let entry_count =
        u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
    pos += 4;

    let mut min = [0.0f32; 3];
    let mut max = [1.0f32; 3];
    let mut hull_list_data: Vec<u8> = Vec::new();
    let mut positions_data: Vec<u8> = Vec::new();
    let mut bounding_verts_data: Vec<u8> = Vec::new();

    for _ in 0..entry_count {
        if pos >= data.len() {
            break;
        }

        let key = read_llsd_key(data, &mut pos)?;

        match key.as_str() {
            "Min" => {
                min = read_llsd_vector3_binary(data, &mut pos)?;
            }
            "Max" => {
                max = read_llsd_vector3_binary(data, &mut pos)?;
            }
            "HullList" => {
                hull_list_data = read_llsd_binary(data, &mut pos)?;
            }
            "Positions" => {
                positions_data = read_llsd_binary(data, &mut pos)?;
            }
            "BoundingVerts" => {
                bounding_verts_data = read_llsd_binary(data, &mut pos)?;
            }
            _ => {
                if let Err(_) = skip_llsd_value(data, &mut pos) {
                    break;
                }
            }
        }
    }

    let bounding_hull = dequantize_vertices(&bounding_verts_data, &min, &max);

    let mut hulls = Vec::new();
    let mut vert_offset = 0usize;

    if hull_list_data.is_empty() {
        let total_verts = positions_data.len() / 6;
        if total_verts > 0 {
            let hull_verts = dequantize_vertices(&positions_data, &min, &max);
            hulls.push(hull_verts);
        }
    } else {
        for &count_byte in &hull_list_data {
            let vert_count = if count_byte == 0 {
                256usize
            } else {
                count_byte as usize
            };
            let byte_count = vert_count * 6;

            if vert_offset + byte_count > positions_data.len() {
                warn!(
                    "Hull vertex data truncated: need {} bytes at offset {}, have {}",
                    byte_count,
                    vert_offset,
                    positions_data.len()
                );
                break;
            }

            let hull_bytes = &positions_data[vert_offset..vert_offset + byte_count];
            let hull_verts = dequantize_vertices(hull_bytes, &min, &max);
            hulls.push(hull_verts);
            vert_offset += byte_count;
        }
    }

    if hulls.is_empty() && !bounding_hull.is_empty() {
        hulls.push(bounding_hull.clone());
    }

    Ok(ConvexPhysicsData {
        min,
        max,
        bounding_hull,
        hulls,
    })
}

fn read_llsd_vector3_binary(data: &[u8], pos: &mut usize) -> Result<[f32; 3]> {
    if *pos >= data.len() {
        bail!("No data for vector3");
    }
    match data[*pos] {
        b'b' => {
            let bytes = read_llsd_binary(data, pos)?;
            if bytes.len() == 12 {
                let x = f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]) as f32;
                let y_start = 8;
                if bytes.len() >= y_start + 4 {
                    Ok([
                        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                    ])
                } else {
                    Ok([x, 0.0, 0.0])
                }
            } else if bytes.len() == 24 {
                Ok([
                    f64::from_le_bytes(bytes[0..8].try_into()?) as f32,
                    f64::from_le_bytes(bytes[8..16].try_into()?) as f32,
                    f64::from_le_bytes(bytes[16..24].try_into()?) as f32,
                ])
            } else if bytes.len() >= 12 {
                Ok([
                    f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                    f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                ])
            } else {
                warn!("Vector3 binary unexpected size: {}", bytes.len());
                Ok([0.0, 0.0, 0.0])
            }
        }
        b'[' => {
            *pos += 1;
            if *pos + 4 <= data.len() {
                *pos += 4;
            }
            let mut vals = [0.0f32; 3];
            for i in 0..3 {
                if *pos < data.len() && data[*pos] == b']' {
                    break;
                }
                if *pos < data.len() && data[*pos] == b'r' {
                    *pos += 1;
                    if *pos + 8 <= data.len() {
                        vals[i] = f64::from_be_bytes(data[*pos..*pos + 8].try_into()?) as f32;
                        *pos += 8;
                    }
                } else if *pos < data.len() && data[*pos] == b'i' {
                    *pos += 1;
                    if *pos + 4 <= data.len() {
                        vals[i] = i32::from_be_bytes(data[*pos..*pos + 4].try_into()?) as f32;
                        *pos += 4;
                    }
                } else {
                    skip_llsd_value(data, pos)?;
                }
            }
            if *pos < data.len() && data[*pos] == b']' {
                *pos += 1;
            }
            Ok(vals)
        }
        _ => {
            skip_llsd_value(data, pos)?;
            Ok([0.0, 0.0, 0.0])
        }
    }
}

fn read_llsd_binary(data: &[u8], pos: &mut usize) -> Result<Vec<u8>> {
    if *pos >= data.len() || data[*pos] != b'b' {
        bail!("Expected binary marker 'b' at {}", *pos);
    }
    *pos += 1;
    if *pos + 4 > data.len() {
        bail!("Not enough bytes for binary length");
    }
    let len =
        u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]) as usize;
    *pos += 4;
    if *pos + len > data.len() {
        bail!("Binary data length {} exceeds available data", len);
    }
    let result = data[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(result)
}

fn dequantize_vertices(data: &[u8], min: &[f32; 3], max: &[f32; 3]) -> Vec<[f32; 3]> {
    let mut verts = Vec::new();
    let mut i = 0;
    while i + 5 < data.len() {
        let ux = u16::from_le_bytes([data[i], data[i + 1]]);
        let uy = u16::from_le_bytes([data[i + 2], data[i + 3]]);
        let uz = u16::from_le_bytes([data[i + 4], data[i + 5]]);

        let x = min[0] + (ux as f32 / 65535.0) * (max[0] - min[0]);
        let y = min[1] + (uy as f32 / 65535.0) * (max[1] - min[1]);
        let z = min[2] + (uz as f32 / 65535.0) * (max[2] - min[2]);

        verts.push([x, y, z]);
        i += 6;
    }
    verts
}

pub fn extract_skin_data(data: &[u8], header: &MeshAssetHeader) -> Result<Option<Vec<u8>>> {
    let block_ref = match &header.skin {
        Some(br) => br,
        None => return Ok(None),
    };

    let abs_offset = header.data_start + block_ref.offset;
    if abs_offset + block_ref.size > data.len() {
        bail!(
            "skin block out of bounds: abs_offset={} size={} datalen={}",
            abs_offset,
            block_ref.size,
            data.len()
        );
    }

    let compressed = &data[abs_offset..abs_offset + block_ref.size];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| anyhow!("skin zlib decompression failed: {}", e))?;

    Ok(Some(decompressed))
}

pub fn to_bullet_hull_array(data: &ConvexPhysicsData, scale: [f32; 3]) -> Vec<f32> {
    let mut result = Vec::new();

    result.push(data.hulls.len() as f32);

    for hull in &data.hulls {
        result.push(hull.len() as f32);
        result.push(0.0); // centroid X
        result.push(0.0); // centroid Y
        result.push(0.0); // centroid Z

        for vert in hull {
            result.push(vert[0] * scale[0]);
            result.push(vert[1] * scale[1]);
            result.push(vert[2] * scale[2]);
        }
    }

    result
}
