use crate::types::{Color4f, EntityId};
use serde::{Deserialize, Serialize};

/// Decoded texture image data (RGBA pixels).
#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Material definition matching GLC_Material properties.
///
/// Supports Blinn-Phong lighting model with ambient, diffuse, specular,
/// and emissive color components plus an optional diffuse texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: EntityId,
    pub name: String,
    pub ambient: Color4f,
    pub diffuse: Color4f,
    pub specular: Color4f,
    pub emissive: Color4f,
    /// Shininess exponent for specular highlights (0.0–128.0).
    pub shininess: f32,
    /// Overall opacity (0.0 = fully transparent, 1.0 = fully opaque).
    pub opacity: f32,
    /// Path to diffuse texture file, if any.
    pub texture_path: Option<String>,
    /// Decoded texture image data, if any.
    #[serde(skip)]
    pub texture_data: Option<TextureData>,
    /// Whether this material has been modified from its loaded state.
    pub is_modified: bool,
}

impl Material {
    /// Default gray material matching GLC_lib's default.
    pub fn default_material() -> Self {
        Self {
            id: EntityId::new(),
            name: "Default".to_string(),
            ambient: Color4f::new(0.2, 0.2, 0.2, 1.0),
            diffuse: Color4f::new(0.8, 0.8, 0.8, 1.0),
            specular: Color4f::new(1.0, 1.0, 1.0, 1.0),
            emissive: Color4f::BLACK,
            shininess: 50.0,
            opacity: 1.0,
            texture_path: None,
            texture_data: None,
            is_modified: false,
        }
    }

    /// Returns true if this material uses transparency.
    pub fn is_transparent(&self) -> bool {
        self.opacity < 1.0 || self.diffuse.a < 1.0
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::default_material()
    }
}
