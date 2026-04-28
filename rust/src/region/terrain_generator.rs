use noise::{Fbm, NoiseFn, Perlin, RidgedMulti, SuperSimplex, Worley};
use tracing::info;

#[derive(Debug, Clone)]
pub struct TerrainParams {
    pub preset: String,
    pub seed: u32,
    pub scale: f32,
    pub roughness: f32,
    pub water_level: f32,
    pub size: u32,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            preset: "rolling_hills".to_string(),
            seed: 0,
            scale: 1.0,
            roughness: 0.5,
            water_level: 20.0,
            size: 256,
        }
    }
}

pub fn generate(params: &TerrainParams) -> Vec<f32> {
    let seed = if params.seed == 0 {
        rand::random::<u32>()
    } else {
        params.seed
    };

    let size = params.size.max(256);
    let scale = params.scale.max(0.01);
    let roughness = params.roughness.clamp(0.0, 1.0);

    info!(
        "[TERRAIN] Generating '{}' terrain: seed={}, size={}, scale={}, roughness={}",
        params.preset, seed, size, scale, roughness
    );

    let heightmap = match params.preset.as_str() {
        "island" => generate_island(seed, size, scale, roughness, params.water_level),
        "mountains" => generate_mountains(seed, size, scale, roughness),
        "rolling_hills" => generate_rolling_hills(seed, size, scale, roughness),
        "desert" => generate_desert(seed, size, scale, roughness),
        "tropical" => generate_tropical(seed, size, scale, roughness, params.water_level),
        "canyon" => generate_canyon(seed, size, scale, roughness),
        "plateau" => generate_plateau(seed, size, scale, roughness),
        "volcanic" => generate_volcanic(seed, size, scale, roughness),
        _ => {
            info!(
                "[TERRAIN] Unknown preset '{}', falling back to rolling_hills",
                params.preset
            );
            generate_rolling_hills(seed, size, scale, roughness)
        }
    };

    info!(
        "[TERRAIN] Generated {} heightmap values, range {:.1}..{:.1}",
        heightmap.len(),
        heightmap.iter().cloned().fold(f32::INFINITY, f32::min),
        heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
    );

    heightmap
}

fn radial_falloff(x: f32, y: f32, size: f32, sharpness: f32) -> f32 {
    let cx = x / size - 0.5;
    let cy = y / size - 0.5;
    let dist = (cx * cx + cy * cy).sqrt() * 2.0;
    (1.0 - (dist * sharpness).min(1.0)).max(0.0)
}

fn octave_count(roughness: f32) -> usize {
    (2.0 + roughness * 6.0) as usize
}

fn fbm_at(fbm: &Fbm<Perlin>, x: f64, y: f64, freq: f64) -> f64 {
    fbm.get([x * freq, y * freq])
}

