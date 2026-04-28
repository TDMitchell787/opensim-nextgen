use bytes::{BufMut, BytesMut};
use std::f32::consts::PI;

pub const PATCHES_PER_EDGE: usize = 16;
pub const END_OF_PATCHES: u8 = 97;
const OO_SQRT2: f32 = 0.7071067811865475244008443621049;
const STRIDE: i32 = 264;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum LayerType {
    Land = 0x4C,
    LandExtended = 0x4D,
    Water = 0x57,
    WaterExtended = 0x58,
    Wind = 0x37,
    WindExtended = 0x39,
    Cloud = 0x38,
    CloudExtended = 0x3A,
}

#[derive(Debug, Clone)]
pub struct GroupHeader {
    pub stride: i32,
    pub patch_size: i32,
    pub layer_type: LayerType,
}

impl Default for GroupHeader {
    fn default() -> Self {
        Self {
            stride: STRIDE,
            patch_size: 16,
            layer_type: LayerType::Land,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatchHeader {
    pub dc_offset: f32,
    pub range: i32,
    pub quant_wbits: i32,
    pub patch_ids: i32,
    pub word_bits: u32,
    pub large_region: bool,
}

impl PatchHeader {
    pub fn new() -> Self {
        Self {
            dc_offset: 0.0,
            range: 0,
            quant_wbits: 136,
            patch_ids: 0,
            word_bits: 0,
            large_region: false,
        }
    }

    pub fn get_x(&self) -> i32 {
        if self.large_region {
            self.patch_ids >> 16
        } else {
            self.patch_ids >> 5
        }
    }

    pub fn get_y(&self) -> i32 {
        if self.large_region {
            self.patch_ids & 0xFFFF
        } else {
            self.patch_ids & 0x1F
        }
    }

    pub fn set_patch_ids(&mut self, x: i32, y: i32) {
        self.patch_ids = if self.large_region {
            (x << 16) | (y & 0xFFFF)
        } else {
            (x << 5) | (y & 0x1F)
        };
    }
}

#[derive(Debug, Clone)]
pub struct TerrainPatch {
    pub x: i32,
    pub y: i32,
    pub data: Vec<f32>,
}

impl TerrainPatch {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            data: vec![0.0; 16 * 16],
        }
    }

    pub fn with_data(x: i32, y: i32, data: Vec<f32>) -> anyhow::Result<Self> {
        if data.len() != 16 * 16 {
            anyhow::bail!("Patch data must be 16x16 (256 floats), got {}", data.len());
        }
        Ok(Self { x, y, data })
    }
}

pub struct BitPack {
    data: BytesMut,
    bit_pos: usize,
}

impl BitPack {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: BytesMut::with_capacity(capacity),
            bit_pos: 0,
        }
    }

    pub fn pack_bits(&mut self, value: i32, bits: usize) {
        if bits == 0 {
            return;
        }

        // Phase 70.17 FIX: Match LibreMetaverse BitPack.PackBits exactly
        // LibreMetaverse does:
        // 1. Convert value to little-endian bytes
        // 2. Pack each byte's bits in MSB-first order (bit 7 down to bit 0)
        // 3. Pack bytes in order (byte 0, then byte 1, etc.)
        //
        // For PackBits(264, 16) where 264 = 0x0108:
        // - LE bytes: [0x08, 0x01, 0x00, 0x00]
        // - Pack 8 bits from 0x08: 0,0,0,0,1,0,0,0
        // - Pack 8 bits from 0x01: 0,0,0,0,0,0,0,1
        // - Output: 0x08 0x01

        let le_bytes = (value as u32).to_le_bytes();
        let mut bits_remaining = bits;
        let mut cur_byte_idx = 0;

        while bits_remaining > 0 {
            let bits_this_byte = bits_remaining.min(8);

            // Extract bits from current byte, MSB-first
            // LibreMetaverse checks (data[curBytePos] & (0x01 << (count - 1)))
            // where count goes from 8 down to 1, so it checks bit 7, 6, 5, ..., 0
            for bit_offset in (0..bits_this_byte).rev() {
                let src_bit = (le_bytes[cur_byte_idx] >> bit_offset) & 1;

                let out_byte_pos = self.bit_pos / 8;
                let out_bit_offset = self.bit_pos % 8;
                let cur_bit: u8 = 0x80 >> out_bit_offset;

                while self.data.len() <= out_byte_pos {
                    self.data.put_u8(0);
                }

                if src_bit != 0 {
                    self.data[out_byte_pos] |= cur_bit;
                } else {
                    self.data[out_byte_pos] &= !cur_bit;
                }

                self.bit_pos += 1;
            }

            bits_remaining -= bits_this_byte;
            cur_byte_idx += 1;
        }
    }

    pub fn pack_float(&mut self, value: f32) {
        let bytes = value.to_le_bytes();
        for &byte in &bytes {
            self.pack_bits(byte as i32, 8);
        }
    }

    pub fn byte_pos(&self) -> usize {
        (self.bit_pos + 7) / 8
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn into_bytes(self) -> BytesMut {
        self.data
    }
}

