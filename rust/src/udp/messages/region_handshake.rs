use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use uuid::Uuid;
use crate::region::data_model::RegionInfo;

#[derive(Debug, Clone)]
pub struct RegionHandshakeMessage {
    pub region_flags: u64,
    pub sim_access: u8,
    pub sim_name: String,
    pub sim_owner: Uuid,
    pub is_estate_manager: bool,
    pub water_height: f32,
    pub billable_factor: f32,
    pub cache_id: Uuid,
    pub terrain_base_0: Uuid,
    pub terrain_base_1: Uuid,
    pub terrain_base_2: Uuid,
    pub terrain_base_3: Uuid,
    pub terrain_detail_0: Uuid,
    pub terrain_detail_1: Uuid,
    pub terrain_detail_2: Uuid,
    pub terrain_detail_3: Uuid,
    pub terrain_start_height_00: f32,
    pub terrain_start_height_01: f32,
    pub terrain_start_height_10: f32,
    pub terrain_start_height_11: f32,
    pub terrain_height_range_00: f32,
    pub terrain_height_range_01: f32,
    pub terrain_height_range_10: f32,
    pub terrain_height_range_11: f32,
    pub region_id: Uuid,
    pub cpu_class_id: u32,
    pub cpu_ratio: u32,
    pub colo_name: String,
    pub product_sku: String,
    pub product_name: String,
    pub region_flags_extended: u64,
    pub region_protocols: u64,
}

impl RegionHandshakeMessage {
    pub const DEFAULT_TERRAIN_TEXTURE_1: Uuid = uuid::uuid!("b8d3965a-ad78-bf43-699b-bff8eca6c975");
    pub const DEFAULT_TERRAIN_TEXTURE_2: Uuid = uuid::uuid!("abb783e6-3e93-26c0-248a-247666855da3");
    pub const DEFAULT_TERRAIN_TEXTURE_3: Uuid = uuid::uuid!("179cdabd-398a-9b6b-1391-4dc333ba321f");
    pub const DEFAULT_TERRAIN_TEXTURE_4: Uuid = uuid::uuid!("beb169c7-11ea-fff2-efe5-0f24dc881df2");

    pub fn new(region_name: String, region_id: Uuid, sim_owner: Uuid) -> Self {
        // Phase 71.15: OpenSim sends ZEROS for TerrainBase0-3 (see LLClientView.cs:921)
        // "this seem now obsolete, sending zero uuids"
        let zero_uuid = Uuid::nil();

        Self {
            region_flags: Self::compute_default_region_flags(),
            sim_access: 0,
            sim_name: region_name,
            sim_owner,
            is_estate_manager: false,
            water_height: 20.0,
            billable_factor: 1.0,
            cache_id: Uuid::new_v4(),
            // Phase 71.15: TerrainBase0-3 MUST be zeros per OpenSim protocol
            terrain_base_0: zero_uuid,
            terrain_base_1: zero_uuid,
            terrain_base_2: zero_uuid,
            terrain_base_3: zero_uuid,
            // TerrainDetail0-3 get actual texture UUIDs
            terrain_detail_0: Self::DEFAULT_TERRAIN_TEXTURE_1,
            terrain_detail_1: Self::DEFAULT_TERRAIN_TEXTURE_2,
            terrain_detail_2: Self::DEFAULT_TERRAIN_TEXTURE_3,
            terrain_detail_3: Self::DEFAULT_TERRAIN_TEXTURE_4,
            terrain_start_height_00: 10.0,
            terrain_start_height_01: 10.0,
            terrain_start_height_10: 10.0,
            terrain_start_height_11: 10.0,
            terrain_height_range_00: 60.0,
            terrain_height_range_01: 60.0,
            terrain_height_range_10: 60.0,
            terrain_height_range_11: 60.0,
            region_id,
            cpu_class_id: 9,
            cpu_ratio: 1,
            colo_name: String::new(),
            product_sku: String::new(),
            product_name: "OpenSim".to_string(),
            region_flags_extended: Self::compute_default_region_flags(),
            region_protocols: 1u64 << 63,
        }
    }

    pub fn from_region_info(region_info: &RegionInfo) -> Self {
        Self::new(
            region_info.region_name.clone(),
            region_info.region_id,
            region_info.owner_id.unwrap_or(region_info.master_avatar_id),
        )
    }

