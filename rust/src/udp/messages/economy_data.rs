use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone)]
pub struct EconomyDataMessage {
    pub object_capacity: i32,
    pub object_count: i32,
    pub price_energy_unit: i32,
    pub price_object_claim: i32,
    pub price_public_object_decay: i32,
    pub price_public_object_delete: i32,
    pub price_parcel_claim: i32,
    pub price_parcel_claim_factor: f32,
    pub price_upload: i32,
    pub price_rent_light: i32,
    pub teleport_min_price: i32,
    pub teleport_price_exponent: f32,
    pub energy_efficiency: f32,
    pub price_object_rent: f32,
    pub price_object_scale_factor: f32,
    pub price_parcel_rent: i32,
    pub price_group_create: i32,
}

impl Default for EconomyDataMessage {
    fn default() -> Self {
        Self {
            object_capacity: 45000,
            object_count: 0,
            price_energy_unit: 0,
            price_object_claim: 0,
            price_public_object_decay: 0,
            price_public_object_delete: 0,
            price_parcel_claim: 0,
            price_parcel_claim_factor: 1.0,
            price_upload: 0,
            price_rent_light: 0,
            teleport_min_price: 0,
            teleport_price_exponent: 0.0,
            energy_efficiency: 1.0,
            price_object_rent: 0.0,
            price_object_scale_factor: 10.0,
            price_parcel_rent: 0,
            price_group_create: 0,
        }
    }
}

impl EconomyDataMessage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(68);

        buf.put_i32_le(self.object_capacity);
        buf.put_i32_le(self.object_count);
        buf.put_i32_le(self.price_energy_unit);
        buf.put_i32_le(self.price_object_claim);
        buf.put_i32_le(self.price_public_object_decay);
        buf.put_i32_le(self.price_public_object_delete);
        buf.put_i32_le(self.price_parcel_claim);
        buf.put_f32_le(self.price_parcel_claim_factor);
        buf.put_i32_le(self.price_upload);
        buf.put_i32_le(self.price_rent_light);
        buf.put_i32_le(self.teleport_min_price);
        buf.put_f32_le(self.teleport_price_exponent);
        buf.put_f32_le(self.energy_efficiency);
        buf.put_f32_le(self.price_object_rent);
        buf.put_f32_le(self.price_object_scale_factor);
        buf.put_i32_le(self.price_parcel_rent);
        buf.put_i32_le(self.price_group_create);

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economy_data_serialization() {
        let message = EconomyDataMessage::new();
        let serialized = message.serialize();

        assert_eq!(serialized.len(), 68);

        assert_eq!(
            i32::from_le_bytes([serialized[0], serialized[1], serialized[2], serialized[3]]),
            45000
        );
    }

    #[test]
    fn test_economy_data_field_count() {
        let message = EconomyDataMessage::new();
        let serialized = message.serialize();

        assert_eq!(serialized.len(), 68);
    }
}
