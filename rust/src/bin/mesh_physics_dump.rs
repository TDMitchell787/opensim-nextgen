use anyhow::Result;
use opensim_next::mesh::parser;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://opensim@localhost/opensim_pg".to_string());

    println!("Connecting to: {}", db_url);
    let pool = sqlx::PgPool::connect(&db_url).await?;

    let specific_id = std::env::args().nth(1);

    let rows = if let Some(ref id) = specific_id {
        let uuid: uuid::Uuid = id.parse()?;
        sqlx::query("SELECT id, name, data FROM assets WHERE id = $1::uuid AND assettype = 49")
            .bind(uuid)
            .fetch_all(&pool)
            .await?
    } else {
        sqlx::query("SELECT id, name, data FROM assets WHERE assettype = 49 ORDER BY create_time DESC LIMIT 10")
            .fetch_all(&pool)
            .await?
    };

    if rows.is_empty() {
        println!("No mesh assets found.");
        return Ok(());
    }

    println!("\n=== Mesh Physics Dump ===\n");

    for row in &rows {
        let id: uuid::Uuid = row.get("id");
        let name: String = row.get("name");
        let data: Vec<u8> = row.get("data");

        println!("--- Asset: {} ---", id);
        println!("  Name: {}", name);
        println!("  Size: {} bytes", data.len());

        match parser::parse_mesh_header(&data) {
            Ok(header) => {
                println!("  Header parsed OK");
                if let Some(ref b) = header.physics_convex {
                    println!("  physics_convex: offset={} size={}", b.offset, b.size);
                } else {
                    println!("  physics_convex: MISSING");
                }
                if let Some(ref b) = header.physics_mesh {
                    println!("  physics_mesh:   offset={} size={}", b.offset, b.size);
                }
                if let Some(ref b) = header.high_lod {
                    println!("  high_lod:       offset={} size={}", b.offset, b.size);
                }
                if let Some(ref b) = header.medium_lod {
                    println!("  medium_lod:     offset={} size={}", b.offset, b.size);
                }
                if let Some(ref b) = header.low_lod {
                    println!("  low_lod:        offset={} size={}", b.offset, b.size);
                }
                if let Some(ref b) = header.lowest_lod {
                    println!("  lowest_lod:     offset={} size={}", b.offset, b.size);
                }
                if let Some(ref b) = header.skin {
                    println!("  skin:           offset={} size={}", b.offset, b.size);
                }

                if header.physics_convex.is_some() {
                    match parser::extract_physics_convex(&data, &header) {
                        Ok(convex) => {
                            println!("  --- Convex Physics Data ---");
                            println!("  Min: [{:.4}, {:.4}, {:.4}]", convex.min[0], convex.min[1], convex.min[2]);
                            println!("  Max: [{:.4}, {:.4}, {:.4}]", convex.max[0], convex.max[1], convex.max[2]);
                            println!("  Bounding hull: {} vertices", convex.bounding_hull.len());
                            println!("  Hull count: {}", convex.hull_count());
                            println!("  Total vertices: {}", convex.total_vertices());

                            for (i, hull) in convex.hulls.iter().enumerate() {
                                println!("    Hull {}: {} vertices", i, hull.len());
                                if hull.len() <= 8 {
                                    for v in hull {
                                        println!("      [{:.4}, {:.4}, {:.4}]", v[0], v[1], v[2]);
                                    }
                                } else {
                                    for v in &hull[..3] {
                                        println!("      [{:.4}, {:.4}, {:.4}]", v[0], v[1], v[2]);
                                    }
                                    println!("      ... ({} more)", hull.len() - 3);
                                }
                            }

                            let scale = [1.0f32, 1.0, 1.0];
                            let hull_array = parser::to_bullet_hull_array(&convex, scale);
                            println!("  BulletSim float array: {} floats", hull_array.len());
                            println!("  First 10: {:?}", &hull_array[..hull_array.len().min(10)]);
                        }
                        Err(e) => {
                            println!("  FAILED to extract physics_convex: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("  FAILED to parse header: {}", e);
                if data.len() >= 16 {
                    println!("  First 16 bytes: {:02x?}", &data[..16]);
                }
            }
        }
        println!();
    }

    println!("Total mesh assets examined: {}", rows.len());
    Ok(())
}
