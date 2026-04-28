use anyhow::{bail, Result};
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlacementSide {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyRegion {
    Upper,
    Lower,
}

#[derive(Debug, Clone)]
pub struct LogoPlacement {
    pub side: PlacementSide,
    pub offset_from_collar_inches: f32,
    pub centered: bool,
    pub body_region: BodyRegion,
}

#[derive(Debug, Clone)]
pub struct TShirtTextureConfig {
    pub base_color: [u8; 4],
    pub logo_path: String,
    pub front_placement: Option<LogoPlacement>,
    pub back_placement: Option<LogoPlacement>,
    pub texture_size: u32,
}

const SHIRT_HEIGHT_INCHES: f32 = 24.0;
const PRINTABLE_W_INCHES: f32 = 10.0;
const PRINTABLE_H_INCHES: f32 = 15.0;

const SL_FRONT_U_MIN: f32 = 0.02;
const SL_FRONT_U_MAX: f32 = 0.48;
const SL_FRONT_V_MIN: f32 = 0.02;
const SL_FRONT_V_MAX: f32 = 0.52;

const SL_BACK_U_MIN: f32 = 0.52;
const SL_BACK_U_MAX: f32 = 0.98;
const SL_BACK_V_MIN: f32 = 0.02;
const SL_BACK_V_MAX: f32 = 0.52;

const SL_FRONT_COLLAR_V: f32 = 0.04;
const SL_FRONT_HEM_V: f32 = 0.48;

const SL_LOWER_FRONT_U_MIN: f32 = 0.02;
const SL_LOWER_FRONT_U_MAX: f32 = 0.48;
const SL_LOWER_FRONT_V_MIN: f32 = 0.02;
const SL_LOWER_FRONT_V_MAX: f32 = 0.98;

const SL_LOWER_BACK_U_MIN: f32 = 0.52;
const SL_LOWER_BACK_U_MAX: f32 = 0.98;
const SL_LOWER_BACK_V_MIN: f32 = 0.02;
const SL_LOWER_BACK_V_MAX: f32 = 0.98;

const SL_LOWER_WAIST_V: f32 = 0.04;
const SL_LOWER_ANKLE_V: f32 = 0.94;
const PANTS_HEIGHT_INCHES: f32 = 40.0;

impl Default for TShirtTextureConfig {
    fn default() -> Self {
        Self {
            base_color: [255, 255, 255, 255],
            logo_path: String::new(),
            front_placement: Some(LogoPlacement {
                side: PlacementSide::Front,
                offset_from_collar_inches: 2.0,
                centered: true,
                body_region: BodyRegion::Upper,
            }),
            back_placement: None,
            texture_size: 1024,
        }
    }
}

pub fn compose_tshirt_texture(config: &TShirtTextureConfig) -> Result<DynamicImage> {
    let sz = config.texture_size;
    let mut canvas = RgbaImage::from_pixel(sz, sz, Rgba(config.base_color));

    if config.logo_path.is_empty() {
        return Ok(DynamicImage::ImageRgba8(canvas));
    }

    let logo = image::open(&config.logo_path)
        .map_err(|e| anyhow::anyhow!("Failed to load logo '{}': {}", config.logo_path, e))?;

    let front_w = SL_FRONT_U_MAX - SL_FRONT_U_MIN;
    let front_h = SL_FRONT_HEM_V - SL_FRONT_COLLAR_V;
    let printable_u_fraction = (PRINTABLE_W_INCHES / 17.0).min(0.80);
    let printable_v_fraction = (PRINTABLE_H_INCHES / SHIRT_HEIGHT_INCHES).min(0.70);
    let max_logo_w = (printable_u_fraction * front_w * sz as f32) as u32;
    let max_logo_h = (printable_v_fraction * front_h * sz as f32) as u32;

    let (orig_w, orig_h) = logo.dimensions();
    if orig_w == 0 || orig_h == 0 {
        bail!("Logo image has zero dimensions");
    }
    let aspect = orig_w as f32 / orig_h as f32;
    let (scaled_w, scaled_h) = if (max_logo_w as f32 / aspect) <= max_logo_h as f32 {
        (max_logo_w, (max_logo_w as f32 / aspect) as u32)
    } else {
        ((max_logo_h as f32 * aspect) as u32, max_logo_h)
    };
    let scaled_logo = logo.resize_exact(scaled_w, scaled_h, image::imageops::FilterType::Lanczos3);

    if let Some(ref placement) = config.front_placement {
        place_logo_sl_uv(&mut canvas, &scaled_logo, placement, sz)?;
    }

    if let Some(ref placement) = config.back_placement {
        place_logo_sl_uv(&mut canvas, &scaled_logo, placement, sz)?;
    }

    Ok(DynamicImage::ImageRgba8(canvas))
}

fn place_logo_sl_uv(
    canvas: &mut RgbaImage,
    logo: &DynamicImage,
    placement: &LogoPlacement,
    texture_size: u32,
) -> Result<()> {
    let sz = texture_size as f32;
    let (logo_w, logo_h) = logo.dimensions();

    let (quad_u_min, quad_u_max, collar_v, hem_v, height_inches) =
        match (placement.body_region, placement.side) {
            (BodyRegion::Upper, PlacementSide::Front) => (
                SL_FRONT_U_MIN,
                SL_FRONT_U_MAX,
                SL_FRONT_COLLAR_V,
                SL_FRONT_HEM_V,
                SHIRT_HEIGHT_INCHES,
            ),
            (BodyRegion::Upper, PlacementSide::Back) => (
                SL_BACK_U_MIN,
                SL_BACK_U_MAX,
                SL_BACK_V_MIN + 0.02,
                SL_BACK_V_MAX - 0.04,
                SHIRT_HEIGHT_INCHES,
            ),
            (BodyRegion::Lower, PlacementSide::Front) => (
                SL_LOWER_FRONT_U_MIN,
                SL_LOWER_FRONT_U_MAX,
                SL_LOWER_WAIST_V,
                SL_LOWER_ANKLE_V,
                PANTS_HEIGHT_INCHES,
            ),
            (BodyRegion::Lower, PlacementSide::Back) => (
                SL_LOWER_BACK_U_MIN,
                SL_LOWER_BACK_U_MAX,
                SL_LOWER_WAIST_V + 0.02,
                SL_LOWER_ANKLE_V - 0.04,
                PANTS_HEIGHT_INCHES,
            ),
        };

    let quad_center_u = (quad_u_min + quad_u_max) / 2.0;
    let collar_to_hem_v = hem_v - collar_v;
    let offset_v_fraction = placement.offset_from_collar_inches / height_inches;
    let logo_top_v = collar_v + offset_v_fraction * collar_to_hem_v;

    let pixel_center_x = (quad_center_u * sz) as i64;
    let pixel_top_y = (logo_top_v * sz) as i64;

    let pixel_x_start = pixel_center_x - (logo_w as i64 / 2);

    let logo_rgba = logo.to_rgba8();
    for dy in 0..logo_h {
        for dx in 0..logo_w {
            let target_x = pixel_x_start + dx as i64;
            let target_y = pixel_top_y + dy as i64;

            if target_x < 0
                || target_x >= texture_size as i64
                || target_y < 0
                || target_y >= texture_size as i64
            {
                continue;
            }

            let src = logo_rgba.get_pixel(dx, dy);
            if src[3] == 0 {
                continue;
            }

            let dst = canvas.get_pixel(target_x as u32, target_y as u32);
            let alpha = src[3] as f32 / 255.0;
            let inv = 1.0 - alpha;
            let blended = Rgba([
                (src[0] as f32 * alpha + dst[0] as f32 * inv) as u8,
                (src[1] as f32 * alpha + dst[1] as f32 * inv) as u8,
                (src[2] as f32 * alpha + dst[2] as f32 * inv) as u8,
                255,
            ]);
            canvas.put_pixel(target_x as u32, target_y as u32, blended);
        }
    }
    Ok(())
}

pub fn load_ruth2_uv_map(region: &str) -> Result<DynamicImage> {
    let path = super::blender_worker::ruth2_uv_path(region)
        .ok_or_else(|| anyhow::anyhow!("Ruth2_v4 UV map not found for region '{}'", region))?;
    let img = image::open(&path)
        .map_err(|e| anyhow::anyhow!("Failed to load UV map '{}': {}", path, e))?;
    Ok(img)
}

pub fn load_sl_avatar_texture(region: &str) -> Result<DynamicImage> {
    let path = super::blender_worker::ruth2_texture_path(region)
        .ok_or_else(|| anyhow::anyhow!("SL avatar texture not found for region '{}'", region))?;
    let img = image::open(&path)
        .map_err(|e| anyhow::anyhow!("Failed to load SL texture '{}': {}", path, e))?;
    Ok(img)
}

pub fn compose_garment_texture_with_uv(
    garment_type: &str,
    base_color: [u8; 4],
    logo: Option<&DynamicImage>,
    logo_placement: Option<&LogoPlacement>,
) -> Result<DynamicImage> {
    let uv_region = match garment_type {
        "shirt" | "jacket" | "dress" => "upper",
        "pants" | "skirt" => "lower",
        _ => "upper",
    };

    let uv_map = load_ruth2_uv_map(uv_region)?;
    let uv_rgba = uv_map.to_rgba8();

    let mut canvas = RgbaImage::from_pixel(1024, 1024, Rgba(base_color));

    if let (Some(logo_img), Some(placement)) = (logo, logo_placement) {
        let (logo_w, logo_h) = logo_img.dimensions();
        if logo_w > 0 && logo_h > 0 {
            let (u_min, u_max, v_min, v_max) = match placement.side {
                PlacementSide::Front => (
                    SL_FRONT_U_MIN,
                    SL_FRONT_U_MAX,
                    SL_FRONT_V_MIN,
                    SL_FRONT_V_MAX,
                ),
                PlacementSide::Back => (SL_BACK_U_MIN, SL_BACK_U_MAX, SL_BACK_V_MIN, SL_BACK_V_MAX),
            };
            let island_w = ((u_max - u_min) * 1024.0) as u32;
            let island_h = ((v_max - v_min) * 1024.0) as u32;
            let max_logo_w = (island_w as f32 * 0.75) as u32;
            let max_logo_h = (island_h as f32 * 0.60) as u32;
            let aspect = logo_w as f32 / logo_h as f32;
            let (scaled_w, scaled_h) = if (max_logo_w as f32 / aspect) <= max_logo_h as f32 {
                (max_logo_w, (max_logo_w as f32 / aspect) as u32)
            } else {
                ((max_logo_h as f32 * aspect) as u32, max_logo_h)
            };
            let scaled =
                logo_img.resize_exact(scaled_w, scaled_h, image::imageops::FilterType::Lanczos3);
            place_logo_sl_uv(&mut canvas, &scaled, placement, 1024)?;
        }
    }

    apply_uv_mask(&mut canvas, &uv_rgba, base_color);

    Ok(DynamicImage::ImageRgba8(canvas))
}

fn apply_uv_mask(canvas: &mut RgbaImage, uv_map: &RgbaImage, base_color: [u8; 4]) {
    let (w, h) = canvas.dimensions();
    let (uv_w, uv_h) = uv_map.dimensions();
    if uv_w != w || uv_h != h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let uv_pixel = uv_map.get_pixel(x, y);
            if uv_pixel[3] == 0 {
                canvas.put_pixel(x, y, Rgba([base_color[0], base_color[1], base_color[2], 0]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = TShirtTextureConfig::default();
        assert_eq!(cfg.base_color, [255, 255, 255, 255]);
        assert_eq!(cfg.texture_size, 1024);
        assert!(cfg.front_placement.is_some());
        assert!(cfg.back_placement.is_none());
    }

    #[test]
    fn test_compose_solid_color_no_logo() {
        let cfg = TShirtTextureConfig {
            base_color: [200, 50, 50, 255],
            logo_path: String::new(),
            front_placement: None,
            back_placement: None,
            texture_size: 64,
        };
        let img = compose_tshirt_texture(&cfg).unwrap();
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
        let pixel = img.get_pixel(32, 32);
        assert_eq!(pixel[0], 200);
        assert_eq!(pixel[1], 50);
    }

    #[test]
    fn test_compose_missing_logo_fails() {
        let cfg = TShirtTextureConfig {
            logo_path: "/nonexistent/logo.png".to_string(),
            front_placement: Some(LogoPlacement {
                side: PlacementSide::Front,
                offset_from_collar_inches: 2.0,
                centered: true,
                body_region: BodyRegion::Upper,
            }),
            ..Default::default()
        };
        assert!(compose_tshirt_texture(&cfg).is_err());
    }

    #[test]
    fn test_load_ruth2_uv_upper() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = load_ruth2_uv_map("upper").expect("Upper UV map load failed");
        assert_eq!(img.width(), 1024);
        assert_eq!(img.height(), 1024);
    }

    #[test]
    fn test_load_ruth2_uv_lower() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = load_ruth2_uv_map("lower").expect("Lower UV map load failed");
        assert_eq!(img.width(), 1024);
        assert_eq!(img.height(), 1024);
    }

    #[test]
    fn test_load_sl_avatar_texture() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = load_sl_avatar_texture("upper").expect("SL Upper texture load failed");
        assert_eq!(img.width(), 1024);
        assert_eq!(img.height(), 1024);
    }

    #[test]
    fn test_load_ruth2_uv_unknown_fails() {
        assert!(load_ruth2_uv_map("nonexistent").is_err());
    }

    #[test]
    fn test_load_sl_avatar_texture_unknown_fails() {
        assert!(load_sl_avatar_texture("nonexistent").is_err());
    }

    #[test]
    fn test_compose_garment_with_uv() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = compose_garment_texture_with_uv("shirt", [100, 150, 200, 255], None, None)
            .expect("Garment texture composition failed");
        assert_eq!(img.width(), 1024);
        assert_eq!(img.height(), 1024);
        let front_pixel = img.get_pixel(256, 256);
        assert_eq!(front_pixel[0], 100);
        assert_eq!(front_pixel[1], 150);
        assert_eq!(front_pixel[2], 200);
    }

    #[test]
    fn test_uv_mask_clears_outside_islands() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = compose_garment_texture_with_uv("shirt", [100, 150, 200, 255], None, None)
            .expect("Garment texture composition failed");
        let corner = img.get_pixel(512, 512);
        assert!(
            corner[3] == 0 || (corner[0] == 100 && corner[1] == 150),
            "Center pixel should be inside UV island (colored) or outside (alpha=0)"
        );
    }

    #[test]
    fn test_uv_logo_scaling_uses_island_size() {
        let front_w = SL_FRONT_U_MAX - SL_FRONT_U_MIN;
        let front_h = SL_FRONT_V_MAX - SL_FRONT_V_MIN;
        let island_w = (front_w * 1024.0) as u32;
        let island_h = (front_h * 1024.0) as u32;
        let max_logo_w = (island_w as f32 * 0.75) as u32;
        let max_logo_h = (island_h as f32 * 0.60) as u32;
        assert!(
            max_logo_w > 300,
            "Front island should allow logos wider than 300px"
        );
        assert!(
            max_logo_h > 200,
            "Front island should allow logos taller than 200px"
        );
        assert!(max_logo_w < 500, "Logo should not exceed island width");
    }

    #[test]
    fn test_placement_math_sl_uv() {
        let front_center_u = (SL_FRONT_U_MIN + SL_FRONT_U_MAX) / 2.0;
        assert!((front_center_u - 0.25).abs() < 0.01);

        let back_center_u = (SL_BACK_U_MIN + SL_BACK_U_MAX) / 2.0;
        assert!((back_center_u - 0.75).abs() < 0.01);

        let collar_v_offset = 2.0 / SHIRT_HEIGHT_INCHES;
        let logo_top_v = SL_FRONT_COLLAR_V + collar_v_offset * (SL_FRONT_HEM_V - SL_FRONT_COLLAR_V);
        assert!(logo_top_v > SL_FRONT_COLLAR_V);
        assert!(logo_top_v < SL_FRONT_HEM_V);
    }

    #[test]
    fn test_lower_body_uv_constants() {
        let lower_front_center_u = (SL_LOWER_FRONT_U_MIN + SL_LOWER_FRONT_U_MAX) / 2.0;
        assert!((lower_front_center_u - 0.25).abs() < 0.01);

        let lower_back_center_u = (SL_LOWER_BACK_U_MIN + SL_LOWER_BACK_U_MAX) / 2.0;
        assert!((lower_back_center_u - 0.75).abs() < 0.01);

        assert!(SL_LOWER_WAIST_V < SL_LOWER_ANKLE_V);
        assert!(SL_LOWER_ANKLE_V <= SL_LOWER_FRONT_V_MAX);
        assert!(PANTS_HEIGHT_INCHES > SHIRT_HEIGHT_INCHES);
    }

    #[test]
    fn test_compose_garment_pants_with_uv() {
        if crate::mesh::blender_worker::ruth2_base_dir().is_none() {
            return;
        }
        let img = compose_garment_texture_with_uv("pants", [50, 50, 128, 255], None, None)
            .expect("Pants texture composition failed");
        assert_eq!(img.width(), 1024);
        assert_eq!(img.height(), 1024);
    }
}