pub struct TerrainCompressor {
    dequantize_table16: Vec<f32>,
    cosine_table16: Vec<f32>,
    copy_matrix16: Vec<usize>,
    quantize_table16: Vec<f32>,
}

impl TerrainCompressor {
    pub fn new() -> Self {
        let mut compressor = Self {
            dequantize_table16: vec![0.0; 16 * 16],
            cosine_table16: vec![0.0; 16 * 16],
            copy_matrix16: vec![0; 16 * 16],
            quantize_table16: vec![0.0; 16 * 16],
        };

        compressor.build_dequantize_table16();
        compressor.setup_cosines16();
        compressor.build_copy_matrix16();
        compressor.build_quantize_table16();

        compressor
    }

    fn build_dequantize_table16(&mut self) {
        for j in 0..16 {
            for i in 0..16 {
                self.dequantize_table16[j * 16 + i] = 1.0 + 2.0 * (i + j) as f32;
            }
        }
    }

    fn build_quantize_table16(&mut self) {
        for j in 0..16 {
            for i in 0..16 {
                self.quantize_table16[j * 16 + i] = 1.0 / (1.0 + 2.0 * (i + j) as f32);
            }
        }
    }

    fn setup_cosines16(&mut self) {
        let hposz = PI * 0.5 / 16.0;

        for u in 0..16 {
            for n in 0..16 {
                self.cosine_table16[u * 16 + n] = ((2.0 * n as f32 + 1.0) * u as f32 * hposz).cos();
            }
        }
    }

    fn build_copy_matrix16(&mut self) {
        let mut diag = false;
        let mut right = true;
        let mut i = 0;
        let mut j = 0;
        let mut count = 0;

        while i < 16 && j < 16 {
            self.copy_matrix16[j * 16 + i] = count;
            count += 1;

            if !diag {
                if right {
                    if i < 15 {
                        i += 1;
                    } else {
                        j += 1;
                    }
                    right = false;
                    diag = true;
                } else {
                    if j < 15 {
                        j += 1;
                    } else {
                        i += 1;
                    }
                    right = true;
                    diag = true;
                }
            } else {
                if right {
                    i += 1;
                    j -= 1;
                    if i == 15 || j == 0 {
                        diag = false;
                    }
                } else {
                    i -= 1;
                    j += 1;
                    if j == 15 || i == 0 {
                        diag = false;
                    }
                }
            }
        }
    }

    pub fn prescan_patch(&self, patch: &[f32]) -> anyhow::Result<PatchHeader> {
        if patch.len() != 16 * 16 {
            anyhow::bail!("Patch must be 16x16 (256 floats), got {}", patch.len());
        }

        let mut zmax = -99999999.0f32;
        let mut zmin = 99999999.0f32;

        for val in patch {
            if *val > zmax {
                zmax = *val;
            }
            if *val < zmin {
                zmin = *val;
            }
        }

        let mut header = PatchHeader::new();
        header.dc_offset = zmin;
        header.range = ((zmax - zmin) + 1.0) as i32;

        Ok(header)
    }

    fn dct_line16(&self, linein: &[f32], lineout: &mut [f32], line: usize) {
        let line_size = line * 16;

        let mut total = 0.0f32;
        for n in 0..16 {
            total += linein[line_size + n];
        }
        lineout[line_size] = OO_SQRT2 * total;

        for u in 1..16 {
            total = 0.0;
            for n in 0..16 {
                total += linein[line_size + n] * self.cosine_table16[u * 16 + n];
            }
            lineout[line_size + u] = total;
        }
    }

    fn dct_column16(&self, linein: &[f32], lineout: &mut [i32], column: usize) {
        const OOSOB: f32 = 2.0 / 16.0;

        let mut total = 0.0f32;

        for n in 0..16 {
            total += linein[16 * n + column];
        }

        lineout[self.copy_matrix16[column]] =
            (OO_SQRT2 * total * OOSOB * self.quantize_table16[column]) as i32;

        for u in 1..16 {
            total = 0.0;

            for n in 0..16 {
                total += linein[16 * n + column] * self.cosine_table16[u * 16 + n];
            }

            lineout[self.copy_matrix16[16 * u + column]] =
                (total * OOSOB * self.quantize_table16[16 * u + column]) as i32;
        }
    }

