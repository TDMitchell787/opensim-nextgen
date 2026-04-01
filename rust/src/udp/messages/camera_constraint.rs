use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone)]
pub struct CameraConstraintMessage {
    pub plane: [f32; 4],
}

impl CameraConstraintMessage {
    pub fn new(plane: [f32; 4]) -> Self {
        Self { plane }
    }

    pub fn default_constraint() -> Self {
        Self {
            plane: [0.9, 0.0, 0.361, -10000.0],
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(16);

        for &component in &self.plane {
            buffer.put_f32_le(component);
        }

        buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_constraint_serialize() {
        let msg = CameraConstraintMessage::default_constraint();
        let data = msg.serialize();

        assert_eq!(data.len(), 16);
    }

    #[test]
    fn test_camera_constraint_custom() {
        let msg = CameraConstraintMessage::new([1.0, 0.0, 0.0, 0.0]);
        let data = msg.serialize();

        assert_eq!(data.len(), 16);
        let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(x, 1.0);
    }
}
