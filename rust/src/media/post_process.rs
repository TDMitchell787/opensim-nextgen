use std::path::Path;
use anyhow::{Result, anyhow};
use tracing::{info, warn};

pub async fn apply_gimp_filter(
    gimp_path: &str,
    frames_dir: &Path,
    filter_name: &str,
) -> Result<()> {
    if !Path::new(gimp_path).exists() {
        return Err(anyhow!("GIMP not found at {}", gimp_path));
    }

    let script = match filter_name {
        "golden_hour" => generate_golden_hour_script(frames_dir),
        "noir" => generate_noir_script(frames_dir),
        "vignette" => generate_vignette_script(frames_dir),
        "letterbox" => generate_letterbox_script(frames_dir),
        "cool_moonlight" => generate_cool_moonlight_script(frames_dir),
        "film_grain" => generate_film_grain_script(frames_dir),
        _ => {
            warn!("[MEDIA] Unknown GIMP filter '{}', skipping", filter_name);
            return Ok(());
        }
    };

    info!("[MEDIA] Applying GIMP filter '{}' to {}", filter_name, frames_dir.display());

    let output = tokio::process::Command::new(gimp_path)
        .args([
            "-i",
            "-b", &script,
            "-b", "(gimp-quit 0)",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("[MEDIA] GIMP filter warning: {}", stderr.lines().last().unwrap_or(""));
    }

    info!("[MEDIA] GIMP filter '{}' applied", filter_name);
    Ok(())
}

fn generate_golden_hour_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image))))
      (gimp-curves-spline drawable HISTOGRAM-RED 10 #(0 0 64 80 128 160 192 220 255 255))
      (gimp-curves-spline drawable HISTOGRAM-GREEN 10 #(0 0 64 60 128 130 192 200 255 240))
      (gimp-curves-spline drawable HISTOGRAM-BLUE 10 #(0 0 64 40 128 100 192 160 255 200))
      (gimp-brightness-contrast drawable 10 15)
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}

fn generate_noir_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image))))
      (gimp-drawable-desaturate drawable DESATURATE-LUMINOSITY)
      (gimp-brightness-contrast drawable -10 40)
      (gimp-curves-spline drawable HISTOGRAM-VALUE 10 #(0 0 64 20 128 140 192 230 255 255))
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}

fn generate_vignette_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image)))
           (width (car (gimp-image-width image)))
           (height (car (gimp-image-height image)))
           (vignette-layer (car (gimp-layer-new image width height RGBA-IMAGE "Vignette" 80 LAYER-MODE-MULTIPLY))))
      (gimp-image-insert-layer image vignette-layer 0 -1)
      (gimp-edit-fill vignette-layer FILL-BLACK)
      (gimp-image-set-active-layer image vignette-layer)
      (gimp-ellipse-select image (* width 0.1) (* height 0.1) (* width 0.8) (* height 0.8) CHANNEL-OP-REPLACE TRUE (* width 0.3) TRUE)
      (gimp-edit-clear vignette-layer)
      (gimp-selection-none image)
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}

fn generate_letterbox_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image)))
           (width (car (gimp-image-width image)))
           (height (car (gimp-image-height image)))
           (bar-height (/ (* height 12) 100))
           (bar-layer (car (gimp-layer-new image width height RGBA-IMAGE "Letterbox" 100 LAYER-MODE-NORMAL))))
      (gimp-image-insert-layer image bar-layer 0 -1)
      (gimp-edit-fill bar-layer FILL-TRANSPARENT)
      (gimp-image-set-active-layer image bar-layer)
      (gimp-palette-set-foreground '(0 0 0))
      (gimp-image-select-rectangle image CHANNEL-OP-REPLACE 0 0 width bar-height)
      (gimp-edit-fill bar-layer FILL-FOREGROUND)
      (gimp-image-select-rectangle image CHANNEL-OP-REPLACE 0 (- height bar-height) width bar-height)
      (gimp-edit-fill bar-layer FILL-FOREGROUND)
      (gimp-selection-none image)
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}

fn generate_cool_moonlight_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image))))
      (gimp-curves-spline drawable HISTOGRAM-RED 10 #(0 0 64 50 128 110 192 170 255 220))
      (gimp-curves-spline drawable HISTOGRAM-GREEN 10 #(0 0 64 55 128 120 192 180 255 230))
      (gimp-curves-spline drawable HISTOGRAM-BLUE 10 #(0 10 64 80 128 150 192 210 255 255))
      (gimp-brightness-contrast drawable -15 10)
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}

fn generate_film_grain_script(frames_dir: &Path) -> String {
    let dir = frames_dir.display().to_string().replace('\\', "/");
    format!(
        r#"(let* ((filelist (cadr (file-glob "{dir}/*.png" 1))))
  (while (not (null? filelist))
    (let* ((filename (car filelist))
           (image (car (gimp-file-load RUN-NONINTERACTIVE filename filename)))
           (drawable (car (gimp-image-flatten image))))
      (plug-in-hsv-noise RUN-NONINTERACTIVE image drawable 5 0 0 25)
      (gimp-image-flatten image)
      (file-png-save RUN-NONINTERACTIVE image (car (gimp-image-flatten image)) filename filename 0 9 1 1 1 1 1)
      (gimp-image-delete image))
    (set! filelist (cdr filelist))))"#
    )
}
