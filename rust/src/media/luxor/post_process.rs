use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostEffect {
    Vignette { strength: f32 },
    Bloom { threshold: f32, radius: u32 },
    Letterbox { ratio: f32 },
    FilmGrain { intensity: f32 },
    ColorGradeWarm,
    ColorGradeCool,
    ColorGradeNoir,
    ToneMapAces,
    ToneMapReinhard,
    Sharpen { strength: f32 },
    ChromaticAberration { offset: f32 },
    DepthFog { density: f32, color: [f32; 3] },
    TiltShift { focus_y: f32, blur_radius: f32 },
}

impl PostEffect {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "vignette" => Some(PostEffect::Vignette { strength: 0.6 }),
            "bloom" => Some(PostEffect::Bloom { threshold: 0.8, radius: 5 }),
            "letterbox" | "cinematic" => Some(PostEffect::Letterbox { ratio: 2.39 }),
            "film_grain" | "grain" => Some(PostEffect::FilmGrain { intensity: 0.08 }),
            "warm" | "color_grade_warm" | "golden" => Some(PostEffect::ColorGradeWarm),
            "cool" | "color_grade_cool" | "blue" => Some(PostEffect::ColorGradeCool),
            "noir" | "color_grade_noir" | "bw" => Some(PostEffect::ColorGradeNoir),
            "aces" | "tone_map_aces" => Some(PostEffect::ToneMapAces),
            "reinhard" | "tone_map_reinhard" => Some(PostEffect::ToneMapReinhard),
            "sharpen" | "sharp" => Some(PostEffect::Sharpen { strength: 0.5 }),
            "chromatic" | "chromatic_aberration" | "ca" => Some(PostEffect::ChromaticAberration { offset: 3.0 }),
            "fog" | "depth_fog" => Some(PostEffect::DepthFog { density: 0.02, color: [0.7, 0.75, 0.85] }),
            "tilt_shift" | "miniature" => Some(PostEffect::TiltShift { focus_y: 0.5, blur_radius: 3.0 }),
            _ => None,
        }
    }
}

pub fn apply_effect(pixels: &mut Vec<u8>, effect: &PostEffect, width: u32, height: u32) {
    match effect {
        PostEffect::Vignette { strength } => apply_vignette(pixels, width, height, *strength),
        PostEffect::Bloom { threshold, radius } => apply_bloom(pixels, width, height, *threshold, *radius),
        PostEffect::Letterbox { ratio } => apply_letterbox(pixels, width, height, *ratio),
        PostEffect::FilmGrain { intensity } => apply_film_grain(pixels, width, height, *intensity),
        PostEffect::ColorGradeWarm => apply_color_grade_warm(pixels, width, height),
        PostEffect::ColorGradeCool => apply_color_grade_cool(pixels, width, height),
        PostEffect::ColorGradeNoir => apply_color_grade_noir(pixels, width, height),
        PostEffect::ToneMapAces => apply_tone_map_aces(pixels, width, height),
        PostEffect::ToneMapReinhard => apply_tone_map_reinhard(pixels, width, height),
        PostEffect::Sharpen { strength } => apply_sharpen(pixels, width, height, *strength),
        PostEffect::ChromaticAberration { offset } => apply_chromatic_aberration(pixels, width, height, *offset),
        PostEffect::DepthFog { density, color } => apply_depth_fog(pixels, width, height, *density, *color),
        PostEffect::TiltShift { focus_y, blur_radius } => apply_tilt_shift(pixels, width, height, *focus_y, *blur_radius),
    }
}

fn apply_vignette(pixels: &mut [u8], width: u32, height: u32, strength: f32) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let factor = 1.0 - (dist * dist * strength).min(1.0);

            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = (pixels[idx] as f32 * factor) as u8;
            pixels[idx + 1] = (pixels[idx + 1] as f32 * factor) as u8;
            pixels[idx + 2] = (pixels[idx + 2] as f32 * factor) as u8;
        }
    }
}

