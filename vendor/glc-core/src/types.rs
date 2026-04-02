use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique entity identifier, matching GLC_uint in the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Generate a new unique ID (thread-safe monotonic counter).
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// RGBA color with f32 components in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4f {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl Default for Color4f {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Polygon rendering mode, matching GL_POINT/GL_LINE/GL_FILL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolygonMode {
    Point,
    Line,
    Fill,
}

impl Default for PolygonMode {
    fn default() -> Self {
        Self::Fill
    }
}

/// High-level render mode combining polygon mode and shading flags.
/// Matches the original's ShadingFlag / WireRenderFlag combinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    /// GL_FILL + ShadingFlag — standard solid shading (default)
    Shading,
    /// GL_LINE + ShadingFlag — wireframe only
    Wireframe,
    /// GL_POINT + ShadingFlag — points only
    Points,
    /// GL_FILL + WireRenderFlag — solid with wireframe overlay
    ShadingWithWireframe,
    /// Overwrite transparency mode
    OverwriteTransparency,
    /// Overwrite transparency and material (GLC_lib 2.5.2)
    OverwriteTransparencyAndMaterial,
    /// Outline/silhouette edge rendering (GLC_lib 2.5.2)
    OutlineSilhouette,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Shading
    }
}
