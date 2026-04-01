use bytes::{Bytes, Buf};
use anyhow::{Result, anyhow};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentAnimationMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub animation_list: Vec<AnimationEntry>,
}

#[derive(Debug, Clone)]
pub struct AnimationEntry {
    pub anim_id: Uuid,
    pub start_anim: bool,
}

impl AgentAnimationMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        if data.len() < 33 {
            return Err(anyhow!("AgentAnimation message too short: {} bytes", data.len()));
        }

        let mut cursor = std::io::Cursor::new(data);

        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        let anim_count = cursor.get_u8() as usize;

        let mut animation_list = Vec::with_capacity(anim_count);
        for _ in 0..anim_count {
            if cursor.remaining() < 17 {
                break;
            }
            let mut anim_id_bytes = [0u8; 16];
            cursor.copy_to_slice(&mut anim_id_bytes);
            let anim_id = Uuid::from_bytes(anim_id_bytes);
            let start_anim = cursor.get_u8() != 0;
            animation_list.push(AnimationEntry { anim_id, start_anim });
        }

        debug!(
            "Parsed AgentAnimation: agent={}, animations={}",
            agent_id, animation_list.len()
        );

        Ok(Self {
            agent_id,
            session_id,
            animation_list,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TestMessageMessage {
    pub test_block1: u32,
}

impl TestMessageMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        debug!("TestMessage parse: {} bytes", data.len());

        if data.is_empty() {
            return Ok(Self { test_block1: 0 });
        }

        let mut cursor = std::io::Cursor::new(data);
        let test_block1 = if cursor.remaining() >= 4 {
            cursor.get_u32_le()
        } else {
            0
        };

        Ok(Self { test_block1 })
    }
}

#[derive(Debug, Clone)]
pub struct AgentHeightWidthMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
    pub height: u16,
    pub width: u16,
}

impl AgentHeightWidthMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        if data.len() < 40 {
            return Err(anyhow!("AgentHeightWidth message too short: {} bytes", data.len()));
        }

        let mut cursor = std::io::Cursor::new(data);

        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        let circuit_code = cursor.get_u32_le();
        let height = cursor.get_u16_le();
        let width = cursor.get_u16_le();

        debug!(
            "Parsed AgentHeightWidth: agent={}, height={}, width={}",
            agent_id, height, width
        );

        Ok(Self {
            agent_id,
            session_id,
            circuit_code,
            height,
            width,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetAlwaysRunMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub always_run: bool,
}

impl SetAlwaysRunMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        if data.len() < 33 {
            return Err(anyhow!("SetAlwaysRun message too short: {} bytes", data.len()));
        }

        let mut cursor = std::io::Cursor::new(data);

        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        let always_run = cursor.get_u8() != 0;

        debug!(
            "Parsed SetAlwaysRun: agent={}, always_run={}",
            agent_id, always_run
        );

        Ok(Self {
            agent_id,
            session_id,
            always_run,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UUIDNameRequestMessage {
    pub ids: Vec<Uuid>,
}

impl UUIDNameRequestMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        let count = if cursor.remaining() > 0 {
            cursor.get_u8() as usize
        } else {
            return Ok(Self { ids: vec![] });
        };

        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            if cursor.remaining() < 16 {
                break;
            }
            let mut id_bytes = [0u8; 16];
            cursor.copy_to_slice(&mut id_bytes);
            ids.push(Uuid::from_bytes(id_bytes));
        }

        debug!("Parsed UUIDNameRequest: {} IDs requested", ids.len());

        Ok(Self { ids })
    }
}

#[derive(Debug, Clone)]
pub struct UUIDNameReplyMessage {
    pub entries: Vec<UUIDNameEntry>,
}

#[derive(Debug, Clone)]
pub struct UUIDNameEntry {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
}

impl UUIDNameReplyMessage {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add_entry(&mut self, id: Uuid, first_name: String, last_name: String) {
        self.entries.push(UUIDNameEntry { id, first_name, last_name });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.entries.len() * 50);

        data.push(self.entries.len() as u8);

        for entry in &self.entries {
            data.extend_from_slice(entry.id.as_bytes());

            let first_bytes = entry.first_name.as_bytes();
            data.push((first_bytes.len() + 1) as u8);
            data.extend_from_slice(first_bytes);
            data.push(0);

            let last_bytes = entry.last_name.as_bytes();
            data.push((last_bytes.len() + 1) as u8);
            data.extend_from_slice(last_bytes);
            data.push(0);
        }

