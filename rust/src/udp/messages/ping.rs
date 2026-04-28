use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct StartPingCheckMessage {
    pub ping_id: u8,
    pub oldest_unacked: u32,
}

impl StartPingCheckMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        debug!(
            "StartPingCheck parse: {} bytes, data: {:?}",
            data.len(),
            data
        );

        if data.len() < 5 {
            return Err(anyhow!(
                "StartPingCheck message too short: {} bytes",
                data.len()
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        // For "Single" blocks, there is NO block count byte
        // The structure is just: [PingID][OldestUnacked]

        // Read Ping ID (1 byte)
        let ping_id = cursor.get_u8();

        // Read OldestUnacked (4 bytes, little-endian U32)
        let oldest_unacked = cursor.get_u32_le();

        debug!(
            "Parsed StartPingCheck: ping_id={}, oldest_unacked={}",
            ping_id, oldest_unacked
        );

        Ok(Self {
            ping_id,
            oldest_unacked,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![self.ping_id];
        buf.extend_from_slice(&self.oldest_unacked.to_le_bytes());
        buf
    }
}

#[derive(Debug, Clone)]
pub struct CompletePingCheckMessage {
    pub ping_id: u8,
}

impl CompletePingCheckMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        if data.len() < 1 {
            return Err(anyhow!(
                "CompletePingCheck message too short: {} bytes",
                data.len()
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        // For "Single" blocks, there is NO block count byte
        // The structure is just: [PingID]

        // Read Ping ID (1 byte)
        let ping_id = cursor.get_u8();

        debug!("Parsed CompletePingCheck: ping_id={}", ping_id);

        Ok(Self { ping_id })
    }

    pub fn serialize(&self) -> Vec<u8> {
        vec![self.ping_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_ping_check_parsing() {
        let ping_id = 42u8;
        let oldest_unacked = 12345u32;
        let original = StartPingCheckMessage {
            ping_id,
            oldest_unacked,
        };

        let serialized = original.serialize();
        let data = Bytes::from(serialized);
        let parsed = StartPingCheckMessage::parse(&data).unwrap();

        assert_eq!(parsed.ping_id, ping_id);
        assert_eq!(parsed.oldest_unacked, oldest_unacked);
    }

    #[test]
    fn test_complete_ping_check_parsing() {
        let ping_id = 99u8;
        let original = CompletePingCheckMessage { ping_id };

        let serialized = original.serialize();
        let data = Bytes::from(serialized);
        let parsed = CompletePingCheckMessage::parse(&data).unwrap();

        assert_eq!(parsed.ping_id, ping_id);
    }

    #[test]
    fn test_ping_check_short_data() {
        let data = Bytes::from(vec![1, 2, 3]); // Too short (needs 6 bytes minimum)
        let result = StartPingCheckMessage::parse(&data);
        assert!(result.is_err());
    }
}
