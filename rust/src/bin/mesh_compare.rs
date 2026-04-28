use anyhow::Result;
use flate2::read::ZlibDecoder;
use sqlx::Row;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://opensim@localhost/gaiagrid".to_string());

    println!("Connecting to: {}", db_url);
    let pool = sqlx::PgPool::connect(&db_url).await?;

    let rows: Vec<_> = sqlx::query(
        "SELECT a.id, a.name, a.data, a.create_time FROM assets a WHERE a.assettype = 49 ORDER BY a.create_time DESC LIMIT 20"
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        println!("No mesh assets found.");
        return Ok(());
    }

    println!("\n=== Mesh Asset Comparison Tool ===\n");
    println!("Found {} mesh assets (type 49)\n", rows.len());

    for row in &rows {
        let id: uuid::Uuid = row.get("id");
        let name: String = row.get("name");
        let data: Vec<u8> = row.get("data");
        let create_time: i32 = row.get("create_time");

        println!("━━━ {} ━━━", id);
        println!(
            "  Name: {}  |  Size: {} bytes  |  Created: {}",
            name,
            data.len(),
            create_time
        );

        if data.is_empty() {
            println!("  EMPTY ASSET DATA\n");
            continue;
        }

        println!("  First 32 bytes: {:02x?}", &data[..data.len().min(32)]);

        match parse_mesh_header_raw(&data) {
            Ok((header_size, sections)) => {
                println!("  Header size: {} bytes (LLSD binary)", header_size);
                for (name, offset, size) in &sections {
                    println!("    {}: offset={} size={}", name, offset, size);
                }

                for (sec_name, sec_offset, sec_size) in &sections {
                    if *sec_size == 0 {
                        continue;
                    }
                    let abs_start = header_size + *sec_offset as usize;
                    let abs_end = abs_start + *sec_size as usize;
                    if abs_end > data.len() {
                        println!(
                            "    {} OVERFLOW: need {} bytes but asset is {} bytes",
                            sec_name,
                            abs_end,
                            data.len()
                        );
                        continue;
                    }
                    let section_data = &data[abs_start..abs_end];

                    if sec_name.contains("lod") {
                        match zlib_decompress(section_data) {
                            Ok(decompressed) => {
                                println!(
                                    "    {} decompressed: {} → {} bytes",
                                    sec_name,
                                    sec_size,
                                    decompressed.len()
                                );
                                println!(
                                    "    {} first 64 bytes: {:02x?}",
                                    sec_name,
                                    &decompressed[..decompressed.len().min(64)]
                                );

                                let face_count = count_llsd_array_items(&decompressed);
                                println!("    {} face count: {}", sec_name, face_count);

                                if !decompressed.is_empty() {
                                    let first_byte = decompressed[0];
                                    println!(
                                        "    {} top-level LLSD type: '{}' (0x{:02x}) {}",
                                        sec_name,
                                        first_byte as char,
                                        first_byte,
                                        match first_byte {
                                            b'[' => "= Array (CORRECT)",
                                            b'{' => "= Map (WRONG — should be Array)",
                                            _ => "= UNKNOWN (WRONG)",
                                        }
                                    );
                                }
                            }
                            Err(e) => {
                                println!(
                                    "    {} zlib FAILED: {} (first 8 bytes: {:02x?})",
                                    sec_name,
                                    e,
                                    &section_data[..section_data.len().min(8)]
                                );
                            }
                        }
                    }

                    if sec_name == "physics_convex" {
                        match zlib_decompress(section_data) {
                            Ok(decompressed) => {
                                println!("    physics_convex decompressed: {} bytes, first byte: '{}' (0x{:02x})",
                                    decompressed.len(), decompressed[0] as char, decompressed[0]);
                            }
                            Err(e) => {
                                println!("    physics_convex zlib FAILED: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("  HEADER PARSE FAILED: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

fn count_llsd_array_items(data: &[u8]) -> i32 {
    if data.len() < 5 || data[0] != b'[' {
        return -1;
    }
    let count = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    count as i32
}

fn parse_mesh_header_raw(data: &[u8]) -> Result<(usize, Vec<(String, i32, i32)>)> {
    let mut pos = 0;

    if data.len() > 17 && &data[..17] == b"<? LLSD/Binary ?>" {
        pos = 18;
    }

    if pos >= data.len() || data[pos] != b'{' {
        anyhow::bail!(
            "Expected LLSD Map start '{{' at byte {}, got 0x{:02x}",
            pos,
            data.get(pos).unwrap_or(&0)
        );
    }

    let header_start = pos;
    let (map_entries, bytes_consumed) = parse_llsd_map_raw(&data[header_start..])?;
    let header_size = header_start + bytes_consumed;

    let mut sections = Vec::new();
    let section_names = [
        "high_lod",
        "medium_lod",
        "low_lod",
        "lowest_lod",
        "physics_convex",
        "physics_mesh",
        "skin",
    ];

    for name in section_names {
        if let Some(sub_map) = map_entries.get(name) {
            let offset = sub_map.get("offset").copied().unwrap_or(-1);
            let size = sub_map.get("size").copied().unwrap_or(-1);
            sections.push((name.to_string(), offset, size));
        }
    }

    Ok((header_size, sections))
}

fn parse_llsd_map_raw(
    data: &[u8],
) -> Result<(
    std::collections::HashMap<String, std::collections::HashMap<String, i32>>,
    usize,
)> {
    use std::collections::HashMap;

    if data.is_empty() || data[0] != b'{' {
        anyhow::bail!("Not a map");
    }

    let count = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    let mut pos = 5;
    let mut result: HashMap<String, HashMap<String, i32>> = HashMap::new();

    for _ in 0..count {
        if pos >= data.len() {
            break;
        }

        if data[pos] != b'k' {
            anyhow::bail!(
                "Expected key marker 'k' at pos {}, got 0x{:02x}",
                pos,
                data[pos]
            );
        }
        pos += 1;
        let key_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let key = std::str::from_utf8(&data[pos..pos + key_len])?.to_string();
        pos += key_len;

        if pos < data.len() && data[pos] == b'{' {
            let sub_count =
                u32::from_be_bytes([data[pos + 1], data[pos + 2], data[pos + 3], data[pos + 4]])
                    as usize;
            pos += 5;
            let mut sub_map = HashMap::new();
            for _ in 0..sub_count {
                if pos >= data.len() {
                    break;
                }
                if data[pos] == b'k' {
                    pos += 1;
                    let sk_len = u32::from_be_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let sk = std::str::from_utf8(&data[pos..pos + sk_len])?.to_string();
                    pos += sk_len;

                    if pos < data.len() && data[pos] == b'i' {
                        pos += 1;
                        let val = i32::from_be_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                        sub_map.insert(sk, val);
                    } else {
                        let (_, skipped) = skip_llsd_value(&data[pos..])?;
                        pos += skipped;
                    }
                } else {
                    break;
                }
            }
            if pos < data.len() && data[pos] == b'}' {
                pos += 1;
            }
            result.insert(key, sub_map);
        } else {
            let (_, skipped) = skip_llsd_value(&data[pos..])?;
            pos += skipped;
            result.insert(key, HashMap::new());
        }
    }

    if pos < data.len() && data[pos] == b'}' {
        pos += 1;
    }

    Ok((result, pos))
}

fn skip_llsd_value(data: &[u8]) -> Result<(u8, usize)> {
    if data.is_empty() {
        anyhow::bail!("No data to skip");
    }
    let tag = data[0];
    match tag {
        b'!' => Ok((tag, 1)),
        b'1' | b'0' => Ok((tag, 1)),
        b'i' => Ok((tag, 5)),
        b'r' => Ok((tag, 9)),
        b'u' => Ok((tag, 17)),
        b's' | b'l' => {
            let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
            Ok((tag, 5 + len))
        }
        b'b' => {
            let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
            Ok((tag, 5 + len))
        }
        b'd' => Ok((tag, 9)),
        b'{' => {
            let count = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
            let mut pos = 5;
            for _ in 0..count {
                if pos >= data.len() {
                    break;
                }
                if data[pos] == b'k' {
                    pos += 1;
                    let kl = u32::from_be_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4 + kl;
                }
                let (_, s) = skip_llsd_value(&data[pos..])?;
                pos += s;
            }
            if pos < data.len() && data[pos] == b'}' {
                pos += 1;
            }
            Ok((tag, pos))
        }
        b'[' => {
            let count = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
            let mut pos = 5;
            for _ in 0..count {
                let (_, s) = skip_llsd_value(&data[pos..])?;
                pos += s;
            }
            if pos < data.len() && data[pos] == b']' {
                pos += 1;
            }
            Ok((tag, pos))
        }
        _ => {
            anyhow::bail!("Unknown LLSD type tag: 0x{:02x} at offset", tag);
        }
    }
}
