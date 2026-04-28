use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CoarseLocationUpdate {
    pub locations: Vec<(u8, u8, u8)>,
    pub you_index: i16,
    pub prey_index: i16,
    pub agent_ids: Vec<Uuid>,
}

impl CoarseLocationUpdate {
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
            you_index: -1,
            prey_index: -1,
            agent_ids: Vec::new(),
        }
    }

    pub fn add_avatar(&mut self, agent_id: Uuid, x: f32, y: f32, z: f32, is_you: bool) {
        let x_u8 = x.clamp(0.0, 255.0) as u8;
        let y_u8 = y.clamp(0.0, 255.0) as u8;
        let z_u8 = (z / 4.0).clamp(0.0, 255.0) as u8;

        self.locations.push((x_u8, y_u8, z_u8));
        self.agent_ids.push(agent_id);

        if is_you {
            self.you_index = (self.locations.len() - 1) as i16;
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let location_count = self.locations.len() as u8;
        data.push(location_count);

        for (x, y, z) in &self.locations {
            data.push(*x);
            data.push(*y);
            data.push(*z);
        }

        data.extend_from_slice(&self.you_index.to_le_bytes());
        data.extend_from_slice(&self.prey_index.to_le_bytes());

        let agent_count = self.agent_ids.len() as u8;
        data.push(agent_count);

        for agent_id in &self.agent_ids {
            data.extend_from_slice(agent_id.as_bytes());
        }

        data
    }

    pub fn parse(data: &Bytes) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        let location_count = cursor.get_u8();
        let mut locations = Vec::new();

        for _ in 0..location_count {
            let x = cursor.get_u8();
            let y = cursor.get_u8();
            let z = cursor.get_u8();
            locations.push((x, y, z));
        }

        let you_index = cursor.get_i16_le();
        let prey_index = cursor.get_i16_le();

        let agent_count = cursor.get_u8();
        let mut agent_ids = Vec::new();

        for _ in 0..agent_count {
            let mut agent_id_bytes = [0u8; 16];
            cursor.copy_to_slice(&mut agent_id_bytes);
            agent_ids.push(Uuid::from_bytes(agent_id_bytes));
        }

        Ok(Self {
            locations,
            you_index,
            prey_index,
            agent_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coarse_location_update_serialization() {
        let mut update = CoarseLocationUpdate::new();
        let agent1 = Uuid::new_v4();
        let agent2 = Uuid::new_v4();

        update.add_avatar(agent1, 128.0, 128.0, 25.0, true);
        update.add_avatar(agent2, 64.0, 192.0, 30.0, false);

        let serialized = update.serialize();
        let data = Bytes::from(serialized);
        let parsed = CoarseLocationUpdate::parse(&data).unwrap();

        assert_eq!(parsed.locations.len(), 2);
        assert_eq!(parsed.locations[0], (128, 128, 6));
        assert_eq!(parsed.locations[1], (64, 192, 7));
        assert_eq!(parsed.you_index, 0);
        assert_eq!(parsed.prey_index, -1);
        assert_eq!(parsed.agent_ids.len(), 2);
    }
}