fn apply_bloom(pixels: &mut [u8], width: u32, height: u32, threshold: f32, radius: u32) {
    let w = width as usize;
    let h = height as usize;
    let threshold_byte = (threshold * 255.0) as u8;

    let mut bright = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let r = pixels[idx] as f32 / 255.0;
            let g = pixels[idx + 1] as f32 / 255.0;
            let b = pixels[idx + 2] as f32 / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            if lum > threshold {
                let excess = lum - threshold;
                let bidx = (y * w + x) * 3;
                bright[bidx] = r * excess;
                bright[bidx + 1] = g * excess;
                bright[bidx + 2] = b * excess;
            }
        }
    }

    let kernel_size = (radius * 2 + 1) as usize;
    let sigma = radius as f32 * 0.5;
    let mut kernel = vec![0.0f32; kernel_size];
    let mut sum = 0.0f32;
    for i in 0..kernel_size {
        let x = i as f32 - radius as f32;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
        sum += kernel[i];
    }
    for k in &mut kernel { *k /= sum; }

    let mut temp = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
            for k in 0..kernel_size {
                let sx = (x as i32 + k as i32 - radius as i32).clamp(0, w as i32 - 1) as usize;
                let bidx = (y * w + sx) * 3;
                r += bright[bidx] * kernel[k];
                g += bright[bidx + 1] * kernel[k];
                b += bright[bidx + 2] * kernel[k];
            }
            let tidx = (y * w + x) * 3;
            temp[tidx] = r;
            temp[tidx + 1] = g;
            temp[tidx + 2] = b;
        }
    }

    let mut blurred = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let mut r = 0.0f32; let mut g = 0.0f32; let mut b = 0.0f32;
            for k in 0..kernel_size {
                let sy = (y as i32 + k as i32 - radius as i32).clamp(0, h as i32 - 1) as usize;
                let tidx = (sy * w + x) * 3;
                r += temp[tidx] * kernel[k];
                g += temp[tidx + 1] * kernel[k];
                b += temp[tidx + 2] * kernel[k];
            }
            let bidx = (y * w + x) * 3;
            blurred[bidx] = r;
            blurred[bidx + 1] = g;
            blurred[bidx + 2] = b;
        }
    }

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let bidx = (y * w + x) * 3;
            pixels[idx] = ((pixels[idx] as f32 / 255.0 + blurred[bidx]).min(1.0) * 255.0) as u8;
            pixels[idx + 1] = ((pixels[idx + 1] as f32 / 255.0 + blurred[bidx + 1]).min(1.0) * 255.0) as u8;
            pixels[idx + 2] = ((pixels[idx + 2] as f32 / 255.0 + blurred[bidx + 2]).min(1.0) * 255.0) as u8;
        }
    }
}

fn apply_letterbox(pixels: &mut [u8], width: u32, height: u32, target_ratio: f32) {
    let current_ratio = width as f32 / height as f32;
    if current_ratio >= target_ratio {
        return;
    }

    let visible_height = (width as f32 / target_ratio) as u32;
    let bar_height = (height.saturating_sub(visible_height)) / 2;

    for y in 0..bar_height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = 0;
            pixels[idx + 1] = 0;
            pixels[idx + 2] = 0;
        }
    }

    for y in (height - bar_height)..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = 0;
            pixels[idx + 1] = 0;
            pixels[idx + 2] = 0;
        }
    }
}

fn apply_film_grain(pixels: &mut [u8], width: u32, height: u32, intensity: f32) {
    let mut state = 42u32;
    for y in 0..height {
        for x in 0..width {
            state = xorshift32(state);
            let noise = ((state as f32 / u32::MAX as f32) - 0.5) * 2.0 * intensity * 255.0;

            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = (pixels[idx] as f32 + noise).clamp(0.0, 255.0) as u8;
            pixels[idx + 1] = (pixels[idx + 1] as f32 + noise).clamp(0.0, 255.0) as u8;
            pixels[idx + 2] = (pixels[idx + 2] as f32 + noise).clamp(0.0, 255.0) as u8;
        }
    }
}

fn apply_color_grade_warm(pixels: &mut [u8], width: u32, height: u32) {
    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as f32 / 255.0;
        let g = pixels[i + 1] as f32 / 255.0;
        let b = pixels[i + 2] as f32 / 255.0;

        pixels[i] = ((r * 1.1 + 0.05).min(1.0) * 255.0) as u8;
        pixels[i + 1] = ((g * 1.02 + 0.02).min(1.0) * 255.0) as u8;
        pixels[i + 2] = ((b * 0.85).min(1.0) * 255.0) as u8;
    }
}

