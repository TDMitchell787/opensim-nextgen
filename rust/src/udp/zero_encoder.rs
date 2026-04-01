use bytes::{BytesMut, BufMut};

pub struct ZeroEncoder {
    dest: BytesMut,
    zero_count: usize,
}

impl ZeroEncoder {
    pub fn new(capacity: usize) -> Self {
        Self {
            dest: BytesMut::with_capacity(capacity),
            zero_count: 0,
        }
    }

    fn flush_zeros(&mut self) {
        if self.zero_count > 0 {
            // LLUDP zero encoding: 00 NN means NN zeros (1-255)
            // 00 FF = 255 zeros, not 256!
            while self.zero_count > 255 {
                self.dest.put_u8(0x00);
                self.dest.put_u8(0xFF);  // 255 zeros
                self.zero_count -= 255;  // Fixed: was incorrectly -= 256
            }
            if self.zero_count > 0 {
                self.dest.put_u8(0x00);
                self.dest.put_u8(self.zero_count as u8);
                self.zero_count = 0;
            }
        }
    }

    pub fn add_byte(&mut self, value: u8) {
        if value == 0 {
            if self.zero_count < 255 {
                self.zero_count += 1;
            } else {
                self.dest.put_u8(0x00);
                self.dest.put_u8(0xFF);
                self.zero_count = 1;
            }
        } else {
            self.flush_zeros();
            self.dest.put_u8(value);
        }
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte(byte);
        }
    }

    pub fn add_u32_le(&mut self, value: u32) {
        if value == 0 {
            self.zero_count += 4;
        } else {
            let bytes = value.to_le_bytes();
            self.add_bytes(&bytes);
        }
    }

    pub fn add_i32_le(&mut self, value: i32) {
        if value == 0 {
            self.zero_count += 4;
        } else {
            let bytes = value.to_le_bytes();
            self.add_bytes(&bytes);
        }
    }

    pub fn add_u64_le(&mut self, value: u64) {
        if value == 0 {
            self.zero_count += 8;
        } else {
            let bytes = value.to_le_bytes();
            self.add_bytes(&bytes);
        }
    }

    pub fn add_f32_le(&mut self, value: f32) {
        if value == 0.0 {
            self.zero_count += 4;
        } else {
            let bytes = value.to_le_bytes();
            self.add_bytes(&bytes);
        }
    }

    pub fn add_string(&mut self, s: &str, max_len: usize) {
        if s.is_empty() {
            self.zero_count += 1;
        } else {
            let bytes = s.as_bytes();
            let len = bytes.len().min(max_len);
            self.add_byte(len as u8);
            self.add_bytes(&bytes[..len]);
        }
    }

    pub fn add_uuid(&mut self, uuid: &uuid::Uuid) {
        let bytes = uuid.as_bytes();
        self.add_bytes(bytes);
    }

    pub fn add_zeros(&mut self, count: usize) {
        self.zero_count += count;
    }

    pub fn finish(mut self) -> BytesMut {
        self.flush_zeros();
        self.dest
    }

    pub fn len(&self) -> usize {
        // Calculate how many 00 NN pairs needed for zero_count zeros
        // Each 00 NN can encode up to 255 zeros
        self.dest.len() + if self.zero_count > 0 {
            let pairs = (self.zero_count + 254) / 255; // ceiling division
            2 * pairs
        } else {
            0
        }
    }
}

pub fn zero_decode(encoded: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(encoded.len() * 2);
    let mut i = 0;

    while i < encoded.len() {
        if encoded[i] == 0 {
            if i + 1 < encoded.len() {
                let count = encoded[i + 1] as usize;
                for _ in 0..count {
                    decoded.push(0);
                }
                i += 2;
            } else {
                decoded.push(0);
                i += 1;
            }
        } else {
            decoded.push(encoded[i]);
            i += 1;
        }
    }

    decoded
}
