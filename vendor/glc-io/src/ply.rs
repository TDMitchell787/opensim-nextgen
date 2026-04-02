use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::Mesh;
use glc_core::scene::{SceneNode, World};
use glc_core::types::Color4f;
use glam::Vec3;
use ply_rs::parser::Parser;
use ply_rs::ply::{DefaultElement, Property};
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

/// Load a PLY (Stanford Polygon) file from disk.
pub fn load_ply(path: &Path) -> Result<World> {
    let file = std::fs::File::open(path)
        .map_err(|e| IoError::ParseError(format!("PLY open: {e}")))?;
    let mut reader = BufReader::new(file);
    load_ply_reader(&mut reader)
}

/// Load a PLY file from in-memory bytes.
pub fn load_ply_from_bytes(bytes: &[u8], _name: &str) -> Result<World> {
    let mut reader = BufReader::new(Cursor::new(bytes));
    load_ply_reader(&mut reader)
}

fn load_ply_reader<R: Read>(reader: &mut BufReader<R>) -> Result<World> {
    let parser = Parser::<DefaultElement>::new();

    let ply = parser
        .read_ply(reader)
        .map_err(|e| IoError::ParseError(format!("PLY parse: {e}")))?;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut vertex_colors = Vec::new(); // r,g,b per vertex as 0-1 floats
    let mut indices = Vec::new();

    // Read vertices
    if let Some(vertices) = ply.payload.get("vertex") {
        for v in vertices {
            let x = get_float(v, "x").unwrap_or(0.0);
            let y = get_float(v, "y").unwrap_or(0.0);
            let z = get_float(v, "z").unwrap_or(0.0);
            positions.extend_from_slice(&[x, y, z]);

            if let (Some(nx), Some(ny), Some(nz)) =
                (get_float(v, "nx"), get_float(v, "ny"), get_float(v, "nz"))
            {
                normals.extend_from_slice(&[nx, ny, nz]);
            }

            // Vertex colors (red/green/blue as u8 or float)
            if let (Some(r), Some(g), Some(b)) = (
                get_color_component(v, "red"),
                get_color_component(v, "green"),
                get_color_component(v, "blue"),
            ) {
                vertex_colors.extend_from_slice(&[r, g, b]);
            }
        }
    }

    // Read faces
    if let Some(faces) = ply.payload.get("face") {
        for face in faces {
            if let Some(Property::ListUInt(ref idx_list)) = face.get("vertex_indices") {
                triangulate_face(idx_list, &mut indices);
            } else if let Some(Property::ListInt(ref idx_list)) = face.get("vertex_indices") {
                let uints: Vec<u32> = idx_list.iter().map(|&i| i as u32).collect();
                triangulate_face(&uints, &mut indices);
            } else if let Some(Property::ListUInt(ref idx_list)) = face.get("vertex_index") {
                triangulate_face(idx_list, &mut indices);
            } else if let Some(Property::ListInt(ref idx_list)) = face.get("vertex_index") {
                let uints: Vec<u32> = idx_list.iter().map(|&i| i as u32).collect();
                triangulate_face(&uints, &mut indices);
            }
        }
    }

    // Generate normals if not provided
    if normals.is_empty() && !indices.is_empty() {
        normals = compute_normals(&positions, &indices);
    }

    let mut world = World::new();

    // Create material from vertex colors (average color) or use default
    let material = if !vertex_colors.is_empty() {
        let n = (vertex_colors.len() / 3) as f32;
        let avg_r: f32 = vertex_colors.iter().step_by(3).sum::<f32>() / n;
        let avg_g: f32 = vertex_colors.iter().skip(1).step_by(3).sum::<f32>() / n;
        let avg_b: f32 = vertex_colors.iter().skip(2).step_by(3).sum::<f32>() / n;
        Material {
            name: "ply_vertex_color".to_string(),
            diffuse: Color4f::new(avg_r, avg_g, avg_b, 1.0),
            ambient: Color4f::new(avg_r * 0.2, avg_g * 0.2, avg_b * 0.2, 1.0),
            ..Material::default()
        }
    } else {
        Material::default()
    };
    world.add_material(material);

    let mesh = Mesh {
        name: "ply_mesh".to_string(),
        positions,
        normals,
        tex_coords: Vec::new(),
        indices,
        ..Mesh::default()
    };
    let mesh_idx = world.add_mesh(mesh);

    // Build scene tree
    let root = world.add_node(SceneNode::new("Root"));
    let child = world.add_node(SceneNode::with_mesh("PLY", mesh_idx));
    world.set_parent(child, root);
    world.root = Some(root);

    Ok(world)
}

/// Extract a float property from a PLY element (handles Float, Double, Int, UInt).
fn get_float(element: &DefaultElement, name: &str) -> Option<f32> {
    match element.get(name)? {
        Property::Float(f) => Some(*f),
        Property::Double(d) => Some(*d as f32),
        Property::Int(i) => Some(*i as f32),
        Property::UInt(u) => Some(*u as f32),
        Property::Short(s) => Some(*s as f32),
        Property::UShort(u) => Some(*u as f32),
        Property::Char(c) => Some(*c as f32),
        Property::UChar(c) => Some(*c as f32),
        _ => None,
    }
}