    pub fn compress_patch(
        &self,
        patch_data: &[f32],
        header: &mut PatchHeader,
        prequant: i32,
    ) -> anyhow::Result<Vec<i32>> {
        if patch_data.len() != 16 * 16 {
            anyhow::bail!(
                "Patch data must be 16x16 (256 floats), got {}",
                patch_data.len()
            );
        }

        let mut block = vec![0.0f32; 16 * 16];
        let wordsize = prequant;
        let oozrange = 1.0 / header.range as f32;
        let range = (1 << prequant) as f32;
        let premult = oozrange * range;
        let sub = (1 << (prequant - 1)) as f32 + header.dc_offset * premult;

        header.quant_wbits = wordsize - 2;
        header.quant_wbits |= (prequant - 2) << 4;

        let mut k = 0;
        for j in 0..16 {
            for i in 0..16 {
                block[k] = patch_data[j * 16 + i] * premult - sub;
                k += 1;
            }
        }

        let mut ftemp = vec![0.0f32; 16 * 16];
        let mut itemp = vec![0i32; 16 * 16];

        for o in 0..16 {
            self.dct_line16(&block, &mut ftemp, o);
        }
        for o in 0..16 {
            self.dct_column16(&ftemp, &mut itemp, o);
        }

        Ok(itemp)
    }

    pub fn encode_patch_header(
        &self,
        output: &mut BitPack,
        header: &mut PatchHeader,
        patch: &[i32],
    ) -> usize {
        let mut wbits = (header.quant_wbits & 0x0F) + 2;
        let max_wbits = wbits as u32 + 5;
        let min_wbits = (wbits as u32) >> 1;

        wbits = min_wbits as i32;

        for &val in patch {
            let mut temp = val;
            if temp == 0 {
                continue;
            }

            if temp < 0 {
                temp = -temp;
            }

            for j in (min_wbits as i32..=max_wbits as i32).rev() {
                if (temp & (1 << j)) != 0 {
                    if j > wbits {
                        wbits = j;
                    }
                    break;
                }
            }
        }

        wbits += 1;

        header.quant_wbits &= 0xF0;
        header.quant_wbits |= wbits - 2;

        output.pack_bits(header.quant_wbits, 8);
        output.pack_float(header.dc_offset);
        output.pack_bits(header.range, 16);
        output.pack_bits(header.patch_ids, if header.large_region { 32 } else { 10 });

        wbits as usize
    }

    pub fn encode_patch(&self, output: &mut BitPack, patch: &[i32], wbits: usize) {
        const ZERO_CODE: i32 = 0x0;
        const ZERO_EOB: i32 = 0x2;
        const POSITIVE_VALUE: i32 = 0x6;
        const NEGATIVE_VALUE: i32 = 0x7;

        for i in 0..16 * 16 {
            let temp = patch[i];

            if temp == 0 {
                let mut eob = true;

                for j in i..16 * 16 {
                    if patch[j] != 0 {
                        eob = false;
                        break;
                    }
                }

                if eob {
                    output.pack_bits(ZERO_EOB, 2);
                    return;
                }
                output.pack_bits(ZERO_CODE, 1);
            } else {
                if temp < 0 {
                    let mut abs_temp = -temp;
                    if abs_temp > (1 << wbits) {
                        abs_temp = 1 << wbits;
                    }

                    output.pack_bits(NEGATIVE_VALUE, 3);
                    output.pack_bits(abs_temp, wbits);
                } else {
                    let mut pos_temp = temp;
                    if pos_temp > (1 << wbits) {
                        pos_temp = 1 << wbits;
                    }

                    output.pack_bits(POSITIVE_VALUE, 3);
                    output.pack_bits(pos_temp, wbits);
                }
            }
        }
    }

    pub fn create_patch(
        &self,
        output: &mut BitPack,
        patch_data: &[f32],
        x: i32,
        y: i32,
    ) -> anyhow::Result<()> {
        self.create_patch_ex(output, patch_data, x, y, false)
    }

    pub fn create_patch_ex(
        &self,
        output: &mut BitPack,
        patch_data: &[f32],
        x: i32,
        y: i32,
        large_region: bool,
    ) -> anyhow::Result<()> {
        let mut header = self.prescan_patch(patch_data)?;
        header.large_region = large_region;
        header.quant_wbits = 130; // Match OpenSim default
        header.set_patch_ids(x, y);

        // Phase 70.8: Flat terrain optimization (matches OpenSim TerrainCompressor.cs:162-176)
        // When range ≈ 1.0, all heights are identical - use simplified encoding
        let frange = header.range as f32;
        let rounded_range = (frange * 100.0).round() / 100.0;
        tracing::debug!(
            "[TERRAIN] Patch ({},{}) range={:.4} rounded={:.2} dc_offset={:.2}",
            x,
            y,
            frange,
            rounded_range,
            header.dc_offset
        );
        if rounded_range == 1.0 {
            tracing::info!(
                "[TERRAIN] Using FLAT terrain encoding for patch ({},{}) - range={:.4}",
                x,
                y,
                frange
            );
            // Flat terrain: QuantWBits=0, DCOffset-0.5, range=1, patchIds, ZERO_EOB
            const ZERO_EOB: i32 = 0x2;

            output.pack_bits(0, 8); // QuantWBits = 0
            output.pack_float(header.dc_offset - 0.5); // DCOffset - 0.5
            output.pack_bits(1, 8); // Range low byte = 1
            output.pack_bits(0, 8); // Range high byte = 0
            output.pack_bits(header.patch_ids, if header.large_region { 32 } else { 10 }); // Patch IDs
            output.pack_bits(ZERO_EOB, 2); // End of block
            return Ok(());
        }

        let patch = self.compress_patch(patch_data, &mut header, 10)?;
        let wbits = self.encode_patch_header(output, &mut header, &patch);
        self.encode_patch(output, &patch, wbits);

        Ok(())
    }

