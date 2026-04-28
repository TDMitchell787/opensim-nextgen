use crate::database::multi_backend::DatabaseConnection;
use crate::protocol::terrain::TerrainCompressor;
use anyhow::{Context, Result};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum TerrainRevision {
    Legacy256 = 11,
    Variable2D = 22,
    Variable2DGzip = 23,
    Compressed2D = 27,
}

pub struct TerrainStorage {
    db_connection: Arc<DatabaseConnection>,
    compressor: TerrainCompressor,
}

impl TerrainStorage {
    pub fn new(db_connection: Arc<DatabaseConnection>) -> Self {
        Self {
            db_connection,
            compressor: TerrainCompressor::new(),
        }
    }

    pub async fn store_terrain(
        &self,
        region_uuid: Uuid,
        heightmap: &[f32],
        revision: TerrainRevision,
    ) -> Result<()> {
        let len = heightmap.len();
        let side = (len as f64).sqrt() as usize;
        if side * side != len || side < 256 || (side & (side - 1)) != 0 {
            anyhow::bail!(
                "Heightmap must be square power-of-2 (min 256x256), got {} elements",
                len
            );
        }

        let heightfield = self.serialize_heightmap(heightmap, revision)?;

        let region_uuid_str = region_uuid.to_string();
        let revision_i32 = revision as i32;

        match &*self.db_connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(
                    "INSERT INTO bakedterrain (regionuuid, revision, heightfield)
                     VALUES ($1, $2, $3)
                     ON CONFLICT (regionuuid, revision)
                     DO UPDATE SET heightfield = EXCLUDED.heightfield",
                )
                .bind(&region_uuid_str)
                .bind(revision_i32)
                .bind(&heightfield)
                .execute(pool)
                .await
                .context("Failed to store terrain in PostgreSQL")?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(
                    "INSERT INTO bakedterrain (RegionUUID, Revision, Heightfield)
                     VALUES (?, ?, ?)
                     ON DUPLICATE KEY UPDATE Heightfield = VALUES(Heightfield)",
                )
                .bind(&region_uuid_str)
                .bind(revision_i32)
                .bind(&heightfield)
                .execute(pool)
                .await
                .context("Failed to store terrain in MySQL")?;
            }
        }

        Ok(())
    }

    pub async fn load_terrain(&self, region_uuid: Uuid) -> Result<Option<Vec<f32>>> {
        let region_uuid_str = region_uuid.to_string();

        match &*self.db_connection {
            DatabaseConnection::PostgreSQL(pool) => {
                let result = sqlx::query(
                    "SELECT revision, heightfield FROM bakedterrain
                     WHERE regionuuid = $1
                     ORDER BY revision DESC
                     LIMIT 1",
                )
                .bind(&region_uuid_str)
                .fetch_optional(pool)
                .await
                .context("Failed to load terrain from PostgreSQL")?;

                if let Some(row) = result {
                    let revision: i32 = row.try_get(0)?;
                    let heightfield: Vec<u8> = row.try_get(1)?;

                    let terrain_revision = match revision {
                        11 => TerrainRevision::Legacy256,
                        22 => TerrainRevision::Variable2D,
                        23 => TerrainRevision::Variable2DGzip,
                        27 => TerrainRevision::Compressed2D,
                        _ => TerrainRevision::Legacy256,
                    };

                    let heightmap = self.deserialize_heightmap(&heightfield, terrain_revision)?;
                    Ok(Some(heightmap))
                } else {
                    Ok(None)
                }
            }
            DatabaseConnection::MySQL(pool) => {
                let result = sqlx::query(
                    "SELECT Revision, Heightfield FROM bakedterrain
                     WHERE RegionUUID = ?
                     ORDER BY Revision DESC
                     LIMIT 1",
                )
                .bind(&region_uuid_str)
                .fetch_optional(pool)
                .await
                .context("Failed to load terrain from MySQL")?;

                if let Some(row) = result {
                    let revision: i32 = row.try_get(0)?;
                    let heightfield: Vec<u8> = row.try_get(1)?;

                    let terrain_revision = match revision {
                        11 => TerrainRevision::Legacy256,
                        22 => TerrainRevision::Variable2D,
                        23 => TerrainRevision::Variable2DGzip,
                        27 => TerrainRevision::Compressed2D,
                        _ => TerrainRevision::Legacy256,
                    };

                    let heightmap = self.deserialize_heightmap(&heightfield, terrain_revision)?;
                    Ok(Some(heightmap))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn delete_terrain(&self, region_uuid: Uuid) -> Result<()> {
        let region_uuid_str = region_uuid.to_string();

        match &*self.db_connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query("DELETE FROM bakedterrain WHERE regionuuid = $1")
                    .bind(&region_uuid_str)
                    .execute(pool)
                    .await
                    .context("Failed to delete terrain from PostgreSQL")?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query("DELETE FROM bakedterrain WHERE RegionUUID = ?")
                    .bind(&region_uuid_str)
                    .execute(pool)
                    .await
                    .context("Failed to delete terrain from MySQL")?;
            }
        }

        Ok(())
    }

    fn serialize_heightmap(&self, heightmap: &[f32], revision: TerrainRevision) -> Result<Vec<u8>> {
        let side = (heightmap.len() as f64).sqrt() as i32;
        match revision {
            TerrainRevision::Variable2D => {
                let mut buffer = Vec::with_capacity(8 + heightmap.len() * 4);

                buffer.extend_from_slice(&side.to_le_bytes());
                buffer.extend_from_slice(&side.to_le_bytes());

                for &height in heightmap {
                    buffer.extend_from_slice(&height.to_le_bytes());
                }

                Ok(buffer)
            }
            TerrainRevision::Variable2DGzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                use std::io::Write;

                let mut uncompressed = Vec::with_capacity(8 + heightmap.len() * 4);

                uncompressed.extend_from_slice(&side.to_le_bytes());
                uncompressed.extend_from_slice(&side.to_le_bytes());

                for &height in heightmap {
                    uncompressed.extend_from_slice(&height.to_le_bytes());
                }

                let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(&uncompressed)?;
                let compressed = encoder.finish()?;

                Ok(compressed)
            }
            TerrainRevision::Compressed2D => {
                let mut buffer = Vec::with_capacity(12 + heightmap.len() * 2);

                let compression_factor = 100i32;

                buffer.extend_from_slice(&side.to_le_bytes());
                buffer.extend_from_slice(&side.to_le_bytes());
                buffer.extend_from_slice(&compression_factor.to_le_bytes());

                for &height in heightmap {
                    let compressed_height = (height * compression_factor as f32) as u16;
                    buffer.extend_from_slice(&compressed_height.to_le_bytes());
                }

                Ok(buffer)
            }
            TerrainRevision::Legacy256 => {
                let mut buffer = Vec::with_capacity(heightmap.len() * 8);

                for &height in heightmap {
                    buffer.extend_from_slice(&(height as f64).to_le_bytes());
                }

                Ok(buffer)
            }
        }
    }

    fn deserialize_heightmap(&self, data: &[u8], revision: TerrainRevision) -> Result<Vec<f32>> {
        match revision {
            TerrainRevision::Variable2D => {
                if data.len() < 8 {
                    anyhow::bail!("Invalid Variable2D terrain data: too short");
                }

                let size_x = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let size_y = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);

                if size_x < 256 || size_y < 256 {
                    anyhow::bail!(
                        "Unsupported terrain size: {}x{}, minimum 256x256",
                        size_x,
                        size_y
                    );
                }

                let total = (size_x as usize) * (size_y as usize);
                let expected_len = 8 + total * 4;
                if data.len() != expected_len {
                    anyhow::bail!(
                        "Invalid Variable2D terrain data length: got {} expected {}",
                        data.len(),
                        expected_len
                    );
                }

                let mut heightmap = Vec::with_capacity(total);
                let mut offset = 8;

                for _ in 0..total {
                    let bytes = [
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ];
                    heightmap.push(f32::from_le_bytes(bytes));
                    offset += 4;
                }

                Ok(heightmap)
            }
            TerrainRevision::Variable2DGzip => {
                use flate2::read::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(data);
                let mut uncompressed = Vec::new();
                decoder.read_to_end(&mut uncompressed)?;

                self.deserialize_heightmap(&uncompressed, TerrainRevision::Variable2D)
            }
            TerrainRevision::Compressed2D => {
                if data.len() < 12 {
                    anyhow::bail!("Invalid Compressed2D terrain data: too short");
                }

                let size_x = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let size_y = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let compression_factor = i32::from_le_bytes([data[8], data[9], data[10], data[11]]);

                if size_x < 256 || size_y < 256 {
                    anyhow::bail!(
                        "Unsupported terrain size: {}x{}, minimum 256x256",
                        size_x,
                        size_y
                    );
                }

                let total = (size_x as usize) * (size_y as usize);
                let expected_len = 12 + total * 2;
                if data.len() != expected_len {
                    anyhow::bail!(
                        "Invalid Compressed2D terrain data length: got {} expected {}",
                        data.len(),
                        expected_len
                    );
                }

                let mut heightmap = Vec::with_capacity(total);
                let mut offset = 12;
                let divisor = compression_factor as f32;

                for _ in 0..total {
                    let bytes = [data[offset], data[offset + 1]];
                    let compressed_height = u16::from_le_bytes(bytes);
                    heightmap.push(compressed_height as f32 / divisor);
                    offset += 2;
                }

                Ok(heightmap)
            }
            TerrainRevision::Legacy256 => {
                let expected_f32 = 256 * 256 * 4;
                let expected_f64 = 256 * 256 * 8;
                if data.len() == expected_f32 {
                    let mut heightmap = Vec::with_capacity(65536);
                    let mut offset = 0;
                    for _ in 0..65536 {
                        let bytes = [
                            data[offset],
                            data[offset + 1],
                            data[offset + 2],
                            data[offset + 3],
                        ];
                        heightmap.push(f32::from_le_bytes(bytes));
                        offset += 4;
                    }
                    return Ok(heightmap);
                } else if data.len() == expected_f64 {
                    let mut heightmap = Vec::with_capacity(65536);
                    let mut offset = 0;
                    for _ in 0..65536 {
                        let bytes = [
                            data[offset],
                            data[offset + 1],
                            data[offset + 2],
                            data[offset + 3],
                            data[offset + 4],
                            data[offset + 5],
                            data[offset + 6],
                            data[offset + 7],
                        ];
                        heightmap.push(f64::from_le_bytes(bytes) as f32);
                        offset += 8;
                    }
                    return Ok(heightmap);
                } else {
                    anyhow::bail!(
                        "Invalid Legacy256 terrain data length: got {}, expected {} (f32) or {} (f64)",
                        data.len(),
                        expected_f32,
                        expected_f64
                    );
                }
            }
        }
    }

    pub fn create_default_heightmap() -> Vec<f32> {
        vec![1.0; 256 * 256]
    }

    pub fn create_default_heightmap_sized(size_x: u32, size_y: u32) -> Vec<f32> {
        vec![1.0; (size_x as usize) * (size_y as usize)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_heightmap() {
        let heightmap = TerrainStorage::create_default_heightmap();
        assert_eq!(heightmap.len(), 256 * 256);
        assert_eq!(heightmap[0], 1.0);
        assert_eq!(heightmap[65535], 1.0);
    }
}
