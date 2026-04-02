use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::{MaterialRange, Mesh};
use glc_core::scene::{SceneNode, World};
use glc_core::types::EntityId;
use std::io::BufRead;
use std::path::Path;

/// Load an OFF file from disk, producing a `World`.
pub fn load_off(path: &Path) -> Result<World> {
    let path_str = path.to_string_lossy();
    let file = std::fs::File::open(path)
        .map_err(|e| IoError::FileNotFound(format!("{path_str}: {e}")))?;
    let reader = std::io::BufReader::new(file);
    load_off_from_reader(reader, &path_str)
}

/// Load an OFF model from in-memory bytes (for web).
pub fn load_off_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let reader = std::io::BufReader::new(std::io::Cursor::new(bytes));
    load_off_from_reader(reader, name)
}

fn load_off_from_reader<R: BufRead>(reader: R, name: &str) -> Result<World> {
    let mut lines = reader.lines();

    // Parse header: "OFF", "COFF", or "4OFF"
    let mut has_vertex_colors = false;
    let header = next_data_line(&mut lines, name)?;
    let header_trimmed = header.trim();
    match header_trimmed {
        "OFF" => {}
        "COFF" => has_vertex_colors = true,
        "4OFF" => {} // treat as regular OFF, ignore 4th coordinate
        _ => {
            // Some OFF files have counts on the same line as "OFF"
            if !header_trimmed.starts_with("OFF") && !header_trimmed.starts_with("COFF") {
                return Err(IoError::ParseError(format!(
                    "{name}: expected OFF header, got: {header_trimmed}"
                )));
            }
            if header_trimmed.starts_with("COFF") {
                has_vertex_colors = true;
            }
        }
    }

    // Parse counts line: num_vertices num_faces num_edges
    let counts_line = if header_trimmed.split_whitespace().count() > 1
        && (header_trimmed.starts_with("OFF") || header_trimmed.starts_with("COFF"))
    {
        // Counts on the same line as header (e.g. "OFF 8 6 12")
        let after = header_trimmed
            .strip_prefix("COFF")
            .or_else(|| header_trimmed.strip_prefix("OFF"))
            .unwrap_or(header_trimmed);
        after.trim().to_string()
    } else {
        next_data_line(&mut lines, name)?
    };

    let counts: Vec<usize> = counts_line
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if counts.len() < 2 {
        return Err(IoError::ParseError(format!(
            "{name}: expected vertex/face counts, got: {counts_line}"
        )));
    }
    let num_vertices = counts[0];
    let num_faces = counts[1];

    // Parse vertices
    let mut positions = Vec::with_capacity(num_vertices * 3);
    let mut vertex_colors: Vec<[f32; 4]> = Vec::new();
    if has_vertex_colors {
        vertex_colors.reserve(num_vertices);
    }

    for i in 0..num_vertices {
        let line = next_data_line(&mut lines, name)
            .map_err(|_| IoError::ParseError(format!("{name}: unexpected EOF at vertex {i}")))?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(IoError::ParseError(format!(
                "{name}: vertex {i} needs at least 3 coordinates"
            )));
        }
        let x: f32 = parts[0].parse().map_err(|_| {
            IoError::ParseError(format!("{name}: bad vertex coordinate at {i}"))
        })?;
        let y: f32 = parts[1].parse().map_err(|_| {
            IoError::ParseError(format!("{name}: bad vertex coordinate at {i}"))
        })?;
        let z: f32 = parts[2].parse().map_err(|_| {
            IoError::ParseError(format!("{name}: bad vertex coordinate at {i}"))
        })?;
        positions.push(x);
        positions.push(y);
        positions.push(z);

        // COFF: vertex colors after coordinates (R G B [A] as 0-255 or 0.0-1.0)
        if has_vertex_colors && parts.len() >= 6 {
            let r: f32 = parts[3].parse().unwrap_or(200.0);
            let g: f32 = parts[4].parse().unwrap_or(200.0);
            let b: f32 = parts[5].parse().unwrap_or(200.0);
            let a: f32 = if parts.len() >= 7 {
                parts[6].parse().unwrap_or(255.0)
            } else {
                255.0
            };
            // Normalize: if values > 1.0, assume 0-255 range
            let scale = if r > 1.0 || g > 1.0 || b > 1.0 {
                1.0 / 255.0
            } else {
                1.0
            };
            vertex_colors.push([r * scale, g * scale, b * scale, a * scale]);
        }
    }

    // Parse faces and triangulate polygons
    let mut indices: Vec<u32> = Vec::with_capacity(num_faces * 3);
    for i in 0..num_faces {
        let line = next_data_line(&mut lines, name)
            .map_err(|_| IoError::ParseError(format!("{name}: unexpected EOF at face {i}")))?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let n_verts: usize = parts[0].parse().map_err(|_| {
            IoError::ParseError(format!("{name}: bad face vertex count at {i}"))
        })?;
        if parts.len() < 1 + n_verts {
            return Err(IoError::ParseError(format!(
                "{name}: face {i} declares {n_verts} vertices but has fewer"
            )));
        }

        let face_indices: Vec<u32> = parts[1..1 + n_verts]
            .iter()
            .map(|s| {
                s.parse::<u32>().map_err(|_| {
                    IoError::ParseError(format!("{name}: bad face index at {i}"))
                })
            })
            .collect::<Result<_>>()?;

        // Fan triangulation for polygons with >3 vertices
        if face_indices.len() >= 3 {
            for j in 1..face_indices.len() - 1 {
                indices.push(face_indices[0]);
                indices.push(face_indices[j]);
                indices.push(face_indices[j + 1]);
            }
        }
    }

    // Compute normals from face data
    let num_verts = positions.len() / 3;
    let mut normals = vec![0.0f32; num_verts * 3];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let (px, py, pz) = (
            positions[i0 * 3],
            positions[i0 * 3 + 1],
            positions[i0 * 3 + 2],
        );
        let (qx, qy, qz) = (
            positions[i1 * 3],
            positions[i1 * 3 + 1],
            positions[i1 * 3 + 2],
        );
        let (rx, ry, rz) = (
            positions[i2 * 3],
            positions[i2 * 3 + 1],
            positions[i2 * 3 + 2],
        );
        // Cross product of edges
        let (ux, uy, uz) = (qx - px, qy - py, qz - pz);
        let (vx, vy, vz) = (rx - px, ry - py, rz - pz);
        let nx = uy * vz - uz * vy;
        let ny = uz * vx - ux * vz;
        let nz = ux * vy - uy * vx;

        for &idx in tri {
            let base = idx as usize * 3;
            normals[base] += nx;
            normals[base + 1] += ny;
            normals[base + 2] += nz;
        }
    }
    // Normalize
    for chunk in normals.chunks_exact_mut(3) {
        let len = (chunk[0] * chunk[0] + chunk[1] * chunk[1] + chunk[2] * chunk[2]).sqrt();
        if len > 1e-8 {
            chunk[0] /= len;
            chunk[1] /= len;
            chunk[2] /= len;
        }
    }

    let total_indices = indices.len() as u32;
    let mesh = Mesh {
        id: EntityId::new(),
        name: name.to_string(),
        positions,
        normals,
        tex_coords: Vec::new(),
        indices,
        line_indices: Vec::new(),
        material_ranges: vec![MaterialRange {
            material_index: 0,
            start: 0,
            count: total_indices,
        }],
        lod: 0,
    };

    let mut world = World::new();
    world.source_path = Some(name.to_string());

    // Use vertex color as diffuse if COFF, otherwise default material
    if !vertex_colors.is_empty() {
        // Average vertex colors for a single material (simplified)
        let (mut ar, mut ag, mut ab) = (0.0f32, 0.0f32, 0.0f32);
        for c in &vertex_colors {
            ar += c[0];
            ag += c[1];
            ab += c[2];
        }
        let n = vertex_colors.len() as f32;
        let mat = Material {
            diffuse: glc_core::types::Color4f {
                r: ar / n,
                g: ag / n,
                b: ab / n,
                a: 1.0,
            },
            ..Material::default()
        };
        world.add_material(mat);
    } else {
        world.add_material(Material::default());
    }

    let mesh_idx = world.add_mesh(mesh);
    let root = world.add_node(SceneNode::with_mesh(name, mesh_idx));
    world.root = Some(root);

    Ok(world)
}