fn generate_island(seed: u32, size: u32, scale: f32, roughness: f32, water_level: f32) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.lacunarity = 2.2;
    fbm.persistence = 0.45;

    let warp = Fbm::<Perlin>::new(seed.wrapping_add(100));

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let sz = size as f64;
    let freq = 1.0 / sz;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let wx = warp.get([nx * 3.0, ny * 3.0]) * 0.02;
            let wy = warp.get([nx * 3.0 + 50.0, ny * 3.0 + 50.0]) * 0.02;

            let noise_val = fbm.get([(nx + wx) * 4.0, (ny + wy) * 4.0]);
            let falloff = radial_falloff(x as f32, y as f32, size as f32, 1.3);
            let falloff_curved = falloff * falloff;

            let raw = (noise_val as f32 * 0.5 + 0.5) * falloff_curved;
            let height = water_level - 5.0 + raw * 40.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_mountains(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut ridged = RidgedMulti::<Perlin>::new(seed);
    ridged.octaves = octave_count(roughness);
    ridged.frequency = 1.5;
    ridged.lacunarity = 2.1;

    let mut fbm = Fbm::<Perlin>::new(seed.wrapping_add(42));
    fbm.octaves = 3;
    fbm.frequency = 1.0;

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let ridge_val = ridged.get([nx * 3.0, ny * 3.0]);
            let base_val = fbm.get([nx * 2.0, ny * 2.0]);

            let combined = ridge_val * 0.7 + base_val * 0.3;
            let height = 15.0 + (combined as f32 * 0.5 + 0.5) * 70.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_rolling_hills(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = (2.0 + roughness * 3.0) as usize;
    fbm.frequency = 1.5;
    fbm.lacunarity = 2.0;
    fbm.persistence = 0.5;

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let noise_val = fbm.get([nx * 3.0, ny * 3.0]);
            let height = 20.0 + (noise_val as f32 * 0.5 + 0.5) * 25.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_desert(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.persistence = 0.4;

    let dune_noise = Perlin::new(seed.wrapping_add(77));

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let base = fbm.get([nx * 2.0, ny * 2.0]) as f32;
            let dune_angle = ny * 8.0 + nx * 2.0;
            let dune_val = (dune_angle.sin() * 0.5 + 0.5) as f32;
            let detail = dune_noise.get([nx * 12.0, ny * 12.0]) as f32;

            let combined = base * 0.3 + dune_val * 0.5 + detail * 0.2;
            let height = 18.0 + combined * 15.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_tropical(
    seed: u32,
    size: u32,
    scale: f32,
    roughness: f32,
    water_level: f32,
) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.lacunarity = 2.0;
    fbm.persistence = 0.5;

    let warp = Fbm::<Perlin>::new(seed.wrapping_add(200));

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let wx = warp.get([nx * 4.0, ny * 4.0]) * 0.015;
            let wy = warp.get([nx * 4.0 + 100.0, ny * 4.0 + 100.0]) * 0.015;

            let noise_val = fbm.get([(nx + wx) * 3.5, (ny + wy) * 3.5]);
            let falloff = radial_falloff(x as f32, y as f32, size as f32, 1.1);

            let raw = (noise_val as f32 * 0.5 + 0.5) * falloff;
            let plateau = if raw > 0.45 { raw.min(0.65) } else { raw };
            let height = water_level - 3.0 + plateau * 50.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_canyon(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.persistence = 0.5;

    let mut ridged = RidgedMulti::<Perlin>::new(seed.wrapping_add(33));
    ridged.octaves = 4;
    ridged.frequency = 1.5;

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let base = fbm.get([nx * 3.0, ny * 3.0]) as f32 * 0.5 + 0.5;
            let ridge = ridged.get([nx * 2.5, ny * 2.5]) as f32;
            let canyon_carve = ridge.abs();

            let height = 20.0 + (base - canyon_carve * 0.6) * 50.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_plateau(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.persistence = 0.45;

    let edge_noise = Perlin::new(seed.wrapping_add(55));

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / size as f64;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let base = fbm.get([nx * 3.0, ny * 3.0]) as f32 * 0.5 + 0.5;
            let edge = edge_noise.get([nx * 6.0, ny * 6.0]) as f32 * 0.05;

            let mesa_threshold = 0.45 + edge;
            let clamped = if base > mesa_threshold {
                mesa_threshold + (base - mesa_threshold) * 0.15
            } else {
                base * 0.7
            };

            let height = 15.0 + clamped * 55.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

fn generate_volcanic(seed: u32, size: u32, scale: f32, roughness: f32) -> Vec<f32> {
    let mut ridged = RidgedMulti::<Perlin>::new(seed);
    ridged.octaves = octave_count(roughness);
    ridged.frequency = 2.0;

    let detail = Fbm::<Perlin>::new(seed.wrapping_add(66));
    let flow_noise = Perlin::new(seed.wrapping_add(99));

    let total = (size * size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let sz = size as f32;
    let freq = 1.0 / size as f64;
    let cx = 0.5;
    let cy = 0.5;

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * freq;
            let ny = y as f64 * freq;

            let dx = nx as f32 - cx;
            let dy = ny as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() * 2.0;

            let cone = (1.0 - dist * 1.2).max(0.0);
            let cone_curved = cone * cone * 0.8 + cone * 0.2;

            let crater_depth = if dist < 0.08 {
                -0.15 * (1.0 - dist / 0.08)
            } else {
                0.0
            };

            let ridge = ridged.get([nx * 4.0, ny * 4.0]) as f32 * 0.15;
            let det = detail.get([nx * 6.0, ny * 6.0]) as f32 * 0.08;

            let flow_val = flow_noise.get([nx * 3.0 + 200.0, ny * 3.0 + 200.0]) as f32;
            let flow_channel = if flow_val > 0.3 && dist > 0.1 && dist < 0.7 {
                -0.05 * (flow_val - 0.3) * (1.0 - dist)
            } else {
                0.0
            };

            let combined = cone_curved + crater_depth + ridge + det + flow_channel;
            let height = 10.0 + combined * 70.0 * scale;
            heightmap.push(height.max(0.0).min(100.0));
        }
    }
    heightmap
}

pub fn generate_grid_tile(
    params: &TerrainParams,
    grid_size: u32,
    grid_x: u32,
    grid_y: u32,
) -> Vec<f32> {
    let seed = if params.seed == 0 {
        rand::random::<u32>()
    } else {
        params.seed
    };

    let tile_size = params.size.max(256);
    let scale = params.scale.max(0.01);
    let roughness = params.roughness.clamp(0.0, 1.0);
    let total_size = grid_size * tile_size;

    info!(
        "[TERRAIN] Generating grid tile ({},{}) of {}x{} grid, preset='{}', seed={}",
        grid_x, grid_y, grid_size, grid_size, params.preset, seed
    );

    let mut fbm = Fbm::<Perlin>::new(seed);
    fbm.octaves = octave_count(roughness);
    fbm.frequency = 2.0;
    fbm.lacunarity = 2.2;
    fbm.persistence = 0.45;

    let ridged = RidgedMulti::<Perlin>::new(seed);
    let warp = Fbm::<Perlin>::new(seed.wrapping_add(100));

    let total = (tile_size * tile_size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    let freq = 1.0 / total_size as f64;
    let x_offset = (grid_x * tile_size) as f64;
    let y_offset = (grid_y * tile_size) as f64;

    for y in 0..tile_size {
        for x in 0..tile_size {
            let gx = (x_offset + x as f64) * freq;
            let gy = (y_offset + y as f64) * freq;

            let raw = match params.preset.as_str() {
                "island" => {
                    let cx = (x_offset + x as f64) / total_size as f64 - 0.5;
                    let cy = (y_offset + y as f64) / total_size as f64 - 0.5;
                    let dist = (cx * cx + cy * cy).sqrt() * 2.0;
                    let falloff = (1.0 - (dist as f32 * 1.3)).max(0.0);
                    let n = fbm.get([gx * 4.0, gy * 4.0]) as f32 * 0.5 + 0.5;
                    (n * falloff * falloff) * 40.0 + params.water_level - 5.0
                }
                "mountains" => {
                    let r = ridged.get([gx * 3.0, gy * 3.0]) as f32;
                    let b = fbm.get([gx * 2.0, gy * 2.0]) as f32;
                    let combined = r * 0.7 + b * 0.3;
                    15.0 + (combined * 0.5 + 0.5) * 70.0
                }
                "volcanic" => {
                    let cx = (x_offset + x as f64) / total_size as f64 - 0.5;
                    let cy = (y_offset + y as f64) / total_size as f64 - 0.5;
                    let dist = ((cx * cx + cy * cy) as f32).sqrt() * 2.0;
                    let cone = (1.0 - dist * 1.2).max(0.0);
                    let cone_curved = cone * cone * 0.8 + cone * 0.2;
                    let crater = if dist < 0.08 {
                        -0.15 * (1.0 - dist / 0.08)
                    } else {
                        0.0
                    };
                    let r = ridged.get([gx * 4.0, gy * 4.0]) as f32 * 0.15;
                    10.0 + (cone_curved + crater + r) * 70.0
                }
                _ => {
                    let n = fbm.get([gx * 3.0, gy * 3.0]) as f32;
                    20.0 + (n * 0.5 + 0.5) * 25.0
                }
            };

            heightmap.push((raw * scale).max(0.0).min(100.0));
        }
    }

    info!(
        "[TERRAIN] Grid tile ({},{}) generated: {} values, range {:.1}..{:.1}",
        grid_x,
        grid_y,
        heightmap.len(),
        heightmap.iter().cloned().fold(f32::INFINITY, f32::min),
        heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
    );

    heightmap
}

pub const PRESET_NAMES: &[&str] = &[
    "island",
    "mountains",
    "rolling_hills",
    "desert",
    "tropical",
    "canyon",
    "plateau",
    "volcanic",
];

pub fn preset_description(preset: &str) -> &str {
    match preset {
        "island" => "Central landmass with beaches, underwater at borders",
        "mountains" => "Jagged peaks and deep valleys, high elevation range",
        "rolling_hills" => "Gentle undulating pastoral terrain",
        "desert" => "Sand dunes with flat basins",
        "tropical" => "Elevated center with coastal lowlands, lush feel",
        "canyon" => "Deep carved channels through plateaus",
        "plateau" => "Flat-topped mesas with steep edges",
        "volcanic" => "Central peak with crater and lava flow channels",
        _ => "Unknown terrain preset",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_heightmap(heightmap: &[f32], size: u32) {
        assert_eq!(heightmap.len(), (size * size) as usize);
        for &h in heightmap {
            assert!(h >= 0.0, "Height below 0: {}", h);
            assert!(h <= 100.0, "Height above 100: {}", h);
            assert!(h.is_finite(), "Non-finite height: {}", h);
        }
    }

    #[test]
    fn test_all_presets_produce_valid_heightmaps() {
        for preset in PRESET_NAMES {
            let params = TerrainParams {
                preset: preset.to_string(),
                seed: 12345,
                scale: 1.0,
                roughness: 0.5,
                water_level: 20.0,
                size: 256,
            };
            let hm = generate(&params);
            check_heightmap(&hm, 256);
        }
    }

    #[test]
    fn test_seed_reproducibility() {
        let params = TerrainParams {
            preset: "island".to_string(),
            seed: 42,
            scale: 1.0,
            roughness: 0.5,
            water_level: 20.0,
            size: 256,
        };
        let hm1 = generate(&params);
        let hm2 = generate(&params);
        assert_eq!(hm1, hm2, "Same seed should produce identical terrain");
    }

    #[test]
    fn test_different_seeds_differ() {
        let params1 = TerrainParams {
            preset: "mountains".to_string(),
            seed: 1,
            ..Default::default()
        };
        let params2 = TerrainParams {
            preset: "mountains".to_string(),
            seed: 2,
            ..Default::default()
        };
        let hm1 = generate(&params1);
        let hm2 = generate(&params2);
        assert_ne!(hm1, hm2, "Different seeds should produce different terrain");
    }

    #[test]
    fn test_island_low_edges_high_center() {
        let params = TerrainParams {
            preset: "island".to_string(),
            seed: 100,
            scale: 1.0,
            roughness: 0.5,
            water_level: 20.0,
            size: 256,
        };
        let hm = generate(&params);

        let edge_avg: f32 = (0..256u32)
            .map(|i| {
                hm[i as usize]
                    + hm[(255 * 256 + i) as usize]
                    + hm[(i * 256) as usize]
                    + hm[(i * 256 + 255) as usize]
            })
            .sum::<f32>()
            / (256.0 * 4.0);

        let center_region: f32 = (112..144u32)
            .flat_map(|y| (112..144u32).map(move |x| (x, y)))
            .map(|(x, y)| hm[(y * 256 + x) as usize])
            .sum::<f32>()
            / (32.0 * 32.0);

        assert!(
            center_region > edge_avg,
            "Island center ({:.1}) should be higher than edges ({:.1})",
            center_region,
            edge_avg
        );
    }

    #[test]
    fn test_volcanic_has_central_peak() {
        let params = TerrainParams {
            preset: "volcanic".to_string(),
            seed: 500,
            scale: 1.0,
            roughness: 0.5,
            water_level: 20.0,
            size: 256,
        };
        let hm = generate(&params);

        let ring_avg: f32 = (100..156u32)
            .flat_map(|y| (100..156u32).map(move |x| (x, y)))
            .filter(|(x, y)| {
                let dx = *x as f32 - 128.0;
                let dy = *y as f32 - 128.0;
                let d = (dx * dx + dy * dy).sqrt();
                d > 15.0 && d < 30.0
            })
            .map(|(x, y)| hm[(y * 256 + x) as usize])
            .sum::<f32>();

        let corner_avg: f32 = (0..32u32)
            .flat_map(|y| (0..32u32).map(move |x| (x, y)))
            .map(|(x, y)| hm[(y * 256 + x) as usize])
            .sum::<f32>()
            / (32.0 * 32.0);

        assert!(
            ring_avg / 1000.0 > corner_avg,
            "Volcanic ring should be higher than corners"
        );
    }

    #[test]
    fn test_scale_affects_height_range() {
        let params_lo = TerrainParams {
            preset: "mountains".to_string(),
            seed: 42,
            scale: 0.5,
            ..Default::default()
        };
        let params_hi = TerrainParams {
            preset: "mountains".to_string(),
            seed: 42,
            scale: 1.5,
            ..Default::default()
        };

        let hm_lo = generate(&params_lo);
        let hm_hi = generate(&params_hi);

        let range_lo = hm_lo.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            - hm_lo.iter().cloned().fold(f32::INFINITY, f32::min);
        let range_hi = hm_hi.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            - hm_hi.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(
            range_hi > range_lo,
            "Higher scale ({:.1}) should have more range ({:.1}) than lower ({:.1}, {:.1})",
            1.5,
            range_hi,
            0.5,
            range_lo
        );
    }

    #[test]
    fn test_unknown_preset_fallback() {
        let params = TerrainParams {
            preset: "nonexistent".to_string(),
            seed: 1,
            ..Default::default()
        };
        let hm = generate(&params);
        check_heightmap(&hm, 256);
    }

    #[test]
    fn test_preset_descriptions() {
        for preset in PRESET_NAMES {
            let desc = preset_description(preset);
            assert!(
                !desc.contains("Unknown"),
                "Preset '{}' should have description",
                preset
            );
        }
    }
}
