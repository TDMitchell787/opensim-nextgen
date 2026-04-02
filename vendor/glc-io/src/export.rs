use crate::error::{IoError, Result};
use glc_core::mesh::Mesh;
use glc_core::scene::{NodeIndex, World};
use glc_core::transform::Transform;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Obj,
    Stl,
}

/// A mesh with its baked absolute transform, ready for export.
struct ExportMesh {
    name: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    tex_coords: Vec<f32>,
    indices: Vec<u32>,
    material_name: Option<String>,
    diffuse: Option<[f32; 3]>,
}

/// Export meshes from a World. If `selected_nodes` is empty, exports all visible meshes.
pub fn export_world(
    world: &World,
    selected_nodes: &HashSet<NodeIndex>,
    path: &Path,
    format: ExportFormat,
) -> Result<u32> {
    match format {
        ExportFormat::Obj => export_obj(world, selected_nodes, path),
        ExportFormat::Stl => export_stl(world, selected_nodes, path),
    }
}

/// Collect meshes for export, applying absolute transforms.
/// If `selected_nodes` is empty, collects all visible mesh nodes.
fn collect_export_meshes(world: &World, selected_nodes: &HashSet<NodeIndex>) -> Vec<ExportMesh> {
    let mut result = Vec::new();
    let export_all = selected_nodes.is_empty();

    for (i, node) in world.nodes.iter().enumerate() {
        let node_idx = NodeIndex(i);

        // Skip if not selected (when we have a selection) or not visible
        if !export_all && !selected_nodes.contains(&node_idx) {
            continue;
        }
        if !export_all && !node.visible {
            continue;
        }
        if export_all && !node.visible {
            continue;
        }

        if let Some(mesh_idx) = node.mesh_index {
            if let Some(mesh) = world.meshes.get(mesh_idx) {
                let abs_transform = world.compute_absolute_transform(node_idx);
                let exported = transform_mesh_for_export(mesh, &abs_transform);

                // Get material info
                let (mat_name, diffuse) = if !mesh.material_ranges.is_empty() {
                    let mi = mesh.material_ranges[0].material_index;
                    if let Some(mat) = world.materials.get(mi) {
                        (
                            Some(if mat.name.is_empty() {
                                format!("material_{mi}")
                            } else {
                                mat.name.clone()
                            }),
                            Some([mat.diffuse.r, mat.diffuse.g, mat.diffuse.b]),
                        )
                    } else {
                        (None, None)
                    }
                } else if let Some(mat) = world.materials.get(mesh_idx) {
                    (
                        Some(if mat.name.is_empty() {
                            format!("material_{mesh_idx}")
                        } else {
                            mat.name.clone()
                        }),
                        Some([mat.diffuse.r, mat.diffuse.g, mat.diffuse.b]),
                    )
                } else {
                    (None, None)
                };

                result.push(ExportMesh {
                    name: if node.name.is_empty() {
                        format!("object_{i}")
                    } else {
                        node.name.clone()
                    },
                    positions: exported.0,
                    normals: exported.1,
                    tex_coords: mesh.tex_coords.clone(),
                    indices: mesh.indices.clone(),
                    diffuse,
                    material_name: mat_name,
                });
            }
        }
    }

    result
}

/// Transform positions and normals by an absolute transform.
/// Returns (transformed_positions, transformed_normals).
fn transform_mesh_for_export(mesh: &Mesh, transform: &Transform) -> (Vec<f32>, Vec<f32>) {
    let mat = transform.matrix;

    if mat == glam::Mat4::IDENTITY {
        return (mesh.positions.clone(), mesh.normals.clone());
    }

    let mat3 = glam::Mat3::from_mat4(mat);
    let normal_mat = mat3.inverse().transpose();

    let mut positions = mesh.positions.clone();
    for chunk in positions.chunks_exact_mut(3) {
        let p = mat.transform_point3(glam::Vec3::new(chunk[0], chunk[1], chunk[2]));
        chunk[0] = p.x;
        chunk[1] = p.y;
        chunk[2] = p.z;
    }

    let mut normals = mesh.normals.clone();
    for chunk in normals.chunks_exact_mut(3) {
        let n = (normal_mat * glam::Vec3::new(chunk[0], chunk[1], chunk[2])).normalize();
        chunk[0] = n.x;
        chunk[1] = n.y;
        chunk[2] = n.z;
    }

    (positions, normals)
}

