use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::{MaterialRange, Mesh};
use glc_core::scene::{NodeIndex, SceneNode, World};
use glc_core::transform::Transform;
use glc_core::types::Color4f;
use glam::{Mat4, Quat, Vec3};
use std::path::Path;

/// Load a glTF 2.0 file (.gltf or .glb) from disk.
pub fn load_gltf(path: &Path) -> Result<World> {
    let (document, buffers, _images) =
        gltf::import(path).map_err(|e| IoError::ParseError(format!("glTF: {e}")))?;
    build_world(&document, &buffers)
}

/// Load a glTF 2.0 file from in-memory bytes.
pub fn load_gltf_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let (document, buffers, _images) = gltf::import_slice(bytes)
        .map_err(|e| IoError::ParseError(format!("glTF ({name}): {e}")))?;
    build_world(&document, &buffers)
}

fn build_world(document: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Result<World> {
    let mut world = World::new();

    // Load all materials
    for mat in document.materials() {
        world.add_material(convert_material(&mat));
    }

    // Load all meshes (each primitive becomes a separate Mesh)
    // Map: (gltf mesh index, primitive index) → our mesh index
    let mut mesh_map: Vec<Vec<usize>> = Vec::new();
    for gltf_mesh in document.meshes() {
        let mut prim_indices = Vec::new();
        for primitive in gltf_mesh.primitives() {
            match load_primitive(&primitive, buffers) {
                Ok(mesh) => {
                    let mat_idx = primitive.material().index();
                    let mut m = mesh;
                    if let Some(mi) = mat_idx {
                        if !m.indices.is_empty() {
                            m.material_ranges.push(MaterialRange {
                                material_index: mi,
                                start: 0,
                                count: m.indices.len() as u32,
                            });
                        }
                    }
                    m.name = gltf_mesh.name().unwrap_or("mesh").to_string();
                    let idx = world.add_mesh(m);
                    prim_indices.push(idx);
                }
                Err(e) => {
                    log::warn!("Skipping primitive: {e}");
                }
            }
        }
        mesh_map.push(prim_indices);
    }

    // Build scene tree from the default scene (or first scene)
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next());

    let root_node = SceneNode::new("Root");
    let root_idx = world.add_node(root_node);
    world.root = Some(root_idx);

    if let Some(scene) = scene {
        for node in scene.nodes() {
            build_node(&mut world, &node, root_idx, &mesh_map);
        }
    }

    // If no materials were loaded, add a default
    if world.materials.is_empty() {
        world.add_material(Material::default());
    }

    Ok(world)
}

/// Convert a glTF PBR material to our Blinn-Phong approximation.
fn convert_material(mat: &gltf::Material) -> Material {
    let pbr = mat.pbr_metallic_roughness();
    let base_color = pbr.base_color_factor();
    let roughness = pbr.roughness_factor();
    let metallic = pbr.metallic_factor();

    // Approximate PBR → Blinn-Phong
    let diffuse = Color4f::new(base_color[0], base_color[1], base_color[2], base_color[3]);
    let ambient = Color4f::new(
        base_color[0] * 0.2,
        base_color[1] * 0.2,
        base_color[2] * 0.2,
        1.0,
    );

    // Metallic surfaces have specular color matching base color
    let spec_strength = if metallic > 0.5 { 0.8 } else { 0.3 };
    let specular = if metallic > 0.5 {
        Color4f::new(
            base_color[0] * spec_strength,
            base_color[1] * spec_strength,
            base_color[2] * spec_strength,
            1.0,
        )
    } else {
        Color4f::new(spec_strength, spec_strength, spec_strength, 1.0)
    };

    // Roughness → shininess (inverse mapping)
    let shininess = ((1.0 - roughness) * 128.0).max(1.0);

    Material {
        name: mat.name().unwrap_or("material").to_string(),
        ambient,
        diffuse,
        specular,
        emissive: Color4f::BLACK,
        shininess,
        opacity: base_color[3],
        texture_path: None,
        ..Material::default()
    }
}