    pub fn create_layer_data_packet(
        &self,
        patches: &[TerrainPatch],
        layer_type: LayerType,
    ) -> anyhow::Result<Vec<u8>> {
        self.create_layer_data_packet_ex(patches, layer_type, false)
    }

    pub fn create_layer_data_packet_ex(
        &self,
        patches: &[TerrainPatch],
        layer_type: LayerType,
        large_region: bool,
    ) -> anyhow::Result<Vec<u8>> {
        let header = GroupHeader {
            stride: STRIDE,
            patch_size: 16,
            layer_type,
        };

        let mut data = Vec::with_capacity(patches.len() * 16 * 16 * 2);
        let mut bitpack = BitPack::new(patches.len() * 16 * 16 * 2);

        bitpack.pack_bits(header.stride, 16);
        bitpack.pack_bits(header.patch_size, 8);
        bitpack.pack_bits(header.layer_type as i32, 8);

        for patch in patches {
            self.create_patch_ex(&mut bitpack, &patch.data, patch.x, patch.y, large_region)?;
        }

        bitpack.pack_bits(END_OF_PATCHES as i32, 8);

        let byte_len = bitpack.byte_pos();
        data.extend_from_slice(&bitpack.data()[..byte_len]);

        Ok(data)
    }
}

impl Default for TerrainCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitpack() {
        let mut bp = BitPack::new(128);

        bp.pack_bits(15, 4);
        bp.pack_bits(255, 8);
        bp.pack_bits(1023, 10);

        assert_eq!(bp.byte_pos(), 3);
        assert_eq!(bp.data()[0], 0xFF);
        assert_eq!(bp.data()[1], 0xFF);
        assert_eq!(bp.data()[2], 0xFC);
    }

    #[test]
    fn test_bitpack_msb_ordering() {
        let mut bp = BitPack::new(128);

        bp.pack_bits(264, 16);
        assert_eq!(bp.data()[0], 0x08);
        assert_eq!(bp.data()[1], 0x01);

        let mut bp2 = BitPack::new(128);
        bp2.pack_bits(0x4C, 8);
        assert_eq!(bp2.data()[0], 0x4C);

        let mut bp3 = BitPack::new(128);
        bp3.pack_bits(5, 4);
        assert_eq!(bp3.data()[0], 0x50);

        let mut bp4 = BitPack::new(128);
        bp4.pack_bits(163, 10);
        assert_eq!(bp4.data()[0], 0xA3);
        assert_eq!(bp4.data()[1] & 0xC0, 0x00);
    }

    #[test]
    fn test_pack_float_no_alignment() {
        let mut bp = BitPack::new(128);
        bp.pack_bits(0xFF, 8);
        bp.pack_bits(0x05, 4);
        bp.pack_float(21.0f32);
        assert_eq!(bp.byte_pos(), (8 + 4 + 32 + 7) / 8);
    }

    #[test]
    fn test_patch_header_ids() {
        let mut header = PatchHeader::new();
        header.set_patch_ids(5, 10);

        assert_eq!(header.get_x(), 5);
        assert_eq!(header.get_y(), 10);
    }

    #[test]
    fn test_terrain_compressor_init() {
        let compressor = TerrainCompressor::new();
        assert_eq!(compressor.dequantize_table16.len(), 256);
        assert_eq!(compressor.cosine_table16.len(), 256);
        assert_eq!(compressor.copy_matrix16.len(), 256);
        assert_eq!(compressor.quantize_table16.len(), 256);
    }

    #[test]
    fn test_prescan_patch() {
        let compressor = TerrainCompressor::new();
        let mut patch = vec![20.0f32; 256];
        patch[0] = 10.0;
        patch[255] = 30.0;

        let header = compressor.prescan_patch(&patch).unwrap();
        assert_eq!(header.dc_offset, 10.0);
        assert_eq!(header.range, 21);
    }
}