        data
    }
}

#[derive(Debug, Clone)]
pub struct UUIDGroupNameRequestMessage {
    pub ids: Vec<Uuid>,
}

impl UUIDGroupNameRequestMessage {
    pub fn parse(data: &Bytes) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        let count = if cursor.remaining() > 0 {
            cursor.get_u8() as usize
        } else {
            return Ok(Self { ids: vec![] });
        };

        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            if cursor.remaining() < 16 {
                break;
            }
            let mut id_bytes = [0u8; 16];
            cursor.copy_to_slice(&mut id_bytes);
            ids.push(Uuid::from_bytes(id_bytes));
        }

        debug!("Parsed UUIDGroupNameRequest: {} IDs requested", ids.len());

        Ok(Self { ids })
    }
}

#[derive(Debug, Clone)]
pub struct UUIDGroupNameReplyMessage {
    pub entries: Vec<UUIDGroupNameEntry>,
}

#[derive(Debug, Clone)]
pub struct UUIDGroupNameEntry {
    pub id: Uuid,
    pub group_name: String,
}

impl UUIDGroupNameReplyMessage {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add_entry(&mut self, id: Uuid, group_name: String) {
        self.entries.push(UUIDGroupNameEntry { id, group_name });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.entries.len() * 40);

        data.push(self.entries.len() as u8);

        for entry in &self.entries {
            data.extend_from_slice(entry.id.as_bytes());

            let name_bytes = entry.group_name.as_bytes();
            data.push(name_bytes.len() as u8);
            data.extend_from_slice(name_bytes);
        }

        data
    }
}

#[derive(Debug, Clone)]
pub struct WearableEntry {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub wearable_type: u8,
}

#[derive(Debug, Clone)]
pub struct AgentWearablesUpdateMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub serial_num: u32,
    pub wearables: Vec<WearableEntry>,
}

impl AgentWearablesUpdateMessage {
    pub fn new(agent_id: Uuid, session_id: Uuid) -> Self {
        Self {
            agent_id,
            session_id,
            serial_num: 1,
            wearables: Vec::new(),
        }
    }

    pub fn with_default_ruth(agent_id: Uuid, session_id: Uuid) -> Self {
        let mut msg = Self::new(agent_id, session_id);

        // Canonical OpenSim default Ruth wearables from AvatarWearable.cs
        // Item IDs and Asset IDs must match what OpenSim uses exactly
        // CRITICAL: Viewer requires minimum 6 wearables to bake textures

        // Shape (type 0) - DEFAULT_BODY_ITEM / DEFAULT_BODY_ASSET
        msg.add_wearable(
            Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfaba9").unwrap(),
            Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfab73").unwrap(),
            0,
        );
        // Skin (type 1) - DEFAULT_SKIN_ITEM / DEFAULT_SKIN_ASSET
        msg.add_wearable(
            Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bfabc9").unwrap(),
            Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bbabbb").unwrap(),
            1,
        );
        // Hair (type 2) - DEFAULT_HAIR_ITEM / DEFAULT_HAIR_ASSET
        msg.add_wearable(
            Uuid::parse_str("d342e6c1-b9d2-11dc-95ff-0800200c9a66").unwrap(),
            Uuid::parse_str("d342e6c0-b9d2-11dc-95ff-0800200c9a66").unwrap(),
            2,
        );
        // Eyes (type 3) - DEFAULT_EYES_ITEM / DEFAULT_EYES_ASSET
        msg.add_wearable(
            Uuid::parse_str("cdc31054-eed8-4021-994f-4e0c6e861b50").unwrap(),
            Uuid::parse_str("4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7").unwrap(),
            3,
        );
        // Shirt (type 4) - DEFAULT_SHIRT_ITEM / DEFAULT_SHIRT_ASSET
        msg.add_wearable(
            Uuid::parse_str("77c41e39-38f9-f75a-0000-585989bf0000").unwrap(),
            Uuid::parse_str("00000000-38f9-1111-024e-222222111110").unwrap(),
            4,
        );
        // Pants (type 5) - DEFAULT_PANTS_ITEM / DEFAULT_PANTS_ASSET
        msg.add_wearable(
            Uuid::parse_str("77c41e39-38f9-f75a-0000-5859892f1111").unwrap(),
            Uuid::parse_str("00000000-38f9-1111-024e-222222111120").unwrap(),
            5,
        );

        msg
    }

