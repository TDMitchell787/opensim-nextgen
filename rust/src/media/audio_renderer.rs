use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use tracing::info;

pub fn render_ambient_audio(preset: &str, duration_secs: f32, output_path: &Path) -> Result<()> {
    let sample_rate: u32 = 44100;
    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let total_samples = (sample_rate as f32 * duration_secs) as usize;

    let samples = generate_ambient_samples(preset, total_samples, sample_rate);

    write_wav(
        output_path,
        &samples,
        sample_rate,
        channels,
        bits_per_sample,
    )?;

    info!(
        "[MEDIA] Audio rendered: {} ({:.1}s, preset='{}')",
        output_path.display(),
        duration_secs,
        preset
    );
    Ok(())
}

fn generate_ambient_samples(preset: &str, total_samples: usize, sample_rate: u32) -> Vec<i16> {
    let mut samples = vec![0i16; total_samples * 2];
    let sr = sample_rate as f64;

    match preset {
        "ocean" | "ocean_ambient" => {
            for i in 0..total_samples {
                let t = i as f64 / sr;
                let wave1 = (t * 0.15 * std::f64::consts::TAU).sin() * 0.3;
                let wave2 = (t * 0.23 * std::f64::consts::TAU).sin() * 0.2;
                let wave3 = (t * 0.08 * std::f64::consts::TAU).sin() * 0.15;
                let noise = pseudo_noise(i) * 0.2;
                let crash = ((t * 0.05 * std::f64::consts::TAU).sin().abs() * 2.0).min(1.0)
                    * pseudo_noise(i + 1000)
                    * 0.15;
                let val = ((wave1 + wave2 + wave3 + noise + crash) * 8000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = ((val as f64) * 0.95 + pseudo_noise(i + 500) * 200.0) as i16;
            }
        }
        "wind" => {
            for i in 0..total_samples {
                let t = i as f64 / sr;
                let gust = (t * 0.1 * std::f64::consts::TAU).sin() * 0.5 + 0.5;
                let noise = pseudo_noise(i) * gust * 0.4;
                let whistle = (t * 800.0 * std::f64::consts::TAU).sin()
                    * (t * 0.07 * std::f64::consts::TAU).sin().abs()
                    * 0.05;
                let val = ((noise + whistle) * 10000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = ((val as f64) * 0.9 + pseudo_noise(i + 300) * 500.0) as i16;
            }
        }
        "rain" => {
            for i in 0..total_samples {
                let noise = pseudo_noise(i) * 0.3 + pseudo_noise(i * 3) * 0.15;
                let drops = if pseudo_noise(i * 7).abs() > 0.97 {
                    pseudo_noise(i * 11) * 0.4
                } else {
                    0.0
                };
                let val = ((noise + drops) * 10000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = ((val as f64) * 0.92 + pseudo_noise(i + 777) * 300.0) as i16;
            }
        }
        "forest" => {
            for i in 0..total_samples {
                let t = i as f64 / sr;
                let breeze = pseudo_noise(i) * 0.1;
                let bird1 = if (t * 0.3).fract() < 0.02 {
                    (t * 2200.0 * std::f64::consts::TAU).sin()
                        * 0.15
                        * (1.0 - (t * 0.3).fract() / 0.02)
                } else {
                    0.0
                };
                let bird2 = if (t * 0.17 + 0.5).fract() < 0.03 {
                    (t * 3400.0 * std::f64::consts::TAU).sin() * 0.1
                } else {
                    0.0
                };
                let crickets = (t * 4500.0 * std::f64::consts::TAU).sin()
                    * (t * 7.0 * std::f64::consts::TAU).sin().abs()
                    * 0.04;
                let val = ((breeze + bird1 + bird2 + crickets) * 10000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = ((val as f64) * 0.85 + pseudo_noise(i + 999) * 200.0) as i16;
            }
        }
        "urban" | "city" => {
            for i in 0..total_samples {
                let t = i as f64 / sr;
                let traffic =
                    pseudo_noise(i) * 0.15 * (1.0 + (t * 0.02 * std::f64::consts::TAU).sin() * 0.3);
                let hum = (t * 60.0 * std::f64::consts::TAU).sin() * 0.03;
                let distant = pseudo_noise(i * 5) * 0.08;
                let val = ((traffic + hum + distant) * 10000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = ((val as f64) * 0.88 + pseudo_noise(i + 1234) * 400.0) as i16;
            }
        }
        _ => {
            for i in 0..total_samples {
                let t = i as f64 / sr;
                let pad = (t * 110.0 * std::f64::consts::TAU).sin() * 0.08
                    + (t * 165.0 * std::f64::consts::TAU).sin() * 0.05
                    + (t * 220.0 * std::f64::consts::TAU).sin() * 0.03;
                let noise = pseudo_noise(i) * 0.05;
                let val = ((pad + noise) * 10000.0) as i16;
                samples[i * 2] = val;
                samples[i * 2 + 1] = val;
            }
        }
    }

    samples
}

fn pseudo_noise(seed: usize) -> f64 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let val = ((x >> 16) & 0x7FFF) as f64 / 32767.0;
    val * 2.0 - 1.0
}

fn write_wav(
    path: &Path,
    samples: &[i16],
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(path)?;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2;
    let file_size = 36 + data_size;

    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}