/// Extract a color component (0-1 float) from a PLY element.
/// Handles both u8 (0-255) and float (0-1) representations.
fn get_color_component(element: &DefaultElement, name: &str) -> Option<f32> {
    match element.get(name)? {
        Property::UChar(c) => Some(*c as f32 / 255.0),
        Property::Char(c) => Some((*c as f32).max(0.0) / 127.0),
        Property::Float(f) => Some(*f),
        Property::Double(d) => Some(*d as f32),
        _ => None,
    }
}

/// Triangulate a polygon face (fan triangulation from first vertex).
fn triangulate_face(vertex_indices: &[u32], out: &mut Vec<u32>) {
    if vertex_indices.len() < 3 {
        return;
    }
    for i in 1..vertex_indices.len() - 1 {
        out.push(vertex_indices[0]);
        out.push(vertex_indices[i]);
        out.push(vertex_indices[i + 1]);
    }
}

/// Compute smooth normals from positions and indices.
fn compute_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0f32; vertex_count * 3];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let v0 = Vec3::new(
            positions[i0 * 3],
            positions[i0 * 3 + 1],
            positions[i0 * 3 + 2],
        );
        let v1 = Vec3::new(
            positions[i1 * 3],
            positions[i1 * 3 + 1],
            positions[i1 * 3 + 2],
        );
        let v2 = Vec3::new(
            positions[i2 * 3],
            positions[i2 * 3 + 1],
            positions[i2 * 3 + 2],
        );

        let normal = (v1 - v0).cross(v2 - v0);
        for &idx in &[i0, i1, i2] {
            normals[idx * 3] += normal.x;
            normals[idx * 3 + 1] += normal.y;
            normals[idx * 3 + 2] += normal.z;
        }
    }

    for chunk in normals.chunks_exact_mut(3) {
        let n = Vec3::new(chunk[0], chunk[1], chunk[2]);
        let len = n.length();
        if len > 1e-8 {
            chunk[0] /= len;
            chunk[1] /= len;
            chunk[2] /= len;
        } else {
            chunk[2] = 1.0;
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascii_ply_triangle() -> Vec<u8> {
        b"ply\r\n\
format ascii 1.0\r\n\
element vertex 3\r\n\
property float x\r\n\
property float y\r\n\
property float z\r\n\
element face 1\r\n\
property list uchar uint vertex_indices\r\n\
end_header\r\n\
0 0 0\r\n\
1 0 0\r\n\
0 1 0\r\n\
3 0 1 2\r\n"
            .to_vec()
    }

    #[test]
    fn test_load_ply_triangle() {
        let data = ascii_ply_triangle();
        let world = load_ply_from_bytes(&data, "test.ply").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert_eq!(world.meshes[0].positions.len(), 9); // 3 verts * 3 floats
        assert_eq!(world.meshes[0].indices.len(), 3);
        assert!(world.root.is_some());
    }

    #[test]
    fn test_load_ply_with_normals() {
        let data = b"ply\r\n\
format ascii 1.0\r\n\
element vertex 3\r\n\
property float x\r\n\
property float y\r\n\
property float z\r\n\
property float nx\r\n\
property float ny\r\n\
property float nz\r\n\
element face 1\r\n\
property list uchar uint vertex_indices\r\n\
end_header\r\n\
0 0 0 0 0 1\r\n\
1 0 0 0 0 1\r\n\
0 1 0 0 0 1\r\n\
3 0 1 2\r\n";
        let world = load_ply_from_bytes(data, "test.ply").unwrap();
        assert_eq!(world.meshes[0].normals.len(), 9);
        // Check normals are what we specified
        assert!((world.meshes[0].normals[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_load_ply_quad_triangulation() {
        let data = b"ply\r\n\
format ascii 1.0\r\n\
element vertex 4\r\n\
property float x\r\n\
property float y\r\n\
property float z\r\n\
element face 1\r\n\
property list uchar uint vertex_indices\r\n\
end_header\r\n\
0 0 0\r\n\
1 0 0\r\n\
1 1 0\r\n\
0 1 0\r\n\
4 0 1 2 3\r\n";
        let world = load_ply_from_bytes(data, "test.ply").unwrap();
        // Quad → 2 triangles = 6 indices
        assert_eq!(world.meshes[0].indices.len(), 6);
    }

    #[test]
    fn test_load_ply_vertex_colors() {
        let data = b"ply\r\n\
format ascii 1.0\r\n\
element vertex 3\r\n\
property float x\r\n\
property float y\r\n\
property float z\r\n\
property uchar red\r\n\
property uchar green\r\n\
property uchar blue\r\n\
element face 1\r\n\
property list uchar uint vertex_indices\r\n\
end_header\r\n\
0 0 0 255 0 0\r\n\
1 0 0 0 255 0\r\n\
0 1 0 0 0 255\r\n\
3 0 1 2\r\n";
        let world = load_ply_from_bytes(data, "test.ply").unwrap();
        // Material should have averaged vertex colors
        assert_eq!(world.materials.len(), 1);
        let mat = &world.materials[0];
        assert!(mat.diffuse.r > 0.0);
    }

    #[test]
    fn test_triangulate_face() {
        let mut out = Vec::new();
        triangulate_face(&[0, 1, 2, 3, 4], &mut out);
        // Pentagon → 3 triangles = 9 indices
        assert_eq!(out.len(), 9);
        assert_eq!(out, vec![0, 1, 2, 0, 2, 3, 0, 3, 4]);
    }
}