/// Load a single glTF primitive into our Mesh format.
fn load_primitive(
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
) -> Result<Mesh> {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // Positions (required)
    let positions_iter = reader
        .read_positions()
        .ok_or_else(|| IoError::ParseError("glTF primitive missing positions".into()))?;
    let mut positions = Vec::new();
    for p in positions_iter {
        positions.extend_from_slice(&p);
    }

    // Normals (optional, compute flat normals if missing)
    let mut normals = Vec::new();
    if let Some(normals_iter) = reader.read_normals() {
        for n in normals_iter {
            normals.extend_from_slice(&n);
        }
    }

    // Texture coordinates (optional, first set)
    let mut tex_coords = Vec::new();
    if let Some(tex_iter) = reader.read_tex_coords(0) {
        for tc in tex_iter.into_f32() {
            tex_coords.extend_from_slice(&tc);
        }
    }

    // Indices
    let indices: Vec<u32>;
    if let Some(idx_reader) = reader.read_indices() {
        indices = idx_reader.into_u32().collect();
    } else {
        // Generate sequential indices for non-indexed geometry
        let vertex_count = positions.len() / 3;
        indices = (0..vertex_count as u32).collect();
    }

    // Generate normals if not present
    if normals.is_empty() {
        normals = compute_flat_normals(&positions, &indices);
    }

    Ok(Mesh {
        name: String::new(),
        positions,
        normals,
        tex_coords,
        indices,
        material_ranges: Vec::new(),
        ..Mesh::default()
    })
}

/// Compute flat (face) normals when they're not provided.
fn compute_flat_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
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

    // Normalize
    for chunk in normals.chunks_exact_mut(3) {
        let n = Vec3::new(chunk[0], chunk[1], chunk[2]);
        let len = n.length();
        if len > 1e-8 {
            chunk[0] = n.x / len;
            chunk[1] = n.y / len;
            chunk[2] = n.z / len;
        } else {
            chunk[0] = 0.0;
            chunk[1] = 0.0;
            chunk[2] = 1.0;
        }
    }

    normals
}

