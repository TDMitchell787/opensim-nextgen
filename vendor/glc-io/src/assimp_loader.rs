//! Assimp-based loader for 40+ 3D formats via russimp-ng.
//!
//! Converts assimp's Scene → our glc_core World. This module is only
//! compiled when the `assimp` feature is enabled.

use crate::error::{IoError, Result};
use glc_core::material::Material as GlcMaterial;
use glc_core::mesh::Mesh as GlcMesh;
use glc_core::scene::{NodeIndex, SceneNode, World};
use glc_core::transform::Transform;
use glc_core::types::Color4f;
use glam::Mat4;
use russimp_ng::material::{Material as AssimpMaterial, PropertyTypeInfo};
use russimp_ng::scene::{PostProcess, Scene};
use std::path::Path;
use std::rc::Rc;

/// Load any assimp-supported format from a file path.
pub fn load_assimp(path: &Path) -> Result<World> {
    let path_str = path.to_str().ok_or_else(|| {
        IoError::ParseError(format!("Invalid path: {}", path.display()))
    })?;

    let scene = Scene::from_file(
        path_str,
        vec![
            PostProcess::Triangulate,
            PostProcess::JoinIdenticalVertices,
            PostProcess::GenerateSmoothNormals,
            PostProcess::FixInfacingNormals,
        ],
    )
    .map_err(|e| IoError::ParseError(format!("assimp: {e}")))?;

    build_world(&scene)
}

/// Load any assimp-supported format from in-memory bytes.
/// `name` should include the file extension for format detection (e.g. "model.fbx").
pub fn load_assimp_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let scene = Scene::from_buffer(
        bytes,
        vec![
            PostProcess::Triangulate,
            PostProcess::JoinIdenticalVertices,
            PostProcess::GenerateSmoothNormals,
            PostProcess::FixInfacingNormals,
        ],
        ext,
    )
    .map_err(|e| IoError::ParseError(format!("assimp ({name}): {e}")))?;

    build_world(&scene)
}

fn build_world(scene: &Scene) -> Result<World> {
    let mut world = World::new();

    // Convert materials
    for mat in &scene.materials {
        world.add_material(convert_material(mat));
    }
    if world.materials.is_empty() {
        world.add_material(GlcMaterial::default());
    }

    // Convert meshes
    for mesh in &scene.meshes {
        world.add_mesh(convert_mesh(mesh));
    }

    // Build scene tree from root node
    let root_node = SceneNode::new("Root");
    let root_idx = world.add_node(root_node);
    world.root = Some(root_idx);

    if let Some(ref root) = scene.root {
        build_node(&mut world, root, root_idx);
    }

    Ok(world)
}

fn convert_material(mat: &AssimpMaterial) -> GlcMaterial {
    let diffuse = get_color_property(mat, "$clr.diffuse")
        .unwrap_or(Color4f::new(0.8, 0.8, 0.8, 1.0));
    let ambient = get_color_property(mat, "$clr.ambient")
        .unwrap_or(Color4f::new(0.2, 0.2, 0.2, 1.0));
    let specular = get_color_property(mat, "$clr.specular")
        .unwrap_or(Color4f::new(1.0, 1.0, 1.0, 1.0));
    let emissive = get_color_property(mat, "$clr.emissive")
        .unwrap_or(Color4f::BLACK);

    let shininess = get_float_property(mat, "$mat.shininess").unwrap_or(50.0);
    let opacity = get_float_property(mat, "$mat.opacity").unwrap_or(1.0);

    let name = get_string_property(mat, "?mat.name")
        .unwrap_or_else(|| "assimp_material".to_string());

    GlcMaterial {
        name,
        ambient,
        diffuse,
        specular,
        emissive,
        shininess: shininess.max(1.0),
        opacity,
        texture_path: None,
        ..GlcMaterial::default()
    }
}

fn get_color_property(mat: &AssimpMaterial, key: &str) -> Option<Color4f> {
    for prop in &mat.properties {
        if prop.key == key {
            if let PropertyTypeInfo::FloatArray(ref floats) = prop.data {
                if floats.len() >= 3 {
                    let a = if floats.len() >= 4 { floats[3] } else { 1.0 };
                    return Some(Color4f::new(floats[0], floats[1], floats[2], a));
                }
            }
        }
    }
    None
}

fn get_float_property(mat: &AssimpMaterial, key: &str) -> Option<f32> {
    for prop in &mat.properties {
        if prop.key == key {
            if let PropertyTypeInfo::FloatArray(ref floats) = prop.data {
                if !floats.is_empty() {
                    return Some(floats[0]);
                }
            }
        }
    }
    None
}

fn get_string_property(mat: &AssimpMaterial, key: &str) -> Option<String> {
    for prop in &mat.properties {
        if prop.key == key {
            if let PropertyTypeInfo::String(ref s) = prop.data {
                return Some(s.clone());
            }
        }
    }
    None
}

