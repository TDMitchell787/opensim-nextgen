use crate::bounding_box::BoundingBox;
use crate::types::EntityId;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// A range of indices into the mesh's index buffer that share a material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRange {
    /// Index of the material in the parent World's material list.
    pub material_index: usize,
    /// Start index (inclusive) into the mesh's index buffer.
    pub start: u32,
    /// Number of indices in this range.
    pub count: u32,
}

/// Triangle mesh geometry with interleaved vertex attributes.
///
/// Vertex layout: position (Vec3), normal (Vec3), tex_coord (Vec2).
/// Indices are u32 triangle lists.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub id: EntityId,
    pub name: String,

    /// Vertex positions, 3 floats per vertex (x, y, z).
    pub positions: Vec<f32>,
    /// Vertex normals, 3 floats per vertex (nx, ny, nz).
    pub normals: Vec<f32>,
    /// Texture coordinates, 2 floats per vertex (u, v). May be empty.
    pub tex_coords: Vec<f32>,
    /// Triangle indices (3 per triangle).
    pub indices: Vec<u32>,
    /// Line segment indices (pairs of vertex indices for edge rendering). May be empty.
    pub line_indices: Vec<u32>,

    /// Material assignment ranges (sorted by start index, non-overlapping).
    pub material_ranges: Vec<MaterialRange>,

    /// Level of detail value (0 = full detail).
    pub lod: u32,
}

impl Mesh {
    /// Number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Number of triangles (faces) in this mesh.
    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Compute the axis-aligned bounding box of this mesh's positions.
    pub fn bounding_box(&self) -> BoundingBox {
        let points: Vec<Vec3> = self
            .positions
            .chunks_exact(3)
            .map(|c| Vec3::new(c[0], c[1], c[2]))
            .collect();
        BoundingBox::from_points(&points)
    }

    /// Returns true if this mesh has texture coordinates.
    pub fn has_tex_coords(&self) -> bool {
        !self.tex_coords.is_empty()
    }
}

