use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::{MaterialRange, Mesh};
use glc_core::scene::{SceneNode, World};
use glc_core::types::EntityId;
use std::path::Path;

/// Load an STL file (ASCII or binary) from disk, producing a `World`.
pub fn load_stl(path: &Path) -> Result<World> {
    let path_str = path.to_string_lossy();
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| IoError::FileNotFound(format!("{path_str}: {e}")))?;

    let mut reader = std::io::BufReader::new(file);
    load_stl_from_reader(&mut reader, &path_str)
}

/// Load an STL model from in-memory bytes (for web).
pub fn load_stl_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let mut reader = std::io::Cursor::new(bytes);
    load_stl_from_reader(&mut reader, name)
}

fn load_stl_from_reader<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    name: &str,
) -> Result<World> {
    let stl = stl_io::read_stl(reader)
        .map_err(|e| IoError::StlError(format!("{name}: {e}")))?;

    let num_vertices = stl.vertices.len();
    let num_faces = stl.faces.len();

    // Convert vertices to flat f32 arrays
    let mut positions = Vec::with_capacity(num_vertices * 3);
    for v in &stl.vertices {
        positions.push(v[0]);
        positions.push(v[1]);
        positions.push(v[2]);
    }

    // Build indices and per-vertex normals from face data.
    // STL stores per-face normals; we expand to per-vertex for consistency.
    // Since STL uses indexed vertices shared across faces, we need to
    // accumulate face normals per vertex and normalize.
    let mut normals = vec![0.0f32; num_vertices * 3];
    let mut indices = Vec::with_capacity(num_faces * 3);

    for face in &stl.faces {
        let nx = face.normal[0];
        let ny = face.normal[1];
        let nz = face.normal[2];

        for &vi in &face.vertices {
            indices.push(vi as u32);
            normals[vi * 3] += nx;
            normals[vi * 3 + 1] += ny;
            normals[vi * 3 + 2] += nz;
        }
    }

    // Normalize accumulated normals
    for chunk in normals.chunks_exact_mut(3) {
        let len = (chunk[0] * chunk[0] + chunk[1] * chunk[1] + chunk[2] * chunk[2]).sqrt();
        if len > 1e-8 {
            chunk[0] /= len;
            chunk[1] /= len;
            chunk[2] /= len;
        }
    }

    let mesh = Mesh {
        id: EntityId::new(),
        name: name.to_string(),
        positions,
        normals,
        tex_coords: Vec::new(), // STL has no texture coords
        indices,
        line_indices: Vec::new(),
        material_ranges: vec![MaterialRange {
            material_index: 0,
            start: 0,
            count: (num_faces * 3) as u32,
        }],
        lod: 0,
    };

    let mut world = World::new();
    world.source_path = Some(name.to_string());
    world.add_material(Material::default());

    let mesh_idx = world.add_mesh(mesh);
    let root = world.add_node(SceneNode::with_mesh(name, mesh_idx));
    world.root = Some(root);

    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_ascii_stl(dir: &Path) -> std::path::PathBuf {
        let stl_path = dir.join("triangle.stl");
        let mut f = std::fs::File::create(&stl_path).unwrap();
        write!(
            f,
            "\
solid triangle
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid triangle
"
        )
        .unwrap();
        stl_path
    }

    #[test]
    fn test_load_ascii_stl() {
        let dir = tempfile::tempdir().unwrap();
        let stl_path = write_ascii_stl(dir.path());
        let world = load_stl(&stl_path).unwrap();

        assert_eq!(world.meshes.len(), 1);
        let mesh = &world.meshes[0];
        assert_eq!(mesh.face_count(), 1, "1 triangle");
        assert_eq!(mesh.vertex_count(), 3, "3 vertices");
        assert!(!mesh.normals.is_empty());
    }

    #[test]
    fn test_load_stl_from_bytes() {
        let stl_ascii = b"\
solid test
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 1 0 0
      vertex 1 1 0
      vertex 0 1 0
    endloop
  endfacet
endsolid test
";
        let world = load_stl_from_bytes(stl_ascii, "test.stl").unwrap();
        assert_eq!(world.meshes[0].face_count(), 2);
    }
}