    pub fn add_wearable(&mut self, item_id: Uuid, asset_id: Uuid, wearable_type: u8) {
        self.wearables.push(WearableEntry {
            item_id,
            asset_id,
            wearable_type,
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        // AgentData block (Single - no count byte)
        // WearableData block (Variable - has count byte)
        let mut data = Vec::with_capacity(36 + 1 + self.wearables.len() * 33);

        // AgentData block (Single)
        data.extend_from_slice(self.agent_id.as_bytes());     // AgentID: 16 bytes
        data.extend_from_slice(self.session_id.as_bytes());   // SessionID: 16 bytes
        data.extend_from_slice(&self.serial_num.to_le_bytes()); // SerialNum: 4 bytes

        // WearableData block count (Variable block)
        data.push(self.wearables.len() as u8);

        // WearableData entries
        for wearable in &self.wearables {
            data.extend_from_slice(wearable.item_id.as_bytes());   // ItemID: 16 bytes
            data.extend_from_slice(wearable.asset_id.as_bytes());  // AssetID: 16 bytes
            data.push(wearable.wearable_type);                      // WearableType: 1 byte
        }

        data
    }
}

#[derive(Debug, Clone)]
pub struct SimulatorViewerTimeMessage {
    pub usec_since_start: u64,
    pub sec_per_day: u32,
    pub sec_per_year: u32,
    pub sun_direction: [f32; 3],
    pub sun_phase: f32,
    pub sun_ang_velocity: [f32; 3],
}

impl SimulatorViewerTimeMessage {
    pub fn new_default() -> Self {
        let sun_angle: f32 = 0.5236;
        Self {
            usec_since_start: 0,
            sec_per_day: 14400,
            sec_per_year: 5259600,
            sun_direction: [sun_angle.cos(), 0.0, sun_angle.sin()],
            sun_phase: 5.585,
            sun_ang_velocity: [0.0, 0.0, 0.0],
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(44);

        data.extend_from_slice(&self.usec_since_start.to_le_bytes());
        data.extend_from_slice(&self.sec_per_day.to_le_bytes());
        data.extend_from_slice(&self.sec_per_year.to_le_bytes());

        for val in &self.sun_direction {
            data.extend_from_slice(&val.to_le_bytes());
        }

        data.extend_from_slice(&self.sun_phase.to_le_bytes());

        for val in &self.sun_ang_velocity {
            data.extend_from_slice(&val.to_le_bytes());
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_name_reply_serialization() {
        let mut reply = UUIDNameReplyMessage::new();
        let test_id = Uuid::new_v4();
        reply.add_entry(test_id, "Test".to_string(), "User".to_string());

        let serialized = reply.serialize();

        assert_eq!(serialized[0], 1);
        assert_eq!(&serialized[1..17], test_id.as_bytes());
        assert_eq!(serialized[17], 4);
        assert_eq!(&serialized[18..22], b"Test");
        assert_eq!(serialized[22], 4);
        assert_eq!(&serialized[23..27], b"User");
    }

    #[test]
    fn test_set_always_run_parsing() {
        let mut data = Vec::new();
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        data.extend_from_slice(agent_id.as_bytes());
        data.extend_from_slice(session_id.as_bytes());
        data.push(1);

        let bytes = Bytes::from(data);
        let parsed = SetAlwaysRunMessage::parse(&bytes).unwrap();

        assert_eq!(parsed.agent_id, agent_id);
        assert_eq!(parsed.session_id, session_id);
        assert!(parsed.always_run);
    }

    #[test]
    fn test_agent_height_width_parsing() {
        let mut data = Vec::new();
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        data.extend_from_slice(agent_id.as_bytes());
        data.extend_from_slice(session_id.as_bytes());
        data.extend_from_slice(&12345u32.to_le_bytes());
        data.extend_from_slice(&1080u16.to_le_bytes());
        data.extend_from_slice(&1920u16.to_le_bytes());

        let bytes = Bytes::from(data);
        let parsed = AgentHeightWidthMessage::parse(&bytes).unwrap();

        assert_eq!(parsed.agent_id, agent_id);
        assert_eq!(parsed.height, 1080);
        assert_eq!(parsed.width, 1920);
    }
}
