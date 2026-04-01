use anyhow::Result;
use bytes::{BytesMut, BufMut};
use crate::protocol::terrain::{TerrainCompressor, TerrainPatch, LayerType};

const LAYER_DATA_MESSAGE_ID: u32 = 0x0B;

#[derive(Debug, Clone)]
pub struct LayerDataMessage {
    pub layer_type: LayerType,
    pub data: Vec<u8>,
}

impl LayerDataMessage {
    pub fn from_terrain_patches(patches: &[TerrainPatch], layer_type: LayerType) -> Result<Self> {
        let compressor = TerrainCompressor::new();
        let data = compressor.create_layer_data_packet(patches, layer_type)?;

        Ok(Self { layer_type, data })
    }

    pub fn serialize_body(&self) -> Vec<u8> {
        // Phase 70.18: Serialize just the message body (after message ID)
        // This allows sending via send_message() which adds the LLUDP header
        // LayerID block: Type(1) = 1 byte
        // LayerData block: Length(2) + Data = 2 + data.len() bytes
        let body_size = 1 + 2 + self.data.len();
        let mut body = BytesMut::with_capacity(body_size);

        // LayerID block: Type (U8) - the layer type
        body.put_u8(self.layer_type as u8);

        // LayerData block: Variable 2 format = 2-byte length prefix (little-endian) + data
        body.put_u16_le(self.data.len() as u16);
        body.put_slice(&self.data);

        body.to_vec()
    }

    pub fn to_udp_packet(&self, sequence: u32, reliable: bool) -> Vec<u8> {
        // Phase 70.15: FIXED - extra_header byte IS required!
        // Header: flags(1) + sequence(4) + extra_header(1) + message_id(1) = 7 bytes for HIGH freq
        // LayerID block: Type(1) = 1 byte
        // LayerData block: Length(2) + Data = 2 + data.len() bytes
        let header_size = 7;  // includes extra_header byte
        let block_overhead = 1 + 2; // LayerID.Type + LayerData.Length
        let payload_size = self.data.len();
        let total_size = header_size + block_overhead + payload_size;

        let mut packet = BytesMut::with_capacity(total_size);

        let flags = if reliable { 0x40 } else { 0x00 };
        packet.put_u8(flags);

        // Sequence number in BIG-ENDIAN (network byte order) per LLUDP protocol
        packet.put_u32(sequence);

        // Phase 70.15: Extra header byte IS REQUIRED for LLUDP compatibility!
        // Without it, the viewer interprets message_id (0x0B) as extra_header_length
        // and tries to skip 11 bytes of "extra header", corrupting the entire message!
        packet.put_u8(0x00);  // Extra header length = 0

        // LayerData (0x0B) is HIGH frequency - only 1 byte for message ID!
        packet.put_u8(LAYER_DATA_MESSAGE_ID as u8);

        // LayerID block: Type (U8) - the layer type
        packet.put_u8(self.layer_type as u8);

        // LayerData block: Variable 2 format = 2-byte length prefix (little-endian) + data
        packet.put_u16_le(self.data.len() as u16);
        packet.put_slice(&self.data);

        packet.to_vec()
    }

    pub fn split_into_packets(
        heightmap: &[f32],
        layer_type: LayerType,
    ) -> Result<Vec<LayerDataMessage>> {
        Self::split_into_packets_ex(heightmap, layer_type, 256, 256)
    }

    pub fn split_into_packets_ex(
        heightmap: &[f32],
        layer_type: LayerType,
        region_size_x: u32,
        region_size_y: u32,
    ) -> Result<Vec<LayerDataMessage>> {
        let expected = (region_size_x * region_size_y) as usize;
        if heightmap.len() != expected {
            anyhow::bail!(
                "Heightmap must be {}x{} ({} floats), got {}",
                region_size_x, region_size_y, expected, heightmap.len()
            );
        }

        let patches_x = (region_size_x / 16) as usize;
        let patches_y = (region_size_y / 16) as usize;
        let large_region = region_size_x > 256 || region_size_y > 256;

        const PATCHES_PER_PACKET: usize = 4;

        let mut packets = Vec::new();
        let mut current_patches = Vec::new();

        for patch_y in 0..patches_y {
            for patch_x in 0..patches_x {
                let mut patch_data = vec![0.0f32; 16 * 16];

                for y in 0..16 {
                    for x in 0..16 {
                        let world_x = patch_x * 16 + x;
                        let world_y = patch_y * 16 + y;
                        let index = world_y * (region_size_x as usize) + world_x;
                        patch_data[y * 16 + x] = heightmap[index];
                    }
                }

                let patch = TerrainPatch::with_data(patch_x as i32, patch_y as i32, patch_data)?;
                current_patches.push(patch);

                if current_patches.len() >= PATCHES_PER_PACKET {
                    let compressor = TerrainCompressor::new();
                    let data = compressor.create_layer_data_packet_ex(&current_patches, layer_type, large_region)?;
                    packets.push(LayerDataMessage { layer_type, data });
                    current_patches.clear();
                }
            }
        }

        if !current_patches.is_empty() {
            let compressor = TerrainCompressor::new();
            let data = compressor.create_layer_data_packet_ex(&current_patches, layer_type, large_region)?;
            packets.push(LayerDataMessage { layer_type, data });
        }

        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_data_packet_creation() {
        let patch = TerrainPatch::new(0, 0);
        let message = LayerDataMessage::from_terrain_patches(&[patch], LayerType::Land).unwrap();

        assert_eq!(message.layer_type, LayerType::Land);
        assert!(!message.data.is_empty());
    }

    #[test]
    fn test_split_into_packets() {
        let heightmap = vec![21.0f32; 256 * 256];
        let packets = LayerDataMessage::split_into_packets(&heightmap, LayerType::Land).unwrap();

        assert!(!packets.is_empty());
        assert!(packets.len() <= 64);
    }

    #[test]
    fn test_udp_packet_format() {
        let patch = TerrainPatch::new(0, 0);
        let message = LayerDataMessage::from_terrain_patches(&[patch], LayerType::Land).unwrap();
        let udp_packet = message.to_udp_packet(1, true);

        // Phase 70.7: Header is now 6 bytes (flags+seq+msgid), not 7
        // Header(6) + LayerID.Type(1) + LayerData.Length(2) = 9 bytes minimum
        assert!(udp_packet.len() >= 9);
        assert_eq!(udp_packet[0], 0x40); // Reliable flag
        // Sequence is big-endian: [0, 0, 0, 1]
        assert_eq!(udp_packet[1], 0x00);
        assert_eq!(udp_packet[4], 0x01);
        // Phase 70.7: No extra header byte - message ID immediately after sequence
        assert_eq!(udp_packet[5], 0x0B); // LayerData HIGH frequency message ID
        assert_eq!(udp_packet[6], 0x4C); // LayerID.Type = Land (0x4C)
        // Bytes 7-8 are the 2-byte length prefix (little-endian)
    }
}
