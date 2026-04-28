use anyhow::Result;
use bytes::Buf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentUpdateMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub body_rotation: [f32; 4],
    pub head_rotation: [f32; 4],
    pub state: u8,
    pub camera_center: [f32; 3],
    pub camera_at_axis: [f32; 3],
    pub camera_left_axis: [f32; 3],
    pub camera_up_axis: [f32; 3],
    pub far: f32,
    pub control_flags: u32,
    pub flags: u8,
}

impl AgentUpdateMessage {
    /// Parse packed quaternion (3 floats) and derive W component
    /// LLQuaternion format: X, Y, Z sent; W = sqrt(1 - x² - y² - z²)
    fn parse_packed_quaternion_le(cursor: &mut std::io::Cursor<&[u8]>) -> [f32; 4] {
        let x = cursor.get_f32_le();
        let y = cursor.get_f32_le();
        let z = cursor.get_f32_le();
        // Derive W: W = sqrt(1 - x² - y² - z²)
        let w_squared = 1.0 - x * x - y * y - z * z;
        let w = if w_squared > 0.0 {
            w_squared.sqrt()
        } else {
            0.0
        };
        [x, y, z, w]
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        // AgentUpdate packet size: 114 bytes
        // - AgentID: 16, SessionID: 16
        // - BodyRotation: 12 (packed quat), HeadRotation: 12 (packed quat)
        // - State: 1
        // - CameraCenter/At/Left/Up: 12 each = 48
        // - Far: 4, ControlFlags: 4, Flags: 1
        // Total: 16+16+12+12+1+48+4+4+1 = 114
        if data.len() < 114 {
            anyhow::bail!(
                "AgentUpdate packet too short: {} bytes (expected ≥114)",
                data.len()
            );
        }

        // UUIDs are big-endian in LLUDP
        let agent_id = Uuid::from_u128(cursor.get_u128());
        let session_id = Uuid::from_u128(cursor.get_u128());

        // LLQuaternion is packed (3 floats, little-endian), W derived
        let body_rotation = Self::parse_packed_quaternion_le(&mut cursor);
        let head_rotation = Self::parse_packed_quaternion_le(&mut cursor);

        let state = cursor.get_u8();

        // LLVector3 fields are little-endian
        let camera_center = [
            cursor.get_f32_le(),
            cursor.get_f32_le(),
            cursor.get_f32_le(),
        ];

        let camera_at_axis = [
            cursor.get_f32_le(),
            cursor.get_f32_le(),
            cursor.get_f32_le(),
        ];

        let camera_left_axis = [
            cursor.get_f32_le(),
            cursor.get_f32_le(),
            cursor.get_f32_le(),
        ];

        let camera_up_axis = [
            cursor.get_f32_le(),
            cursor.get_f32_le(),
            cursor.get_f32_le(),
        ];

        let far = cursor.get_f32_le();
        let control_flags = cursor.get_u32_le();
        let flags = cursor.get_u8();

        Ok(Self {
            agent_id,
            session_id,
            body_rotation,
            head_rotation,
            state,
            camera_center,
            camera_at_axis,
            camera_left_axis,
            camera_up_axis,
            far,
            control_flags,
            flags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_update_parse() {
        let mut data = Vec::new();

        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        // UUIDs are big-endian
        data.extend_from_slice(&agent_id.as_u128().to_be_bytes());
        data.extend_from_slice(&session_id.as_u128().to_be_bytes());

        // Packed quaternion (3 floats, little-endian) - identity rotation (0,0,0 -> W=1)
        data.extend_from_slice(&0.0f32.to_le_bytes()); // X
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Y
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Z

        // Head rotation - packed quaternion
        data.extend_from_slice(&0.0f32.to_le_bytes()); // X
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Y
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Z

        // State
        data.push(0x00);

        // Camera center (little-endian)
        data.extend_from_slice(&128.0f32.to_le_bytes());
        data.extend_from_slice(&128.0f32.to_le_bytes());
        data.extend_from_slice(&21.0f32.to_le_bytes());

        // Camera at axis
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Camera left axis
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Camera up axis
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Far (little-endian)
        data.extend_from_slice(&96.0f32.to_le_bytes());
        // ControlFlags (little-endian)
        data.extend_from_slice(&0x00000000u32.to_le_bytes());
        // Flags
        data.push(0x00);

        assert_eq!(data.len(), 114, "AgentUpdate packet should be 114 bytes");

        let message = AgentUpdateMessage::parse(&data).unwrap();

        assert_eq!(message.agent_id, agent_id);
        assert_eq!(message.session_id, session_id);
        assert_eq!(message.camera_center[0], 128.0);
        assert_eq!(message.camera_center[1], 128.0);
        assert_eq!(message.camera_center[2], 21.0);
        assert_eq!(message.far, 96.0);
        // Verify derived W component for identity quaternion
        assert!(
            (message.body_rotation[3] - 1.0).abs() < 0.0001,
            "W should be 1.0 for identity"
        );
    }
}