    fn compute_default_region_flags() -> u64 {
        let mut flags = 0u64;
        flags |= 1 << 0;
        flags |= 1 << 1;
        flags |= 1 << 2;
        flags |= 1 << 3;
        flags |= 1 << 21;
        flags
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(512);

        // Phase 68.13: RegionInfo block is "Single" - NO block count byte!
        // This was causing message corruption that prevented viewer from going in-world

        buffer.put_u32_le(self.region_flags as u32);
        buffer.put_u8(self.sim_access);

        let sim_name_bytes = self.sim_name.as_bytes();
        buffer.put_u8(sim_name_bytes.len() as u8);
        buffer.put_slice(sim_name_bytes);

        buffer.put_slice(self.sim_owner.as_bytes());
        buffer.put_u8(self.is_estate_manager as u8);
        buffer.put_f32_le(self.water_height);
        buffer.put_f32_le(self.billable_factor);
        buffer.put_slice(self.cache_id.as_bytes());

        buffer.put_slice(self.terrain_base_0.as_bytes());
        buffer.put_slice(self.terrain_base_1.as_bytes());
        buffer.put_slice(self.terrain_base_2.as_bytes());
        buffer.put_slice(self.terrain_base_3.as_bytes());
        buffer.put_slice(self.terrain_detail_0.as_bytes());
        buffer.put_slice(self.terrain_detail_1.as_bytes());
        buffer.put_slice(self.terrain_detail_2.as_bytes());
        buffer.put_slice(self.terrain_detail_3.as_bytes());

        buffer.put_f32_le(self.terrain_start_height_00);
        buffer.put_f32_le(self.terrain_start_height_01);
        buffer.put_f32_le(self.terrain_start_height_10);
        buffer.put_f32_le(self.terrain_start_height_11);
        buffer.put_f32_le(self.terrain_height_range_00);
        buffer.put_f32_le(self.terrain_height_range_01);
        buffer.put_f32_le(self.terrain_height_range_10);
        buffer.put_f32_le(self.terrain_height_range_11);

        // Phase 68.13: RegionInfo2 block is "Single" - NO block count byte!
        buffer.put_slice(self.region_id.as_bytes());

        // Phase 68.13: RegionInfo3 block is "Single" - NO block count byte!
        buffer.put_i32_le(self.cpu_class_id as i32);
        buffer.put_i32_le(self.cpu_ratio as i32);
        let colo_bytes = self.colo_name.as_bytes();
        buffer.put_u8(colo_bytes.len() as u8);
        buffer.put_slice(colo_bytes);
        let sku_bytes = self.product_sku.as_bytes();
        buffer.put_u8(sku_bytes.len() as u8);
        buffer.put_slice(sku_bytes);
        let product_bytes = self.product_name.as_bytes();
        buffer.put_u8(product_bytes.len() as u8);
        buffer.put_slice(product_bytes);

        buffer.put_u8(1);
        buffer.put_u64_le(self.region_flags_extended);
        buffer.put_u64_le(self.region_protocols);

        buffer.to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct RegionHandshakeReplyMessage {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub flags: u32,
}

impl RegionHandshakeReplyMessage {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 36 {
            anyhow::bail!("RegionHandshakeReply packet too short: {} bytes (expected at least 36)", data.len());
        }

        let mut cursor = std::io::Cursor::new(data);

        // Use copy_to_slice + from_bytes (same as UseCircuitCode) for reliable parsing
        let mut agent_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut agent_id_bytes);
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        let mut session_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut session_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);

        let flags = cursor.get_u32();

        Ok(Self { agent_id, session_id, flags })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(36);

        buffer.put_u128(self.agent_id.as_u128());
        buffer.put_u128(self.session_id.as_u128());
        buffer.put_u32(self.flags);

        buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_handshake_reply_parse() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let flags: u32 = 0x00000001;

        let mut data = Vec::new();
        data.extend_from_slice(&agent_id.as_u128().to_be_bytes());
        data.extend_from_slice(&session_id.as_u128().to_be_bytes());
        data.extend_from_slice(&flags.to_be_bytes());

        let message = RegionHandshakeReplyMessage::parse(&data).unwrap();

        assert_eq!(message.agent_id, agent_id);
        assert_eq!(message.session_id, session_id);
        assert_eq!(message.flags, flags);
    }

    #[test]
    fn test_region_handshake_reply_roundtrip() {
        let original = RegionHandshakeReplyMessage {
            agent_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            flags: 0x12345678,
        };

        let serialized = original.serialize();
        let parsed = RegionHandshakeReplyMessage::parse(&serialized).unwrap();

        assert_eq!(original.agent_id, parsed.agent_id);
        assert_eq!(original.session_id, parsed.session_id);
        assert_eq!(original.flags, parsed.flags);
    }
}
