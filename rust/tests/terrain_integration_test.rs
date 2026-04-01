use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use uuid::Uuid;

#[cfg(test)]
mod terrain_integration_tests {
    use super::*;
    use opensim_next::database::multi_backend::{DatabaseConnection, MultiDatabaseConfig};
    use opensim_next::region::terrain_storage::{TerrainStorage, TerrainRevision};
    use opensim_next::region::terrain_sender::TerrainSender;
    use opensim_next::protocol::terrain::{TerrainCompressor, TerrainPatch, LayerType};
    use opensim_next::udp::messages::LayerDataMessage;

    #[tokio::test]
    #[ignore]
    async fn test_terrain_storage_and_retrieval() -> Result<()> {
        let config = MultiDatabaseConfig::postgresql(
            "localhost",
            5432,
            "opensim_test",
            "opensim",
            "password123"
        );

        let db_connection = match DatabaseConnection::new(&config).await {
            Ok(conn) => Arc::new(conn),
            Err(_) => {
                eprintln!("Database not available for integration test (PostgreSQL)");
                return Ok(());
            }
        };

        let terrain_storage = TerrainStorage::new(db_connection);
        let region_uuid = Uuid::new_v4();
        let default_heightmap = TerrainStorage::create_default_heightmap();

        terrain_storage.store_terrain(
            region_uuid,
            &default_heightmap,
            TerrainRevision::Variable2D
        ).await?;

        let loaded_terrain = terrain_storage.load_terrain(region_uuid).await?;
        assert!(loaded_terrain.is_some());

        let terrain = loaded_terrain.unwrap();
        assert_eq!(terrain.len(), 256 * 256);
        assert!((terrain[0] - 21.0).abs() < 0.01);

        Ok(())
    }

    #[tokio::test]
    async fn test_terrain_compression_and_packets() -> Result<()> {
        let default_heightmap = TerrainStorage::create_default_heightmap();

        let patches = LayerDataMessage::split_into_packets(
            &default_heightmap,
            LayerType::Land
        )?;

        assert!(patches.len() > 0);
        assert!(patches.len() <= 256);

        for message in &patches {
            assert!(message.data.len() > 0);
            assert_eq!(message.layer_type, LayerType::Land);
        }

        Ok(())
    }

    #[test]
    fn test_terrain_patch_creation() -> Result<()> {
        let mut patch_data = vec![21.0f32; 16 * 16];
        patch_data[0] = 25.0;
        patch_data[255] = 30.0;

        let patch = TerrainPatch::with_data(0, 0, patch_data)?;
        assert_eq!(patch.x, 0);
        assert_eq!(patch.y, 0);
        assert_eq!(patch.data.len(), 256);
        assert!((patch.data[0] - 25.0).abs() < 0.01);
        assert!((patch.data[255] - 30.0).abs() < 0.01);

        Ok(())
    }

    #[test]
    fn test_terrain_compressor_initialization() {
        let _compressor = TerrainCompressor::new();
    }

    #[test]
    fn test_default_heightmap_properties() {
        let heightmap = TerrainStorage::create_default_heightmap();

        assert_eq!(heightmap.len(), 256 * 256);

        for height in &heightmap {
            assert!((*height - 21.0).abs() < 0.01);
        }
    }

    #[tokio::test]
    async fn test_layer_data_message_format() -> Result<()> {
        let default_heightmap = TerrainStorage::create_default_heightmap();
        let packets = LayerDataMessage::split_into_packets(&default_heightmap, LayerType::Land)?;

        assert!(packets.len() > 0);

        for (i, message) in packets.iter().enumerate() {
            let udp_packet = message.to_udp_packet(i as u32, true);

            assert!(udp_packet.len() >= 10);

            assert_eq!(udp_packet[0] & 0x40, 0x40);

            let sequence = u32::from_be_bytes([
                udp_packet[1],
                udp_packet[2],
                udp_packet[3],
                udp_packet[4],
            ]);
            assert_eq!(sequence, i as u32);
        }

        Ok(())
    }
}