/// Read next non-comment, non-empty line.
fn next_data_line<R: BufRead>(
    lines: &mut std::io::Lines<R>,
    name: &str,
) -> Result<String> {
    for line in lines {
        let line = line.map_err(|e| IoError::ParseError(format!("{name}: {e}")))?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return Ok(trimmed);
    }
    Err(IoError::ParseError(format!("{name}: unexpected end of file")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_off_cube() {
        let off_data = b"\
OFF
8 6 12
-0.5 -0.5 -0.5
 0.5 -0.5 -0.5
 0.5  0.5 -0.5
-0.5  0.5 -0.5
-0.5 -0.5  0.5
 0.5 -0.5  0.5
 0.5  0.5  0.5
-0.5  0.5  0.5
4 0 1 2 3
4 4 7 6 5
4 0 4 5 1
4 2 6 7 3
4 0 3 7 4
4 1 5 6 2
";
        let world = load_off_from_bytes(off_data, "cube.off").unwrap();
        assert_eq!(world.meshes.len(), 1);
        let mesh = &world.meshes[0];
        assert_eq!(mesh.vertex_count(), 8);
        // 6 quads triangulated = 12 triangles = 36 indices
        assert_eq!(mesh.face_count(), 12);
    }

    #[test]
    fn test_load_off_triangle() {
        let off_data = b"\
OFF
3 1 0
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
3 0 1 2
";
        let world = load_off_from_bytes(off_data, "tri.off").unwrap();
        assert_eq!(world.meshes[0].vertex_count(), 3);
        assert_eq!(world.meshes[0].face_count(), 1);
    }

    #[test]
    fn test_load_coff() {
        let off_data = b"\
COFF
3 1 0
0.0 0.0 0.0 255 0 0
1.0 0.0 0.0 0 255 0
0.0 1.0 0.0 0 0 255
3 0 1 2
";
        let world = load_off_from_bytes(off_data, "colored.off").unwrap();
        assert_eq!(world.meshes.len(), 1);
        // Material should have averaged color
        assert_eq!(world.materials.len(), 1);
    }

    #[test]
    fn test_load_off_with_comments() {
        let off_data = b"\
# This is a comment
OFF
# Another comment
3 1 0

0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
# Face definition
3 0 1 2
";
        let world = load_off_from_bytes(off_data, "comments.off").unwrap();
        assert_eq!(world.meshes[0].face_count(), 1);
    }
}
