//! JPEG2000 (J2K/J2C) texture handling for Second Life viewer compatibility
//!
//! Second Life viewers use JPEG2000 codestream format (J2C) for all textures.
//! This module provides encoding and decoding utilities.
//!
//! Encoding uses the system's opj_compress tool from OpenJPEG.
//! Decoding uses the jpeg2k Rust crate.

use anyhow::{Result, anyhow};
use tracing::{debug, warn, info};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::process::Command;
use std::path::Path;

pub struct J2KCodec {
    reduction_factor: u32,
}

impl Default for J2KCodec {
    fn default() -> Self {
        Self {
            reduction_factor: 0,
        }
    }
}

impl J2KCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_reduction(mut self, factor: u32) -> Self {
        self.reduction_factor = factor;
        self
    }

    pub fn decode_j2k(&self, j2k_data: &[u8]) -> Result<DecodedImage> {
        if j2k_data.len() < 4 {
            return Err(anyhow!("J2K data too short"));
        }

        if !is_valid_j2k(j2k_data) && !is_valid_jp2(j2k_data) {
            return Err(anyhow!("Invalid J2K/JP2 format marker"));
        }

        let params = jpeg2k::DecodeParameters::default()
            .reduce(self.reduction_factor);

        let image = jpeg2k::Image::from_bytes_with(j2k_data, params)
            .map_err(|e| anyhow!("Failed to decode J2K: {:?}", e))?;

        let width = image.width();
        let height = image.height();

        let dynamic_image: DynamicImage = DynamicImage::try_from(&image)
            .map_err(|e| anyhow!("Failed to convert J2K to DynamicImage: {:?}", e))?;

        let rgba = dynamic_image.to_rgba8();
        let pixels = rgba.into_raw();
        let has_alpha = dynamic_image.color().has_alpha();

        debug!("Decoded J2K image: {}x{}, has_alpha={}", width, height, has_alpha);

        Ok(DecodedImage {
            width,
            height,
            pixels,
            has_alpha,
        })
    }

    pub fn decode_to_png(&self, j2k_data: &[u8]) -> Result<Vec<u8>> {
        let decoded = self.decode_j2k(j2k_data)?;
        decoded.to_png()
    }

    pub fn decode_to_rgba(&self, j2k_data: &[u8]) -> Result<RgbaImage> {
        let decoded = self.decode_j2k(j2k_data)?;
        ImageBuffer::from_raw(decoded.width, decoded.height, decoded.pixels)
            .ok_or_else(|| anyhow!("Failed to create RGBA buffer"))
    }

    pub fn encode_image_to_j2k(&self, image: &DynamicImage) -> Result<Vec<u8>> {
        J2KEncoder::new().encode(image)
    }

    pub fn encode_png_to_j2k(&self, png_data: &[u8]) -> Result<Vec<u8>> {
        let image = image::load_from_memory_with_format(png_data, image::ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to load PNG: {}", e))?;
        self.encode_image_to_j2k(&image)
    }
}

#[derive(Debug, Clone)]
pub struct J2KEncoder {
    quality: Option<u32>,
    num_resolutions: u32,
    lossless: bool,
}

impl Default for J2KEncoder {
    fn default() -> Self {
        Self {
            quality: None,
            num_resolutions: 6,
            lossless: false,
        }
    }
}

