use super::managed_image::{ImageChannels, ManagedImage};
use image::{DynamicImage, GenericImageView};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::OnceLock;
use tracing::{debug, error, warn};

static STATIC_ASSETS: OnceLock<HashMap<String, Vec<u8>>> = OnceLock::new();

pub struct StaticAssets;

impl StaticAssets {
    pub fn initialize() {
        STATIC_ASSETS.get_or_init(|| {
            let mut assets = HashMap::new();

            assets.insert("head_color.tga".to_string(), Self::create_default_skin_texture(1024, 1024));
            assets.insert("head_alpha.tga".to_string(), Self::create_default_alpha_texture(1024, 1024));
            assets.insert("head_skingrain.tga".to_string(), Self::create_skin_grain_texture(1024, 1024));
            assets.insert("head_hair.tga".to_string(), Self::create_default_hair_texture(1024, 1024));
            assets.insert("upperbody_color.tga".to_string(), Self::create_default_skin_texture(1024, 1024));
            assets.insert("lowerbody_color.tga".to_string(), Self::create_default_skin_texture(1024, 1024));

            debug!("Initialized {} static baking assets", assets.len());
            assets
        });
    }

    pub fn load_resource(filename: &str) -> Option<ManagedImage> {
        Self::initialize();

        let assets = STATIC_ASSETS.get()?;
        let data = assets.get(filename)?;

        ManagedImage::from_bytes(data).ok()
    }

    fn create_default_skin_texture(width: u32, height: u32) -> Vec<u8> {
        let mut img = image::RgbaImage::new(width, height);

        let skin_r = 215u8;
        let skin_g = 180u8;
        let skin_b = 160u8;

        for y in 0..height {
            for x in 0..width {
                let noise = ((x.wrapping_mul(7) ^ y.wrapping_mul(13)) % 10) as i16 - 5;
                let r = (skin_r as i16 + noise).clamp(0, 255) as u8;
                let g = (skin_g as i16 + noise).clamp(0, 255) as u8;
                let b = (skin_b as i16 + noise).clamp(0, 255) as u8;
                img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
            }
        }

        Self::encode_png(&img)
    }

    fn create_default_alpha_texture(width: u32, height: u32) -> Vec<u8> {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }

        Self::encode_png(&img)
    }

    fn create_skin_grain_texture(width: u32, height: u32) -> Vec<u8> {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let grain = (((x.wrapping_mul(17) ^ y.wrapping_mul(31)) % 20) + 235) as u8;
                img.put_pixel(x, y, image::Rgba([grain, grain, grain, grain]));
            }
        }

        Self::encode_png(&img)
    }

    fn create_default_hair_texture(width: u32, height: u32) -> Vec<u8> {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let alpha = if (y as f32 / height as f32) < 0.3 {
                    ((y as f32 / height as f32 / 0.3) * 255.0) as u8
                } else {
                    255
                };
                img.put_pixel(x, y, image::Rgba([80, 60, 40, alpha]));
            }
        }

        Self::encode_png(&img)
    }

    fn encode_png(img: &image::RgbaImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap_or_default();
        bytes
    }

    pub fn create_default_baked_texture_for_type(bake_type: super::types::BakeType) -> Vec<u8> {
        let (width, height) = bake_type.dimensions();

        let (r, g, b) = match bake_type {
            super::types::BakeType::Head |
            super::types::BakeType::UpperBody |
            super::types::BakeType::LowerBody => (215, 180, 160),
            super::types::BakeType::Eyes => (100, 150, 200),
            super::types::BakeType::Hair => (80, 60, 40),
            super::types::BakeType::Skirt => (128, 128, 128),
            _ => (200, 200, 200),
        };

        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let noise = ((x.wrapping_mul(7) ^ y.wrapping_mul(13)) % 6) as i16 - 3;
                let r_val = (r as i16 + noise).clamp(0, 255) as u8;
                let g_val = (g as i16 + noise).clamp(0, 255) as u8;
                let b_val = (b as i16 + noise).clamp(0, 255) as u8;
                img.put_pixel(x, y, image::Rgba([r_val, g_val, b_val, 255]));
            }
        }

        Self::encode_png(&img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_asset_creation() {
        StaticAssets::initialize();

        let skin = StaticAssets::load_resource("head_color.tga");
        assert!(skin.is_some());

        let skin_img = skin.unwrap();
        assert_eq!(skin_img.width, 1024);
        assert_eq!(skin_img.height, 1024);
    }
}
