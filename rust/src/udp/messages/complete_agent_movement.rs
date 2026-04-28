use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CompleteAgentMovementMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
}

impl CompleteAgentMovementMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        if data.len() < 36 {
            return Err(anyhow!(
                "CompleteAgentMovement message too short: {} bytes",
                data.len()
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        // Read Agent ID (16 bytes UUID)
        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        // Read Session ID (16 bytes UUID)
        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        // Read Circuit Code (4 bytes, little endian for CompleteAgentMovement)
        let circuit_code = cursor.get_u32_le();

        debug!(
            "Parsed CompleteAgentMovement: agent={}, session={}, circuit={}",
            agent_id, session_id, circuit_code
        );

        Ok(Self {
            agent_id,
            session_id,
            circuit_code,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(36);

        // Agent ID (16 bytes)
        data.extend_from_slice(self.agent_id.as_bytes());

        // Session ID (16 bytes)
        data.extend_from_slice(self.session_id.as_bytes());

        // Circuit Code (4 bytes, little endian)
        data.extend_from_slice(&self.circuit_code.to_le_bytes());

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_agent_movement_parsing() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let circuit_code = 12345u32;

        let original = CompleteAgentMovementMessage {
            agent_id,
            session_id,
            circuit_code,
        };

        let serialized = original.serialize();
        let data = Bytes::from(serialized);
        let parsed = CompleteAgentMovementMessage::parse(&data).unwrap();

        assert_eq!(parsed.agent_id, agent_id);
        assert_eq!(parsed.session_id, session_id);
        assert_eq!(parsed.circuit_code, circuit_code);
    }

    #[test]
    fn test_complete_agent_movement_short_data() {
        let data = Bytes::from(vec![1, 2, 3]); // Too short
        let result = CompleteAgentMovementMessage::parse(&data);
        assert!(result.is_err());
    }
}