/// Recursively build scene tree nodes from glTF node hierarchy.
fn build_node(
    world: &mut World,
    node: &gltf::Node,
    parent: NodeIndex,
    mesh_map: &[Vec<usize>],
) {
    let fallback = format!("Node_{}", node.index());
    let name = node.name().unwrap_or(&fallback);

    // Get transform
    let transform = match node.transform() {
        gltf::scene::Transform::Matrix { matrix } => {
            let cols = matrix;
            Transform::new(Mat4::from_cols_array_2d(&cols))
        }
        gltf::scene::Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => {
            let t = Vec3::from(translation);
            let r = Quat::from_array(rotation);
            let s = Vec3::from(scale);
            Transform::from_trs(t, r, s)
        }
    };

    if let Some(mesh) = node.mesh() {
        let mesh_idx = mesh.index();
        if let Some(prim_indices) = mesh_map.get(mesh_idx) {
            if prim_indices.len() == 1 {
                // Single primitive: create one node with mesh
                let mut scene_node = SceneNode::with_mesh(name.to_string(), prim_indices[0]);
                scene_node.transform = transform;
                let node_idx = world.add_node(scene_node);
                world.set_parent(node_idx, parent);

                // Recurse children under this node
                for child in node.children() {
                    build_node(world, &child, node_idx, mesh_map);
                }
                return;
            } else {
                // Multiple primitives: create a group node, then child nodes per primitive
                let mut group = SceneNode::new(name.to_string());
                group.transform = transform;
                let group_idx = world.add_node(group);
                world.set_parent(group_idx, parent);

                for (i, &mi) in prim_indices.iter().enumerate() {
                    let prim_node =
                        SceneNode::with_mesh(format!("{name}_prim{i}"), mi);
                    let prim_idx = world.add_node(prim_node);
                    world.set_parent(prim_idx, group_idx);
                }

                // Recurse children under group node
                for child in node.children() {
                    build_node(world, &child, group_idx, mesh_map);
                }
                return;
            }
        }
    }

    // Node without mesh — create a group node
    let mut scene_node = SceneNode::new(name.to_string());
    scene_node.transform = transform;
    let node_idx = world.add_node(scene_node);
    world.set_parent(node_idx, parent);

    for child in node.children() {
        build_node(world, &child, node_idx, mesh_map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_flat_normals_triangle() {
        // Triangle in XY plane
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let indices = vec![0, 1, 2];
        let normals = compute_flat_normals(&positions, &indices);
        assert_eq!(normals.len(), 9);
        // Normal should point in +Z direction
        assert!((normals[2] - 1.0).abs() < 0.01); // z component of first vertex normal
        assert!((normals[5] - 1.0).abs() < 0.01);
        assert!((normals[8] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_material_default_pbr() {
        // Create a minimal glTF document with default material
        let glb_bytes = minimal_glb_with_material();
        let (doc, _buffers, _images) = gltf::import_slice(&glb_bytes).unwrap();
        if let Some(mat) = doc.materials().next() {
            let m = convert_material(&mat);
            // Default PBR: white base color, roughness 1.0, metallic 1.0
            assert!(m.diffuse.r > 0.0);
            assert!(m.shininess >= 1.0);
        }
    }

    /// Create a minimal GLB with one triangle and a material.
    fn minimal_glb_with_material() -> Vec<u8> {
        // Minimal valid GLB: header + JSON chunk + BIN chunk
        let json = r#"{
            "asset":{"version":"2.0"},
            "scene":0,
            "scenes":[{"nodes":[0]}],
            "nodes":[{"mesh":0}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"material":0}]}],
            "materials":[{"pbrMetallicRoughness":{"baseColorFactor":[0.8,0.2,0.1,1.0],"metallicFactor":0.0,"roughnessFactor":0.5}}],
            "accessors":[
                {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","max":[1,1,0],"min":[0,0,0]},
                {"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}
            ],
            "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":36},
                {"buffer":0,"byteOffset":36,"byteLength":6}
            ],
            "buffers":[{"byteLength":44}]
        }"#;

        // Pad JSON to 4-byte alignment
        let json_bytes = json.as_bytes();
        let json_pad = (4 - (json_bytes.len() % 4)) % 4;
        let json_chunk_len = json_bytes.len() + json_pad;

        // BIN data: 3 vertices (36 bytes) + 3 indices u16 (6 bytes) + 2 pad = 44 bytes
        let positions: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let indices: [u16; 3] = [0, 1, 2];
        let mut bin = Vec::new();
        for f in &positions {
            bin.extend_from_slice(&f.to_le_bytes());
        }
        for i in &indices {
            bin.extend_from_slice(&i.to_le_bytes());
        }
        // Pad to 4 bytes
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let bin_chunk_len = bin.len();

        // GLB header: magic + version + length
        let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;
        let mut glb = Vec::with_capacity(total_len);

        // Header
        glb.extend_from_slice(b"glTF"); // magic
        glb.extend_from_slice(&2u32.to_le_bytes()); // version
        glb.extend_from_slice(&(total_len as u32).to_le_bytes()); // total length

        // JSON chunk
        glb.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(json_bytes);
        for _ in 0..json_pad {
            glb.push(b' ');
        }

        // BIN chunk
        glb.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);

        glb
    }

    #[test]
    fn test_load_glb_minimal() {
        let glb = minimal_glb_with_material();
        let world = load_gltf_from_bytes(&glb, "test.glb").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert_eq!(world.meshes[0].positions.len(), 9); // 3 vertices * 3 floats
        assert_eq!(world.meshes[0].indices.len(), 3);
        assert_eq!(world.materials.len(), 1);
        assert!(world.root.is_some());
        // Scene tree: Root → Node_0 (with mesh)
        assert!(world.nodes.len() >= 2);
    }

    #[test]
    fn test_load_glb_scene_tree() {
        let glb = minimal_glb_with_material();
        let world = load_gltf_from_bytes(&glb, "test.glb").unwrap();
        let root = world.root.unwrap();
        assert_eq!(world.nodes[root.0].children.len(), 1);
        let child = world.nodes[root.0].children[0];
        assert!(world.nodes[child.0].mesh_index.is_some());
    }
}
