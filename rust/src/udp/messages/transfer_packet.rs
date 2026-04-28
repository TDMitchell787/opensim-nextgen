//! TransferPacket message builder
//!
//! Server sends this to deliver actual asset data to the viewer.
//! Message structure from message_template.msg:
//! TransferPacket High 17 NotTrusted Unencoded
//!   TransferData Single
//!     TransferID  LLUUID
//!     ChannelType S32
//!     Packet      S32
//!     Status      S32
//!     Data        Variable 2

use bytes::{BufMut, BytesMut};
use uuid::Uuid;

/// Transfer packet status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PacketStatus {
    Ok = 0,    // More packets coming
    Done = 1,  // Final packet
    Skip = 2,  // Skip this packet
    Abort = 3, // Transfer aborted
    Error = 4, // Error occurred
}

#[derive(Debug, Clone)]
pub struct TransferPacket {
    pub transfer_id: Uuid,
    pub channel_type: i32,
    pub packet_num: i32,
    pub status: PacketStatus,
    pub data: Vec<u8>,
}

impl TransferPacket {
    pub fn new(
        transfer_id: Uuid,
        channel_type: i32,
        packet_num: i32,
        status: PacketStatus,
        data: Vec<u8>,
    ) -> Self {
        Self {
            transfer_id,
            channel_type,
            packet_num,
            status,
            data,
        }
    }

    /// Create a single-packet transfer (status = Done)
    pub fn new_single(transfer_id: Uuid, channel_type: i32, data: Vec<u8>) -> Self {
        Self::new(transfer_id, channel_type, 0, PacketStatus::Done, data)
    }

    /// Create a multi-packet transfer chunk
    pub fn new_chunk(
        transfer_id: Uuid,
        channel_type: i32,
        packet_num: i32,
        data: Vec<u8>,
        is_final: bool,
    ) -> Self {
        let status = if is_final {
            PacketStatus::Done
        } else {
            PacketStatus::Ok
        };

        Self::new(transfer_id, channel_type, packet_num, status, data)
    }

    pub fn serialize(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(38 + self.data.len());

        // TransferID (16 bytes)
        buf.put_slice(self.transfer_id.as_bytes());

        // ChannelType (4 bytes)
        buf.put_i32_le(self.channel_type);

        // Packet number (4 bytes)
        buf.put_i32_le(self.packet_num);

        // Status (4 bytes)
        buf.put_i32_le(self.status as i32);

        // Data length (2 bytes for Variable 2)
        buf.put_u16_le(self.data.len() as u16);

        // Data
        buf.put_slice(&self.data);

        buf
    }
}

/// Helper to split large assets into multiple TransferPackets
pub struct TransferPacketBuilder {
    transfer_id: Uuid,
    channel_type: i32,
    chunk_size: usize,
}

impl TransferPacketBuilder {
    pub fn new(transfer_id: Uuid, channel_type: i32) -> Self {
        Self {
            transfer_id,
            channel_type,
            chunk_size: 1000, // Default chunk size (safe for UDP MTU)
        }
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Split asset data into multiple TransferPackets
    pub fn build_packets(&self, asset_data: &[u8]) -> Vec<TransferPacket> {
        if asset_data.is_empty() {
            return vec![TransferPacket::new_single(
                self.transfer_id,
                self.channel_type,
                Vec::new(),
            )];
        }

        let total_packets = (asset_data.len() + self.chunk_size - 1) / self.chunk_size;
        let mut packets = Vec::with_capacity(total_packets);

        for (packet_num, chunk) in asset_data.chunks(self.chunk_size).enumerate() {
            let is_final = packet_num == total_packets - 1;
            let packet = TransferPacket::new_chunk(
                self.transfer_id,
                self.channel_type,
                packet_num as i32,
                chunk.to_vec(),
                is_final,
            );
            packets.push(packet);
        }

        packets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_packet() {
        let transfer_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let data = vec![0xaa, 0xbb, 0xcc, 0xdd];

        let packet = TransferPacket::new_single(transfer_id, 2, data.clone());

        assert_eq!(packet.packet_num, 0);
        assert_eq!(packet.status, PacketStatus::Done);
        assert_eq!(packet.data, data);

        let serialized = packet.serialize();
        assert!(serialized.len() > 38);
    }

    #[test]
    fn test_multi_packet_builder() {
        let transfer_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let data = vec![0u8; 2500]; // 2.5KB of data

        let builder = TransferPacketBuilder::new(transfer_id, 2).with_chunk_size(1000);

        let packets = builder.build_packets(&data);

        assert_eq!(packets.len(), 3); // Should split into 3 packets

        // First two packets should have status Ok
        assert_eq!(packets[0].status, PacketStatus::Ok);
        assert_eq!(packets[0].packet_num, 0);
        assert_eq!(packets[0].data.len(), 1000);

        assert_eq!(packets[1].status, PacketStatus::Ok);
        assert_eq!(packets[1].packet_num, 1);
        assert_eq!(packets[1].data.len(), 1000);

        // Last packet should have status Done
        assert_eq!(packets[2].status, PacketStatus::Done);
        assert_eq!(packets[2].packet_num, 2);
        assert_eq!(packets[2].data.len(), 500);
    }

    #[test]
    fn test_empty_data() {
        let transfer_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let builder = TransferPacketBuilder::new(transfer_id, 2);

        let packets = builder.build_packets(&[]);

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].status, PacketStatus::Done);
        assert_eq!(packets[0].data.len(), 0);
    }
}