fn convert_mesh(mesh: &russimp_ng::mesh::Mesh) -> GlcMesh {
    let mut positions = Vec::with_capacity(mesh.vertices.len() * 3);
    for v in &mesh.vertices {
        positions.extend_from_slice(&[v.x, v.y, v.z]);
    }

    let mut normals = Vec::with_capacity(mesh.normals.len() * 3);
    for n in &mesh.normals {
        normals.extend_from_slice(&[n.x, n.y, n.z]);
    }

    // First UV channel
    let mut tex_coords = Vec::new();
    if let Some(Some(ref uvs)) = mesh.texture_coords.first() {
        tex_coords.reserve(uvs.len() * 2);
        for tc in uvs {
            tex_coords.extend_from_slice(&[tc.x, tc.y]);
        }
    }

    // Collect face indices (already triangulated by PostProcess::Triangulate)
    let mut indices = Vec::new();
    for face in &mesh.faces {
        for &idx in &face.0 {
            indices.push(idx);
        }
    }

    let mut m = GlcMesh {
        name: mesh.name.clone(),
        positions,
        normals,
        tex_coords,
        indices,
        ..GlcMesh::default()
    };

    // Assign material range if the mesh has a material index
    if !m.indices.is_empty() {
        m.material_ranges.push(glc_core::mesh::MaterialRange {
            material_index: mesh.material_index as usize,
            start: 0,
            count: m.indices.len() as u32,
        });
    }

    m
}

/// Convert assimp's row-major Matrix4x4 to glam's column-major Mat4.
fn convert_matrix(m: &russimp_ng::Matrix4x4) -> Mat4 {
    // assimp stores row-major: a1,a2,a3,a4 is first row
    // glam Mat4::from_cols_array expects column-major
    Mat4::from_cols_array(&[
        m.a1, m.b1, m.c1, m.d1, // column 0
        m.a2, m.b2, m.c2, m.d2, // column 1
        m.a3, m.b3, m.c3, m.d3, // column 2
        m.a4, m.b4, m.c4, m.d4, // column 3
    ])
}

fn build_node(
    world: &mut World,
    ai_node: &Rc<russimp_ng::node::Node>,
    parent: NodeIndex,
) {
    let transform = Transform::new(convert_matrix(&ai_node.transformation));

    if ai_node.meshes.is_empty() {
        // Group node (no mesh)
        let mut scene_node = SceneNode::new(ai_node.name.clone());
        scene_node.transform = transform;
        let node_idx = world.add_node(scene_node);
        world.set_parent(node_idx, parent);

        for child in ai_node.children.borrow().iter() {
            build_node(world, child, node_idx);
        }
    } else if ai_node.meshes.len() == 1 {
        // Single mesh node
        let mut scene_node =
            SceneNode::with_mesh(ai_node.name.clone(), ai_node.meshes[0] as usize);
        scene_node.transform = transform;
        let node_idx = world.add_node(scene_node);
        world.set_parent(node_idx, parent);

        for child in ai_node.children.borrow().iter() {
            build_node(world, child, node_idx);
        }
    } else {
        // Multiple meshes: create group node with child per mesh
        let mut group = SceneNode::new(ai_node.name.clone());
        group.transform = transform;
        let group_idx = world.add_node(group);
        world.set_parent(group_idx, parent);

        for (i, &mesh_idx) in ai_node.meshes.iter().enumerate() {
            let prim_node = SceneNode::with_mesh(
                format!("{}_{}", ai_node.name, i),
                mesh_idx as usize,
            );
            let prim_idx = world.add_node(prim_node);
            world.set_parent(prim_idx, group_idx);
        }

        for child in ai_node.children.borrow().iter() {
            build_node(world, child, group_idx);
        }
    }
}

/// List of additional file extensions supported through assimp
/// (excluding formats we already handle natively: obj, stl, off, 3dxml, gltf, glb, ply).
pub fn assimp_extensions() -> &'static [&'static str] {
    &[
        "fbx", "dae", "3ds", "blend", "dxf", "x", "lwo", "lws", "md2", "md3",
        "md5mesh", "mdl", "ase", "b3d", "bvh", "ifc", "xgl", "zgl", "smd",
        "vta", "ogex", "3d", "x3d", "x3db", "ac", "ms3d", "cob", "scn",
        "q3o", "q3s", "raw", "nff", "ter", "hmp", "ndo", "3mf",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_matrix_identity() {
        let ai_mat = russimp_ng::Matrix4x4 {
            a1: 1.0, a2: 0.0, a3: 0.0, a4: 0.0,
            b1: 0.0, b2: 1.0, b3: 0.0, b4: 0.0,
            c1: 0.0, c2: 0.0, c3: 1.0, c4: 0.0,
            d1: 0.0, d2: 0.0, d3: 0.0, d4: 1.0,
        };
        let mat = convert_matrix(&ai_mat);
        assert_eq!(mat, Mat4::IDENTITY);
    }

    #[test]
    fn test_assimp_extensions_not_empty() {
        let exts = assimp_extensions();
        assert!(exts.len() > 20);
        assert!(exts.contains(&"fbx"));
        assert!(exts.contains(&"dae"));
        assert!(exts.contains(&"3ds"));
    }

    #[test]
    fn test_load_stl_from_buffer() {
        // Use assimp to load an STL from memory (proves the integration works)
        let stl_data = b"solid test\n\
            facet normal 0 0 1\n\
              outer loop\n\
                vertex 0 0 0\n\
                vertex 1 0 0\n\
                vertex 0 1 0\n\
              endloop\n\
            endfacet\n\
            endsolid test";

        let world = load_assimp_from_bytes(stl_data, "test.stl").unwrap();
        assert!(!world.meshes.is_empty());
        assert!(world.meshes[0].positions.len() >= 9); // at least 3 vertices
        assert!(world.root.is_some());
    }
}