impl J2KEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_quality(mut self, quality: u32) -> Self {
        self.quality = Some(quality.min(100));
        self.lossless = false;
        self
    }

    pub fn lossless(mut self) -> Self {
        self.lossless = true;
        self.quality = None;
        self
    }

    pub fn with_resolutions(mut self, num: u32) -> Self {
        self.num_resolutions = num.max(1).min(10);
        self
    }

    pub fn encode(&self, image: &DynamicImage) -> Result<Vec<u8>> {
        let resized = resize_to_power_of_two(image);

        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let input_path = temp_dir.join(format!("opensim_j2k_input_{}.png", timestamp));
        let output_path = temp_dir.join(format!("opensim_j2k_output_{}.j2k", timestamp));

        resized.save(&input_path)
            .map_err(|e| anyhow!("Failed to save temp PNG: {}", e))?;

        let result = self.run_opj_compress(&input_path, &output_path);

        let _ = std::fs::remove_file(&input_path);

        match result {
            Ok(_) => {
                let j2k_data = std::fs::read(&output_path)
                    .map_err(|e| anyhow!("Failed to read J2K output: {}", e))?;
                let _ = std::fs::remove_file(&output_path);

                if !is_valid_j2k(&j2k_data) {
                    return Err(anyhow!("opj_compress produced invalid J2K output"));
                }

                info!("Encoded J2K image: {}x{} -> {} bytes",
                      resized.width(), resized.height(), j2k_data.len());
                Ok(j2k_data)
            }
            Err(e) => {
                let _ = std::fs::remove_file(&output_path);
                Err(e)
            }
        }
    }

    fn run_opj_compress(&self, input: &Path, output: &Path) -> Result<()> {
        let mut cmd = Command::new("opj_compress");

        cmd.arg("-i").arg(input)
           .arg("-o").arg(output);

        if self.lossless {
            // Lossless mode (default for opj_compress)
        } else if let Some(quality) = self.quality {
            let rate = 100.0 / quality as f64;
            cmd.arg("-r").arg(format!("{:.1}", rate));
        } else {
            cmd.arg("-r").arg("20");
        }

        cmd.arg("-n").arg(self.num_resolutions.to_string());

        let output_result = cmd.output()
            .map_err(|e| anyhow!("Failed to run opj_compress: {}. Is OpenJPEG installed?", e))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(anyhow!("opj_compress failed: {}", stderr));
        }

        Ok(())
    }

    pub fn encode_rgba(&self, width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>> {
        let img: RgbaImage = ImageBuffer::from_raw(width, height, pixels.to_vec())
            .ok_or_else(|| anyhow!("Invalid pixel data for {}x{}", width, height))?;
        self.encode(&DynamicImage::ImageRgba8(img))
    }
}

pub fn check_opj_compress_available() -> bool {
    Command::new("opj_compress")
        .arg("--help")
        .output()
        .map(|o| o.status.success() || !o.stderr.is_empty())
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub has_alpha: bool,
}

