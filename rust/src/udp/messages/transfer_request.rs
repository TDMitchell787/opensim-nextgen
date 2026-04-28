//! TransferRequest message handling
//!
//! Viewer sends this to request asset data from the server.
//! Message structure from message_template.msg:
//! TransferRequest Low 153 NotTrusted Zerocoded
//!   TransferInfo Single
//!     TransferID  LLUUID
//!     ChannelType S32
//!     SourceType  S32
//!     Priority    F32
//!     Params      Variable 2

use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub transfer_id: Uuid,
    pub channel_type: i32,
    pub source_type: i32,
    pub priority: f32,
    pub params: Vec<u8>,
}

impl TransferRequest {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 36 {
            return Err(anyhow!(
                "TransferRequest data too short: {} bytes",
                data.len()
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        // TransferID (16 bytes)
        let mut transfer_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut transfer_id_bytes);
        let transfer_id = Uuid::from_bytes(transfer_id_bytes);

        // ChannelType (4 bytes, signed)
        let channel_type = cursor.get_i32_le();

        // SourceType (4 bytes, signed)
        let source_type = cursor.get_i32_le();

        // Priority (4 bytes, float)
        let priority = cursor.get_f32_le();

        // Params length (2 bytes for Variable 2)
        let params_len = cursor.get_u16_le() as usize;

        // Params data
        let mut params = vec![0u8; params_len];
        if cursor.remaining() >= params_len {
            cursor.copy_to_slice(&mut params);
        } else {
            return Err(anyhow!(
                "TransferRequest params truncated: expected {} bytes, got {}",
                params_len,
                cursor.remaining()
            ));
        }

        Ok(Self {
            transfer_id,
            channel_type,
            source_type,
            priority,
            params,
        })
    }

    /// Extract asset ID from params for SourceType 2 (asset)
    pub fn get_asset_id(&self) -> Option<Uuid> {
        if self.source_type == 2 && self.params.len() >= 20 {
            // Params for asset transfer:
            // UUID (16 bytes) - Asset ID
            // S32 (4 bytes) - Asset Type
            let mut asset_id_bytes = [0u8; 16];
            asset_id_bytes.copy_from_slice(&self.params[0..16]);
            Some(Uuid::from_bytes(asset_id_bytes))
        } else {
            None
        }
    }

    /// Extract asset type from params for SourceType 2 (asset)
    pub fn get_asset_type(&self) -> Option<i32> {
        if self.source_type == 2 && self.params.len() >= 20 {
            let mut cursor = std::io::Cursor::new(&self.params[16..20]);
            Some(cursor.get_i32_le())
        } else {
            None
        }
    }

    /// Extract sim inventory params for SourceType 3 (sim inventory)
    /// Params format:
    /// - AgentID (16 bytes)
    /// - SessionID (16 bytes)
    /// - OwnerID (16 bytes)
    /// - TaskID (16 bytes) - Object containing the inventory item
    /// - ItemID (16 bytes) - The inventory item itself
    pub fn get_sim_inventory_params(&self) -> Option<SimInventoryParams> {
        if self.source_type == 3 && self.params.len() >= 80 {
            let mut agent_id_bytes = [0u8; 16];
            let mut session_id_bytes = [0u8; 16];
            let mut owner_id_bytes = [0u8; 16];
            let mut task_id_bytes = [0u8; 16];
            let mut item_id_bytes = [0u8; 16];

            agent_id_bytes.copy_from_slice(&self.params[0..16]);
            session_id_bytes.copy_from_slice(&self.params[16..32]);
            owner_id_bytes.copy_from_slice(&self.params[32..48]);
            task_id_bytes.copy_from_slice(&self.params[48..64]);
            item_id_bytes.copy_from_slice(&self.params[64..80]);

            Some(SimInventoryParams {
                agent_id: Uuid::from_bytes(agent_id_bytes),
                session_id: Uuid::from_bytes(session_id_bytes),
                owner_id: Uuid::from_bytes(owner_id_bytes),
                task_id: Uuid::from_bytes(task_id_bytes),
                item_id: Uuid::from_bytes(item_id_bytes),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimInventoryParams {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub owner_id: Uuid,
    pub task_id: Uuid,
    pub item_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transfer_request() {
        let mut data = Vec::new();

        // TransferID
        data.extend_from_slice(&[
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ]);

        // ChannelType (2 = asset)
        data.extend_from_slice(&2i32.to_le_bytes());

        // SourceType (2 = asset)
        data.extend_from_slice(&2i32.to_le_bytes());

        // Priority (1.0)
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Params length (20 = UUID + asset type)
        data.extend_from_slice(&20u16.to_le_bytes());

        // Params: Asset UUID
        data.extend_from_slice(&[
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99,
        ]);

        // Params: Asset Type (0 = texture)
        data.extend_from_slice(&0i32.to_le_bytes());

        let request = TransferRequest::parse(&data).unwrap();

        assert_eq!(request.channel_type, 2);
        assert_eq!(request.source_type, 2);
        assert_eq!(request.priority, 1.0);
        assert_eq!(request.params.len(), 20);

        let asset_id = request.get_asset_id().unwrap();
        assert_eq!(asset_id.as_bytes()[0], 0xaa);

        let asset_type = request.get_asset_type().unwrap();
        assert_eq!(asset_type, 0);
    }
}