/// Export to Wavefront OBJ format with accompanying .mtl file.
/// Returns the number of meshes exported.
fn export_obj(world: &World, selected_nodes: &HashSet<NodeIndex>, path: &Path) -> Result<u32> {
    let meshes = collect_export_meshes(world, selected_nodes);
    if meshes.is_empty() {
        return Ok(0);
    }

    let mtl_name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
        + ".mtl";
    let mtl_path = path.with_extension("mtl");

    let mut obj_file = std::fs::File::create(path)
        .map_err(IoError::Io)?;
    let mut mtl_file = std::fs::File::create(&mtl_path)
        .map_err(IoError::Io)?;

    // OBJ header
    writeln!(obj_file, "# Exported by GLC Player")?;
    writeln!(obj_file, "mtllib {mtl_name}")?;
    writeln!(obj_file)?;

    // MTL header
    writeln!(mtl_file, "# Exported by GLC Player")?;

    let mut vertex_offset: u32 = 0;
    let mut normal_offset: u32 = 0;
    let mut texcoord_offset: u32 = 0;
    let mut written_materials: HashSet<String> = HashSet::new();

    for mesh in &meshes {
        // Write material if not already written
        if let (Some(mat_name), Some(diffuse)) = (&mesh.material_name, &mesh.diffuse) {
            if !written_materials.contains(mat_name) {
                writeln!(mtl_file)?;
                writeln!(mtl_file, "newmtl {mat_name}")?;
                writeln!(mtl_file, "Kd {} {} {}", diffuse[0], diffuse[1], diffuse[2])?;
                writeln!(mtl_file, "Ka 0.1 0.1 0.1")?;
                writeln!(mtl_file, "Ks 0.3 0.3 0.3")?;
                writeln!(mtl_file, "Ns 32.0")?;
                writeln!(mtl_file, "d 1.0")?;
                written_materials.insert(mat_name.clone());
            }
        }

        // OBJ object group
        writeln!(obj_file, "o {}", mesh.name)?;
        if let Some(mat_name) = &mesh.material_name {
            writeln!(obj_file, "usemtl {mat_name}")?;
        }

        // Vertices
        let vert_count = mesh.positions.len() / 3;
        for chunk in mesh.positions.chunks_exact(3) {
            writeln!(obj_file, "v {} {} {}", chunk[0], chunk[1], chunk[2])?;
        }

        // Normals
        let normal_count = mesh.normals.len() / 3;
        for chunk in mesh.normals.chunks_exact(3) {
            writeln!(obj_file, "vn {} {} {}", chunk[0], chunk[1], chunk[2])?;
        }

        // Texture coordinates
        let tc_count = mesh.tex_coords.len() / 2;
        for chunk in mesh.tex_coords.chunks_exact(2) {
            writeln!(obj_file, "vt {} {}", chunk[0], chunk[1])?;
        }

        // Faces (1-based indices with offsets)
        let has_tc = tc_count > 0;
        for tri in mesh.indices.chunks_exact(3) {
            let i0 = tri[0] + 1 + vertex_offset;
            let i1 = tri[1] + 1 + vertex_offset;
            let i2 = tri[2] + 1 + vertex_offset;

            if has_tc {
                let t0 = tri[0] + 1 + texcoord_offset;
                let t1 = tri[1] + 1 + texcoord_offset;
                let t2 = tri[2] + 1 + texcoord_offset;
                let n0 = tri[0] + 1 + normal_offset;
                let n1 = tri[1] + 1 + normal_offset;
                let n2 = tri[2] + 1 + normal_offset;
                writeln!(obj_file, "f {i0}/{t0}/{n0} {i1}/{t1}/{n1} {i2}/{t2}/{n2}")?;
            } else {
                let n0 = tri[0] + 1 + normal_offset;
                let n1 = tri[1] + 1 + normal_offset;
                let n2 = tri[2] + 1 + normal_offset;
                writeln!(obj_file, "f {i0}//{n0} {i1}//{n1} {i2}//{n2}")?;
            }
        }

        writeln!(obj_file)?;

        vertex_offset += vert_count as u32;
        normal_offset += normal_count as u32;
        texcoord_offset += tc_count as u32;
    }

    Ok(meshes.len() as u32)
}