fn apply_color_grade_cool(pixels: &mut [u8], width: u32, height: u32) {
    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as f32 / 255.0;
        let g = pixels[i + 1] as f32 / 255.0;
        let b = pixels[i + 2] as f32 / 255.0;

        pixels[i] = ((r * 0.85).min(1.0) * 255.0) as u8;
        pixels[i + 1] = ((g * 0.95 + 0.02).min(1.0) * 255.0) as u8;
        pixels[i + 2] = ((b * 1.1 + 0.05).min(1.0) * 255.0) as u8;
    }
}

fn apply_color_grade_noir(pixels: &mut [u8], width: u32, height: u32) {
    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as f32 / 255.0;
        let g = pixels[i + 1] as f32 / 255.0;
        let b = pixels[i + 2] as f32 / 255.0;

        let lum = 0.299 * r + 0.587 * g + 0.114 * b;

        let contrast = ((lum - 0.5) * 1.5 + 0.5).clamp(0.0, 1.0);

        let final_val = (contrast * 255.0) as u8;
        pixels[i] = final_val;
        pixels[i + 1] = final_val;
        pixels[i + 2] = final_val;
    }
}

fn apply_tone_map_aces(pixels: &mut [u8], width: u32, height: u32) {
    for i in (0..pixels.len()).step_by(4) {
        for c in 0..3 {
            let x = pixels[i + c] as f32 / 255.0;
            let mapped = (x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14);
            pixels[i + c] = (mapped.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
}

fn apply_tone_map_reinhard(pixels: &mut [u8], width: u32, height: u32) {
    for i in (0..pixels.len()).step_by(4) {
        for c in 0..3 {
            let x = pixels[i + c] as f32 / 255.0;
            let mapped = x / (1.0 + x);
            pixels[i + c] = (mapped.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
}

fn apply_sharpen(pixels: &mut [u8], width: u32, height: u32, strength: f32) {
    let w = width as usize;
    let h = height as usize;
    let original = pixels.to_vec();

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            for c in 0..3 {
                let idx = |px: usize, py: usize| (py * w + px) * 4 + c;

                let center = original[idx(x, y)] as f32;
                let neighbors =
                    original[idx(x - 1, y)] as f32 +
                    original[idx(x + 1, y)] as f32 +
                    original[idx(x, y - 1)] as f32 +
                    original[idx(x, y + 1)] as f32;

                let detail = center - neighbors * 0.25;
                let sharpened = center + detail * strength;
                pixels[idx(x, y)] = sharpened.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn apply_chromatic_aberration(pixels: &mut [u8], width: u32, height: u32, offset: f32) {
    let w = width as usize;
    let h = height as usize;
    let original = pixels.to_vec();
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let shift = (dist * offset) as i32;

            let idx = (y * w + x) * 4;

            let rx = (x as i32 - shift).clamp(0, w as i32 - 1) as usize;
            pixels[idx] = original[(y * w + rx) * 4];

            let bx = (x as i32 + shift).clamp(0, w as i32 - 1) as usize;
            pixels[idx + 2] = original[(y * w + bx) * 4 + 2];
        }
    }
}

fn apply_depth_fog(pixels: &mut [u8], width: u32, height: u32, density: f32, fog_color: [f32; 3]) {
    for y in 0..height {
        let depth_factor = 1.0 - (y as f32 / height as f32);
        let fog_amount = (1.0 - (-density * depth_factor * 100.0).exp()).clamp(0.0, 0.8);

        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            for c in 0..3 {
                let original = pixels[idx + c] as f32 / 255.0;
                let fogged = original * (1.0 - fog_amount) + fog_color[c] * fog_amount;
                pixels[idx + c] = (fogged.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }
}

fn apply_tilt_shift(pixels: &mut [u8], width: u32, height: u32, focus_y: f32, blur_radius: f32) {
    let w = width as usize;
    let h = height as usize;
    let original = pixels.to_vec();
    let focus_row = (focus_y * h as f32) as usize;

    for y in 0..h {
        let dist_from_focus = (y as f32 - focus_row as f32).abs() / h as f32;
        let blur_amount = (dist_from_focus * blur_radius * 2.0).min(blur_radius);
        let kernel_half = blur_amount.ceil() as i32;

        if kernel_half <= 0 { continue; }

        for x in 0..w {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut count = 0.0f32;

            for ky in -kernel_half..=kernel_half {
                for kx in -kernel_half..=kernel_half {
                    let sy = (y as i32 + ky).clamp(0, h as i32 - 1) as usize;
                    let sx = (x as i32 + kx).clamp(0, w as i32 - 1) as usize;
                    let sidx = (sy * w + sx) * 4;
                    r += original[sidx] as f32;
                    g += original[sidx + 1] as f32;
                    b += original[sidx + 2] as f32;
                    count += 1.0;
                }
            }

            let idx = (y * w + x) * 4;
            pixels[idx] = (r / count) as u8;
            pixels[idx + 1] = (g / count) as u8;
            pixels[idx + 2] = (b / count) as u8;
        }
    }
}

fn xorshift32(mut state: u32) -> u32 {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(width: u32, height: u32) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 200.0) as u8 + 50;
                let g = ((y as f32 / height as f32) * 200.0) as u8 + 50;
                let b = 128;
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255);
            }
        }
        pixels
    }

    #[test]
    fn test_vignette() {
        let mut pixels = make_test_image(64, 48);
        let corner_before = pixels[0];
        let center_idx = (24 * 64 + 32) * 4;
        let center_before = pixels[center_idx];

        apply_vignette(&mut pixels, 64, 48, 0.6);

        assert!(pixels[0] < corner_before, "Corner should be darker");
        assert!((pixels[center_idx] as i32 - center_before as i32).abs() < 20,
            "Center should be relatively unchanged");
    }

    #[test]
    fn test_letterbox() {
        let mut pixels = make_test_image(64, 48);
        apply_letterbox(&mut pixels, 64, 48, 2.39);

        assert_eq!(pixels[0], 0, "Top bar should be black");
        assert_eq!(pixels[1], 0);
        assert_eq!(pixels[2], 0);
    }

    #[test]
    fn test_film_grain() {
        let mut pixels = make_test_image(32, 24);
        let original = pixels.clone();
        apply_film_grain(&mut pixels, 32, 24, 0.1);

        let mut diffs = 0;
        for i in 0..pixels.len() {
            if pixels[i] != original[i] { diffs += 1; }
        }
        assert!(diffs > 0, "Film grain should modify pixels");
    }

    #[test]
    fn test_noir_greyscale() {
        let mut pixels = vec![200, 50, 50, 255, 50, 200, 50, 255];
        apply_color_grade_noir(&mut pixels, 2, 1);

        assert_eq!(pixels[0], pixels[1], "Noir should produce greyscale");
        assert_eq!(pixels[1], pixels[2]);
    }

    #[test]
    fn test_warm_grade() {
        let mut pixels = vec![128, 128, 128, 255];
        apply_color_grade_warm(&mut pixels, 1, 1);

        assert!(pixels[0] > 128, "Warm should boost red");
        assert!(pixels[2] < 128, "Warm should reduce blue");
    }

    #[test]
    fn test_cool_grade() {
        let mut pixels = vec![128, 128, 128, 255];
        apply_color_grade_cool(&mut pixels, 1, 1);

        assert!(pixels[0] < 128, "Cool should reduce red");
        assert!(pixels[2] > 128, "Cool should boost blue");
    }

    #[test]
    fn test_all_effects_parse() {
        let names = [
            "vignette", "bloom", "letterbox", "grain", "warm", "cool", "noir",
            "aces", "reinhard", "sharpen", "chromatic", "fog", "tilt_shift",
        ];
        for name in &names {
            assert!(PostEffect::from_name(name).is_some(), "Effect '{}' should parse", name);
        }
    }

    #[test]
    fn test_aces_tone_map() {
        let mut pixels = vec![200, 200, 200, 255];
        apply_tone_map_aces(&mut pixels, 1, 1);
        assert!(pixels[0] < 200, "ACES should compress bright values");
    }

    #[test]
    fn test_chromatic_aberration() {
        let mut pixels = make_test_image(64, 48);
        let original = pixels.clone();
        apply_chromatic_aberration(&mut pixels, 64, 48, 3.0);

        let mut diffs = 0;
        for i in 0..pixels.len() {
            if pixels[i] != original[i] { diffs += 1; }
        }
        assert!(diffs > 0, "CA should shift channels at edges");
    }
}
