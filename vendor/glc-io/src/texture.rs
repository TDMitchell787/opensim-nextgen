use glc_core::material::TextureData;
use std::path::Path;

/// Decode image bytes (JPEG, PNG) into RGBA pixel data.
pub fn decode_texture_bytes(bytes: &[u8]) -> Option<TextureData> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(TextureData {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

/// Load and decode a texture image file from disk.
pub fn load_texture_from_file(path: &Path) -> Option<TextureData> {
    let bytes = std::fs::read(path).ok()?;
    let result = decode_texture_bytes(&bytes);
    if result.is_none() {
        log::warn!("Failed to decode texture: {}", path.display());
    }
    result
}