impl DecodedImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>, has_alpha: bool) -> Self {
        Self { width, height, pixels, has_alpha }
    }

    pub fn from_png(png_data: &[u8]) -> Result<Self> {
        let img = image::load_from_memory_with_format(png_data, image::ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to load PNG: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let has_alpha = img.color().has_alpha();

        Ok(Self {
            width,
            height,
            pixels: rgba.into_raw(),
            has_alpha,
        })
    }

    pub fn from_tga(tga_data: &[u8]) -> Result<Self> {
        let img = image::load_from_memory_with_format(tga_data, image::ImageFormat::Tga)
            .map_err(|e| anyhow!("Failed to load TGA: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let has_alpha = img.color().has_alpha();

        Ok(Self {
            width,
            height,
            pixels: rgba.into_raw(),
            has_alpha,
        })
    }

    pub fn from_any(data: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(data)
            .map_err(|e| anyhow!("Failed to load image: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let has_alpha = img.color().has_alpha();

        Ok(Self {
            width,
            height,
            pixels: rgba.into_raw(),
            has_alpha,
        })
    }

    pub fn to_png(&self) -> Result<Vec<u8>> {
        let img: RgbaImage = ImageBuffer::from_raw(
            self.width,
            self.height,
            self.pixels.clone()
        ).ok_or_else(|| anyhow!("Failed to create image buffer"))?;

        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);

        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to encode PNG: {}", e))?;

        Ok(png_data)
    }

    pub fn to_dynamic_image(&self) -> Result<DynamicImage> {
        let img: RgbaImage = ImageBuffer::from_raw(
            self.width,
            self.height,
            self.pixels.clone()
        ).ok_or_else(|| anyhow!("Failed to create image buffer"))?;

        Ok(DynamicImage::ImageRgba8(img))
    }

    pub fn to_j2k(&self) -> Result<Vec<u8>> {
        let image = self.to_dynamic_image()?;
        J2KEncoder::new().encode(&image)
    }

    pub fn to_j2k_lossless(&self) -> Result<Vec<u8>> {
        let image = self.to_dynamic_image()?;
        J2KEncoder::new().lossless().encode(&image)
    }

    pub fn to_j2k_with_quality(&self, quality: u32) -> Result<Vec<u8>> {
        let image = self.to_dynamic_image()?;
        J2KEncoder::new().with_quality(quality).encode(&image)
    }

    pub fn solid_color(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let pixel_count = (width * height) as usize;
        let mut pixels = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
            pixels.push(a);
        }
        Self {
            width,
            height,
            pixels,
            has_alpha: a < 255,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 >= self.pixels.len() {
            return None;
        }
        Some([
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ])
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.pixels.len() {
            self.pixels[idx] = rgba[0];
            self.pixels[idx + 1] = rgba[1];
            self.pixels[idx + 2] = rgba[2];
            self.pixels[idx + 3] = rgba[3];
        }
    }
}

pub fn is_valid_j2k(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }
    data[0] == 0xFF && data[1] == 0x4F
}

pub fn is_valid_jp2(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    &data[0..4] == &[0x00, 0x00, 0x00, 0x0C] &&
    &data[4..8] == b"jP  "
}

pub fn extract_j2c_from_jp2(data: &[u8]) -> Option<&[u8]> {
    if !is_valid_jp2(data) {
        if is_valid_j2k(data) {
            return Some(data);
        }
        return None;
    }

    let mut pos = 0;
    while pos + 8 <= data.len() {
        let box_size = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let box_type = &data[pos + 4..pos + 8];

        if box_type == b"jp2c" {
            let content_start = pos + 8;
            let content_end = if box_size == 0 {
                data.len()
            } else if box_size == 1 && pos + 16 <= data.len() {
                let xl_size = u64::from_be_bytes([
                    data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11],
                    data[pos + 12], data[pos + 13], data[pos + 14], data[pos + 15],
                ]) as usize;
                pos + xl_size
            } else {
                pos + box_size
            };

            if content_start < content_end && content_end <= data.len() {
                let codestream = &data[content_start..content_end];
                if is_valid_j2k(codestream) {
                    debug!("Extracted J2C codestream from JP2: {} bytes -> {} bytes",
                           data.len(), codestream.len());
                    return Some(codestream);
                }
            }
        }

        if box_size == 0 {
            break;
        } else if box_size < 8 {
            warn!("Invalid JP2 box size {} at offset {}", box_size, pos);
            break;
        }
        pos += box_size;
    }

    None
}

pub fn ensure_j2c_codestream(data: &[u8]) -> &[u8] {
    if is_valid_j2k(data) {
        return data;
    }
    if is_valid_jp2(data) {
        if let Some(j2c) = extract_j2c_from_jp2(data) {
            return j2c;
        }
    }
    data
}

pub fn detect_texture_format(data: &[u8]) -> TextureFormat {
    if is_valid_j2k(data) {
        TextureFormat::J2K
    } else if is_valid_jp2(data) {
        TextureFormat::JP2
    } else if data.len() >= 8 && &data[0..8] == &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        TextureFormat::PNG
    } else if data.len() >= 3 && &data[0..3] == &[0xFF, 0xD8, 0xFF] {
        TextureFormat::JPEG
    } else if data.len() >= 18 && (data[2] == 2 || data[2] == 10) {
        TextureFormat::TGA
    } else if data.len() >= 4 && &data[0..4] == b"DDS " {
        TextureFormat::DDS
    } else {
        TextureFormat::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    J2K,
    JP2,
    PNG,
    JPEG,
    TGA,
    DDS,
    Unknown,
}

impl TextureFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            TextureFormat::J2K => "image/x-j2c",
            TextureFormat::JP2 => "image/jp2",
            TextureFormat::PNG => "image/png",
            TextureFormat::JPEG => "image/jpeg",
            TextureFormat::TGA => "image/x-targa",
            TextureFormat::DDS => "image/vnd-ms.dds",
            TextureFormat::Unknown => "application/octet-stream",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            TextureFormat::J2K => "j2c",
            TextureFormat::JP2 => "jp2",
            TextureFormat::PNG => "png",
            TextureFormat::JPEG => "jpg",
            TextureFormat::TGA => "tga",
            TextureFormat::DDS => "dds",
            TextureFormat::Unknown => "bin",
        }
    }

    pub fn is_jpeg2000(&self) -> bool {
        matches!(self, TextureFormat::J2K | TextureFormat::JP2)
    }
}

pub fn create_minimal_j2k_texture() -> Vec<u8> {
    vec![
        0xFF, 0x4F,
        0xFF, 0x51, 0x00, 0x29,
        0x00, 0x00,
        0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x03,
        0x07, 0x01, 0x01,
        0x07, 0x01, 0x01,
        0x07, 0x01, 0x01,
        0xFF, 0x52, 0x00, 0x0C,
        0x00,
        0x00, 0x00, 0x01,
        0x01,
        0x00, 0x02, 0x02, 0x00,
        0xFF, 0x90, 0x00, 0x0A,
        0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
        0xFF, 0x93,
        0x00,
        0xFF, 0xD9,
    ]
}