/// Export to binary STL format.
/// Returns the number of meshes exported.
fn export_stl(world: &World, selected_nodes: &HashSet<NodeIndex>, path: &Path) -> Result<u32> {
    let meshes = collect_export_meshes(world, selected_nodes);
    if meshes.is_empty() {
        return Ok(0);
    }

    let mut file = std::fs::File::create(path)
        .map_err(IoError::Io)?;

    // Count total triangles
    let total_triangles: u32 = meshes
        .iter()
        .map(|m| (m.indices.len() / 3) as u32)
        .sum();

    // 80-byte header
    let mut header = [0u8; 80];
    let header_text = b"GLC Player Export";
    header[..header_text.len()].copy_from_slice(header_text);
    file.write_all(&header)?;

    // Triangle count (u32 little-endian)
    file.write_all(&total_triangles.to_le_bytes())?;

    // Write triangles: 50 bytes each (12 normal + 36 vertex + 2 attribute)
    for mesh in &meshes {
        for tri in mesh.indices.chunks_exact(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            // Compute face normal from vertices (or use stored normals)
            let p0 = glam::Vec3::new(
                mesh.positions[i0 * 3],
                mesh.positions[i0 * 3 + 1],
                mesh.positions[i0 * 3 + 2],
            );
            let p1 = glam::Vec3::new(
                mesh.positions[i1 * 3],
                mesh.positions[i1 * 3 + 1],
                mesh.positions[i1 * 3 + 2],
            );
            let p2 = glam::Vec3::new(
                mesh.positions[i2 * 3],
                mesh.positions[i2 * 3 + 1],
                mesh.positions[i2 * 3 + 2],
            );

            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            let normal = edge1.cross(edge2).normalize_or_zero();

            // Normal (12 bytes)
            file.write_all(&normal.x.to_le_bytes())?;
            file.write_all(&normal.y.to_le_bytes())?;
            file.write_all(&normal.z.to_le_bytes())?;

            // Vertex 0 (12 bytes)
            file.write_all(&p0.x.to_le_bytes())?;
            file.write_all(&p0.y.to_le_bytes())?;
            file.write_all(&p0.z.to_le_bytes())?;

            // Vertex 1 (12 bytes)
            file.write_all(&p1.x.to_le_bytes())?;
            file.write_all(&p1.y.to_le_bytes())?;
            file.write_all(&p1.z.to_le_bytes())?;

            // Vertex 2 (12 bytes)
            file.write_all(&p2.x.to_le_bytes())?;
            file.write_all(&p2.y.to_le_bytes())?;
            file.write_all(&p2.z.to_le_bytes())?;

            // Attribute byte count (2 bytes, unused)
            file.write_all(&[0u8; 2])?;
        }
    }

    Ok(meshes.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glc_core::material::Material;
    use glc_core::mesh::Mesh;
    use glc_core::scene::{SceneNode, World};
    use glc_core::transform::Transform;
    use std::collections::HashSet;

    /// Create a simple World with a single triangle mesh.
    fn make_triangle_world() -> World {
        let mut world = World::new();

        let mat = Material::default();
        world.add_material(mat);

        let mesh = Mesh {
            name: "triangle".into(),
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            tex_coords: vec![],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let mesh_idx = world.add_mesh(mesh);

        let root = world.add_node(SceneNode::new("Root"));
        let child = world.add_node(SceneNode::with_mesh("Triangle", mesh_idx));
        world.set_parent(child, root);
        world.root = Some(root);
        world
    }

    /// Create a World with two mesh nodes for selection testing.
    fn make_two_mesh_world() -> World {
        let mut world = World::new();

        let mat = Material::default();
        world.add_material(mat);

        let mesh_a = Mesh {
            name: "meshA".into(),
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            tex_coords: vec![],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let idx_a = world.add_mesh(mesh_a);

        let mesh_b = Mesh {
            name: "meshB".into(),
            positions: vec![2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            tex_coords: vec![],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let idx_b = world.add_mesh(mesh_b);

        let root = world.add_node(SceneNode::new("Root"));
        let child_a = world.add_node(SceneNode::with_mesh("A", idx_a));
        let child_b = world.add_node(SceneNode::with_mesh("B", idx_b));
        world.set_parent(child_a, root);
        world.set_parent(child_b, root);
        world.root = Some(root);
        world
    }

    #[test]
    fn test_obj_roundtrip() {
        let world = make_triangle_world();
        let dir = tempfile::tempdir().unwrap();
        let obj_path = dir.path().join("test.obj");

        let count = export_obj(&world, &HashSet::new(), &obj_path).unwrap();
        assert_eq!(count, 1);

        // Load it back
        let loaded = crate::obj::load_obj(&obj_path).unwrap();
        assert_eq!(loaded.meshes.len(), 1);
        assert_eq!(loaded.meshes[0].vertex_count(), 3);
        assert_eq!(loaded.meshes[0].face_count(), 1);
    }

    #[test]
    fn test_stl_roundtrip() {
        let world = make_triangle_world();
        let dir = tempfile::tempdir().unwrap();
        let stl_path = dir.path().join("test.stl");

        let count = export_stl(&world, &HashSet::new(), &stl_path).unwrap();
        assert_eq!(count, 1);

        // Load it back
        let loaded = crate::stl::load_stl(&stl_path).unwrap();
        // STL duplicates vertices per triangle
        assert_eq!(loaded.meshes[0].face_count(), 1);
    }

    #[test]
    fn test_export_with_transform() {
        let mut world = make_triangle_world();
        // Apply a translation to the mesh node (index 1)
        let t = Transform::from_translation(glam::Vec3::new(10.0, 0.0, 0.0));
        world.nodes[1].transform = t;

        let dir = tempfile::tempdir().unwrap();
        let obj_path = dir.path().join("transformed.obj");

        export_obj(&world, &HashSet::new(), &obj_path).unwrap();

        // Reload and check that positions are transformed
        let loaded = crate::obj::load_obj(&obj_path).unwrap();
        let pos = &loaded.meshes[0].positions;
        // First vertex should be at (10.0, 0.0, 0.0)
        assert!((pos[0] - 10.0).abs() < 1e-4, "x = {}", pos[0]);
    }

    #[test]
    fn test_export_selected_subset() {
        let world = make_two_mesh_world();
        // Select only the first mesh node (index 1, node "A")
        let mut selected = HashSet::new();
        selected.insert(NodeIndex(1));

        let dir = tempfile::tempdir().unwrap();
        let stl_path = dir.path().join("selected.stl");

        let count = export_stl(&world, &selected, &stl_path).unwrap();
        assert_eq!(count, 1); // only 1 mesh exported

        // Load and verify only one triangle
        let loaded = crate::stl::load_stl(&stl_path).unwrap();
        assert_eq!(loaded.meshes[0].face_count(), 1);
    }
}
