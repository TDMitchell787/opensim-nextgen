use std::path::Path;
use anyhow::{Result, anyhow};
use tracing::{info, warn};

pub async fn run_blender(blender_path: &str, script_path: &Path) -> Result<()> {
    if !Path::new(blender_path).exists() {
        return Err(anyhow!("Blender not found at {}", blender_path));
    }

    info!("[MEDIA] Starting Blender render: {}", script_path.display());

    let output = tokio::process::Command::new(blender_path)
        .args([
            "--background",
            "--python", script_path.to_str().unwrap_or("render.py"),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stdout.lines() {
        if line.starts_with("Fra:") || line.contains("Render") || line.contains("complete") {
            info!("[MEDIA] Blender: {}", line);
        }
    }

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        warn!("[MEDIA] Blender stderr: {}", stderr);
        return Err(anyhow!("Blender exited with code {}: {}", exit_code,
            stderr.lines().last().unwrap_or("unknown error")));
    }

    info!("[MEDIA] Blender render complete");
    Ok(())
}

pub async fn run_blender_with_blend(
    blender_path: &str,
    blend_file: &Path,
    script_path: &Path,
) -> Result<()> {
    if !Path::new(blender_path).exists() {
        return Err(anyhow!("Blender not found at {}", blender_path));
    }

    let output = tokio::process::Command::new(blender_path)
        .args([
            "--background",
            blend_file.to_str().unwrap_or("scene.blend"),
            "--python", script_path.to_str().unwrap_or("render.py"),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Blender exited with error: {}",
            stderr.lines().last().unwrap_or("unknown")));
    }

    Ok(())
}
