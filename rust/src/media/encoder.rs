use anyhow::{anyhow, Result};
use std::path::Path;
use tracing::{info, warn};

pub async fn encode_video(
    ffmpeg_path: &str,
    frames_dir: &Path,
    output_path: &Path,
    fps: u32,
) -> Result<()> {
    if !Path::new(ffmpeg_path).exists() {
        return Err(anyhow!("ffmpeg not found at {}", ffmpeg_path));
    }

    let frame_pattern = frames_dir.join("frame_%04d.png");

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    info!(
        "[MEDIA] Encoding video: {} → {}",
        frames_dir.display(),
        output_path.display()
    );

    let fps_str = fps.to_string();
    let output = tokio::process::Command::new(ffmpeg_path)
        .args([
            "-y",
            "-framerate",
            &fps_str,
            "-i",
            frame_pattern.to_str().unwrap_or("frame_%04d.png"),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "18",
            "-r",
            &fps_str,
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap_or("output.mp4"),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("[MEDIA] ffmpeg stderr: {}", stderr);
        return Err(anyhow!(
            "ffmpeg encoding failed: {}",
            stderr.lines().last().unwrap_or("unknown error")
        ));
    }

    info!("[MEDIA] Video encoded: {}", output_path.display());
    Ok(())
}

pub async fn mux_audio(
    ffmpeg_path: &str,
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<()> {
    if !Path::new(ffmpeg_path).exists() {
        return Err(anyhow!("ffmpeg not found at {}", ffmpeg_path));
    }

    info!(
        "[MEDIA] Muxing audio: {} + {} → {}",
        video_path.display(),
        audio_path.display(),
        output_path.display()
    );

    let output = tokio::process::Command::new(ffmpeg_path)
        .args([
            "-y",
            "-i",
            video_path.to_str().unwrap_or("video.mp4"),
            "-i",
            audio_path.to_str().unwrap_or("audio.wav"),
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-shortest",
            output_path.to_str().unwrap_or("output.mp4"),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "ffmpeg mux failed: {}",
            stderr.lines().last().unwrap_or("unknown error")
        ));
    }

    info!("[MEDIA] Audio muxed successfully");
    Ok(())
}

pub async fn encode_stills_to_gif(
    ffmpeg_path: &str,
    frames_dir: &Path,
    output_path: &Path,
    fps: u32,
) -> Result<()> {
    let frame_pattern = frames_dir.join("frame_%04d.png");

    let output = tokio::process::Command::new(ffmpeg_path)
        .args([
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
            frame_pattern.to_str().unwrap_or("frame_%04d.png"),
            "-vf",
            "fps=10,scale=480:-1:flags=lanczos",
            output_path.to_str().unwrap_or("output.gif"),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "GIF encoding failed: {}",
            stderr.lines().last().unwrap_or("unknown error")
        ));
    }

    Ok(())
}