/// Check if a number is a power of 2
pub fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Get next power of two >= n
pub fn next_power_of_two(n: u32) -> u32 {
    if n == 0 { return 1; }
    let mut v = n - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

/// Resize image to power-of-two dimensions suitable for SL
pub fn resize_to_power_of_two(image: &DynamicImage) -> DynamicImage {
    let (width, height) = (image.width(), image.height());

    let new_width = next_power_of_two(width).min(1024);
    let new_height = next_power_of_two(height).min(1024);

    if new_width != width || new_height != height {
        image.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        image.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_j2k_marker_detection() {
        let valid_j2k = vec![0xFF, 0x4F, 0x00, 0x00];
        let invalid_j2k = vec![0x89, 0x50, 0x4E, 0x47];

        assert!(is_valid_j2k(&valid_j2k));
        assert!(!is_valid_j2k(&invalid_j2k));
    }

    #[test]
    fn test_jp2_marker_detection() {
        let valid_jp2 = vec![0x00, 0x00, 0x00, 0x0C, b'j', b'P', b' ', b' ', 0x0D, 0x0A, 0x87, 0x0A];
        assert!(is_valid_jp2(&valid_jp2));
    }

    #[test]
    fn test_texture_format_detection() {
        assert_eq!(detect_texture_format(&[0xFF, 0x4F]), TextureFormat::J2K);
        assert_eq!(detect_texture_format(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]), TextureFormat::PNG);
        assert_eq!(detect_texture_format(&[0xFF, 0xD8, 0xFF]), TextureFormat::JPEG);
        assert_eq!(detect_texture_format(&[b'D', b'D', b'S', b' ']), TextureFormat::DDS);
    }

    #[test]
    fn test_decoded_image_solid_color() {
        let image = DecodedImage::solid_color(4, 4, 255, 0, 0, 255);
        assert_eq!(image.width, 4);
        assert_eq!(image.height, 4);
        assert_eq!(image.pixels.len(), 64);

        let pixel = image.get_pixel(0, 0).unwrap();
        assert_eq!(pixel, [255, 0, 0, 255]);
    }

    #[test]
    fn test_decoded_image_pixel_access() {
        let mut image = DecodedImage::solid_color(4, 4, 0, 0, 0, 255);
        image.set_pixel(1, 1, [255, 128, 64, 255]);

        let pixel = image.get_pixel(1, 1).unwrap();
        assert_eq!(pixel, [255, 128, 64, 255]);

        assert!(image.get_pixel(10, 10).is_none());
    }

    #[test]
    fn test_codec_creation() {
        let codec = J2KCodec::new().with_reduction(2);
        assert_eq!(codec.reduction_factor, 2);
    }

    #[test]
    fn test_texture_format_mime() {
        assert_eq!(TextureFormat::J2K.mime_type(), "image/x-j2c");
        assert_eq!(TextureFormat::PNG.mime_type(), "image/png");
    }

    #[test]
    fn test_minimal_j2k_is_valid() {
        let j2k = create_minimal_j2k_texture();
        assert!(is_valid_j2k(&j2k));
    }

    #[test]
    fn test_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(256));
        assert!(is_power_of_two(512));
        assert!(is_power_of_two(1024));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(100));
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(100), 128);
        assert_eq!(next_power_of_two(512), 512);
        assert_eq!(next_power_of_two(513), 1024);
    }

    #[test]
    fn test_opj_compress_available() {
        let available = check_opj_compress_available();
        println!("opj_compress available: {}", available);
    }

    #[test]
    fn test_encoder_creation() {
        let encoder = J2KEncoder::new();
        assert_eq!(encoder.num_resolutions, 6);
        assert!(!encoder.lossless);

        let encoder = J2KEncoder::new().lossless();
        assert!(encoder.lossless);

        let encoder = J2KEncoder::new().with_quality(80);
        assert_eq!(encoder.quality, Some(80));
    }

    #[test]
    fn test_encode_solid_color_image() {
        if !check_opj_compress_available() {
            println!("Skipping encode test - opj_compress not available");
            return;
        }

        let image = DecodedImage::solid_color(64, 64, 255, 0, 0, 255);
        let j2k_data = image.to_j2k();

        assert!(j2k_data.is_ok(), "Failed to encode: {:?}", j2k_data.err());
        let j2k_data = j2k_data.unwrap();

        assert!(is_valid_j2k(&j2k_data), "Output is not valid J2K");
        assert!(j2k_data.len() > 0, "J2K output is empty");

        println!("Encoded 64x64 red image to {} bytes J2K", j2k_data.len());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        if !check_opj_compress_available() {
            println!("Skipping roundtrip test - opj_compress not available");
            return;
        }

        let original = DecodedImage::solid_color(32, 32, 128, 64, 192, 255);
        let j2k_data = original.to_j2k_lossless().expect("Failed to encode");

        let codec = J2KCodec::new();
        let decoded = codec.decode_j2k(&j2k_data).expect("Failed to decode");

        assert_eq!(decoded.width, 32);
        assert_eq!(decoded.height, 32);
        println!("Roundtrip successful: 32x32 -> {} bytes J2K -> decoded", j2k_data.len());
    }
}
