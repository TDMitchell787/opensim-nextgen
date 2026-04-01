//! TransferInfo message builder
//!
//! Server sends this to acknowledge a TransferRequest and provide transfer metadata.
//! Message structure from message_template.msg:
//! TransferInfo Low 154 NotTrusted Zerocoded
//!   TransferInfo Single
//!     TransferID  LLUUID
//!     ChannelType S32
//!     TargetType  S32
//!     Status      S32
//!     Size        S32
//!     Params      Variable 2

use bytes::{BufMut, BytesMut};
use uuid::Uuid;

/// Transfer status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TransferStatus {
    Ok = 0,
    Done = 1,
    Skip = 2,
    Abort = 3,
    Error = 4,
    UnknownSource = 5,
    InsufficientPermissions = 6,
}

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub transfer_id: Uuid,
    pub channel_type: i32,
    pub target_type: i32,
    pub status: TransferStatus,
    pub size: i32,
    pub params: Vec<u8>,
}

impl TransferInfo {
    pub fn new(
        transfer_id: Uuid,
        channel_type: i32,
        target_type: i32,
        status: TransferStatus,
        size: i32,
    ) -> Self {
        Self {
            transfer_id,
            channel_type,
            target_type,
            status,
            size,
            params: Vec::new(),
        }
    }

    /// Create a successful transfer info for an asset
    pub fn new_asset_ok(transfer_id: Uuid, asset_size: i32) -> Self {
        Self::new(
            transfer_id,
            2, // ChannelType 2 = Asset
            2, // TargetType 2 = Asset
            TransferStatus::Ok,
            asset_size,
        )
    }

    /// Create an error response
    pub fn new_error(transfer_id: Uuid, channel_type: i32, status: TransferStatus) -> Self {
        Self::new(transfer_id, channel_type, 0, status, 0)
    }

    pub fn serialize(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(50 + self.params.len());

        // TransferID (16 bytes)
        buf.put_slice(self.transfer_id.as_bytes());

        // ChannelType (4 bytes)
        buf.put_i32_le(self.channel_type);

        // TargetType (4 bytes)
        buf.put_i32_le(self.target_type);

        // Status (4 bytes)
        buf.put_i32_le(self.status as i32);

        // Size (4 bytes)
        buf.put_i32_le(self.size);

        // Params length (2 bytes for Variable 2)
        buf.put_u16_le(self.params.len() as u16);

        // Params data
        buf.put_slice(&self.params);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_transfer_info() {
        let transfer_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let info = TransferInfo::new_asset_ok(transfer_id, 1024);

        let serialized = info.serialize();

        // Verify structure
        assert!(serialized.len() >= 38); // Minimum size without params

        // Verify transfer ID (first 16 bytes)
        assert_eq!(&serialized[0..16], transfer_id.as_bytes());

        // Verify channel type (bytes 16-19)
        let channel_type = i32::from_le_bytes([
            serialized[16],
            serialized[17],
            serialized[18],
            serialized[19],
        ]);
        assert_eq!(channel_type, 2);

        // Verify size (bytes 28-31)
        let size = i32::from_le_bytes([
            serialized[28],
            serialized[29],
            serialized[30],
            serialized[31],
        ]);
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_error_transfer_info() {
        let transfer_id = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let info = TransferInfo::new_error(
            transfer_id,
            2,
            TransferStatus::UnknownSource,
        );

        assert_eq!(info.status, TransferStatus::UnknownSource);
        assert_eq!(info.size, 0);
        assert_eq!(info.target_type, 0);
    }
}
