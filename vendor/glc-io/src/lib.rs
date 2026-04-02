pub mod album;
#[cfg(feature = "assimp")]
pub mod assimp_loader;
pub mod error;
pub mod export;
pub mod gltf_loader;
pub mod obj;
pub mod off;
pub mod ply;
pub mod stl;
pub mod texture;
pub mod three_dxml;

use error::{IoError, Result};
use glc_core::scene::World;
use std::path::Path;

/// Supported 3D file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    Obj,
    Stl,
    Off,
    ThreeDxml,
    Gltf,
    Ply,
}

/// Detect the model format from a file extension.
pub fn detect_format(path: &Path) -> Option<ModelFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "obj" => Some(ModelFormat::Obj),
        "stl" => Some(ModelFormat::Stl),
        "off" => Some(ModelFormat::Off),
        "3dxml" => Some(ModelFormat::ThreeDxml),
        "gltf" | "glb" => Some(ModelFormat::Gltf),
        "ply" => Some(ModelFormat::Ply),
        _ => None,
    }
}

/// Load a 3D model from a file path (desktop).
/// Automatically detects format from the file extension.
/// Falls back to assimp for unrecognized extensions when the `assimp` feature is enabled.
pub fn load_model(path: &Path) -> Result<World> {
    if !path.exists() {
        return Err(IoError::FileNotFound(path.to_string_lossy().into_owned()));
    }

    if let Some(format) = detect_format(path) {
        return match format {
            ModelFormat::Obj => obj::load_obj(path),
            ModelFormat::Stl => stl::load_stl(path),
            ModelFormat::Off => off::load_off(path),
            ModelFormat::ThreeDxml => three_dxml::load_3dxml(path),
            ModelFormat::Gltf => gltf_loader::load_gltf(path),
            ModelFormat::Ply => ply::load_ply(path),
        };
    }

    // Fall back to assimp for formats we don't handle natively
    #[cfg(feature = "assimp")]
    return assimp_loader::load_assimp(path);

    #[cfg(not(feature = "assimp"))]
    Err(IoError::UnsupportedFormat(path.to_string_lossy().into_owned()))
}

/// Load a 3D model from in-memory bytes (web).
/// `name` should include a file extension for format detection.
/// Falls back to assimp for unrecognized extensions when the `assimp` feature is enabled.
pub fn load_model_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let path = Path::new(name);

    if let Some(format) = detect_format(path) {
        return match format {
            ModelFormat::Obj => obj::load_obj_from_bytes(bytes, name),
            ModelFormat::Stl => stl::load_stl_from_bytes(bytes, name),
            ModelFormat::Off => off::load_off_from_bytes(bytes, name),
            ModelFormat::ThreeDxml => three_dxml::load_3dxml_from_bytes(bytes, name),
            ModelFormat::Gltf => gltf_loader::load_gltf_from_bytes(bytes, name),
            ModelFormat::Ply => ply::load_ply_from_bytes(bytes, name),
        };
    }

    // Fall back to assimp for formats we don't handle natively
    #[cfg(feature = "assimp")]
    return assimp_loader::load_assimp_from_bytes(bytes, name);

    #[cfg(not(feature = "assimp"))]
    Err(IoError::UnsupportedFormat(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_format_obj() {
        assert_eq!(detect_format(Path::new("model.obj")), Some(ModelFormat::Obj));
        assert_eq!(detect_format(Path::new("model.OBJ")), Some(ModelFormat::Obj));
    }

    #[test]
    fn test_detect_format_stl() {
        assert_eq!(detect_format(Path::new("model.stl")), Some(ModelFormat::Stl));
        assert_eq!(detect_format(Path::new("model.STL")), Some(ModelFormat::Stl));
    }

    #[test]
    fn test_detect_format_off() {
        assert_eq!(detect_format(Path::new("model.off")), Some(ModelFormat::Off));
        assert_eq!(detect_format(Path::new("model.OFF")), Some(ModelFormat::Off));
    }

    #[test]
    fn test_detect_format_3dxml() {
        assert_eq!(detect_format(Path::new("model.3dxml")), Some(ModelFormat::ThreeDxml));
        assert_eq!(detect_format(Path::new("model.3DXML")), Some(ModelFormat::ThreeDxml));
    }

    #[test]
    fn test_detect_format_gltf() {
        assert_eq!(detect_format(Path::new("model.gltf")), Some(ModelFormat::Gltf));
        assert_eq!(detect_format(Path::new("model.glb")), Some(ModelFormat::Gltf));
        assert_eq!(detect_format(Path::new("model.GLB")), Some(ModelFormat::Gltf));
    }

    #[test]
    fn test_detect_format_ply() {
        assert_eq!(detect_format(Path::new("model.ply")), Some(ModelFormat::Ply));
        assert_eq!(detect_format(Path::new("model.PLY")), Some(ModelFormat::Ply));
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(detect_format(Path::new("model.fbx")), None);
        assert_eq!(detect_format(Path::new("model")), None);
    }

    #[test]
    fn test_load_model_file_not_found() {
        let result = load_model(Path::new("/nonexistent/model.obj"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_model_unsupported() {
        // .xyz is not supported by any loader including assimp
        let result = load_model(Path::new("/nonexistent/model.xyz123"));
        assert!(result.is_err());
    }
}
