use bytes::{BufMut, BytesMut};
use tracing::debug;

pub struct PacketAck {
    pub packet_ids: Vec<u32>,
}

impl PacketAck {
    pub fn new(packet_ids: Vec<u32>) -> Self {
        Self { packet_ids }
    }

    pub fn build(&self, sequence_number: u32) -> BytesMut {
        let mut buf = BytesMut::new();

        let flags: u8 = 0x00;
        buf.put_u8(flags);

        buf.put_u32(sequence_number);

        buf.put_u8(0);

        buf.put_u8(0xFF);
        buf.put_u8(0xFF);
        buf.put_u8(0x00);
        buf.put_u8(0xFB);

        let packet_count = self.packet_ids.len() as u8;
        buf.put_u8(packet_count);

        for packet_id in &self.packet_ids {
            buf.put_u32_le(*packet_id);
        }

        debug!("Built PacketAck with {} acknowledgments", packet_count);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_ack_build() {
        let packet_ids = vec![1, 2, 3];
        let ack = PacketAck::new(packet_ids);
        let buf = ack.build(100);

        assert!(buf.len() > 0);
        assert_eq!(buf[0] & 0x80, 0);
    }

    #[test]
    fn test_packet_ack_empty() {
        let ack = PacketAck::new(vec![]);
        let buf = ack.build(1);

        assert!(buf.len() > 0);
    }
}
