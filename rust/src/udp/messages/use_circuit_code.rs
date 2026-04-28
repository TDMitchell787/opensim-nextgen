use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UseCircuitCodeMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
}

impl UseCircuitCodeMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        debug!(
            "UseCircuitCode packet data ({} bytes): {:02x?}",
            data.len(),
            &data[..data.len().min(50)]
        );

        if data.len() < 36 {
            return Err(anyhow!(
                "UseCircuitCode message too short: {} bytes (expected 36)",
                data.len()
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        // CORRECT field order: Circuit Code, Session ID, Agent ID

        // Read Circuit Code (4 bytes, little endian)
        let circuit_code = cursor.get_u32_le();

        // Read Session ID (16 bytes UUID) - direct bytes, no swapping needed
        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        // Read Agent ID (16 bytes UUID) - direct bytes, no swapping needed
        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        debug!(
            "Parsed UseCircuitCode: agent={}, session={}, circuit={}",
            agent_id, session_id, circuit_code
        );

        Ok(Self {
            agent_id,
            session_id,
            circuit_code,
        })
    }

    fn parse_sl_uuid(bytes: &[u8; 16]) -> Uuid {
        // Second Life UUIDs: first 3 groups little-endian, last 2 groups big-endian
        let mut reordered = [0u8; 16];

        // Group 1 (4 bytes): reverse
        reordered[0] = bytes[3];
        reordered[1] = bytes[2];
        reordered[2] = bytes[1];
        reordered[3] = bytes[0];

        // Group 2 (2 bytes): reverse
        reordered[4] = bytes[5];
        reordered[5] = bytes[4];

        // Group 3 (2 bytes): reverse
        reordered[6] = bytes[7];
        reordered[7] = bytes[6];

        // Groups 4 and 5 (8 bytes): keep as-is (big-endian)
        reordered[8..16].copy_from_slice(&bytes[8..16]);

        Uuid::from_bytes(reordered)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(36);

        // CORRECT field order: Circuit Code, Session ID, Agent ID

        // Circuit Code (4 bytes, little endian)
        data.extend_from_slice(&self.circuit_code.to_le_bytes());

        // Session ID (16 bytes) - direct bytes, no swapping needed
        data.extend_from_slice(self.session_id.as_bytes());

        // Agent ID (16 bytes) - direct bytes, no swapping needed
        data.extend_from_slice(self.agent_id.as_bytes());

        data
    }

    fn serialize_sl_uuid(uuid: &Uuid) -> [u8; 16] {
        let bytes = uuid.as_bytes();
        let mut reordered = [0u8; 16];

        // Reverse first 3 groups (little-endian)
        reordered[0] = bytes[3];
        reordered[1] = bytes[2];
        reordered[2] = bytes[1];
        reordered[3] = bytes[0];

        reordered[4] = bytes[5];
        reordered[5] = bytes[4];

        reordered[6] = bytes[7];
        reordered[7] = bytes[6];

        // Keep last 2 groups as-is (big-endian)
        reordered[8..16].copy_from_slice(&bytes[8..16]);

        reordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_circuit_code_parsing() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let circuit_code = 12345u32;

        let original = UseCircuitCodeMessage {
            agent_id,
            session_id,
            circuit_code,
        };

        let serialized = original.serialize();
        let data = Bytes::from(serialized);
        let parsed = UseCircuitCodeMessage::parse(&data).unwrap();

        assert_eq!(parsed.agent_id, agent_id);
        assert_eq!(parsed.session_id, session_id);
        assert_eq!(parsed.circuit_code, circuit_code);
    }

    #[test]
    fn test_use_circuit_code_short_data() {
        let data = Bytes::from(vec![1, 2, 3]); // Too short
        let result = UseCircuitCodeMessage::parse(&data);
        assert!(result.is_err());
    }
}
