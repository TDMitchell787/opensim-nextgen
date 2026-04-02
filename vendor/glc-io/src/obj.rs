use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::{MaterialRange, Mesh};
use glc_core::scene::{SceneNode, World};
use glc_core::types::{Color4f, EntityId};
use std::path::Path;

/// Load an OBJ file from disk, producing a `World`.
pub fn load_obj(path: &Path) -> Result<World> {
    let path_str = path.to_string_lossy().into_owned();
    let parent = path.parent().unwrap_or(Path::new("."));

    let (models, materials_result) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .map_err(|e| IoError::ObjError(format!("{path_str}: {e}")))?;

    let mut world = World::new();
    world.source_path = Some(path_str.clone());

    // Load materials from MTL file
    let tobj_materials = match materials_result {
        Ok(mats) => mats,
        Err(e) => {
            log::warn!("Failed to load MTL for {path_str}: {e}");
            Vec::new()
        }
    };

    for mat in &tobj_materials {
        world.add_material(convert_material(mat, parent));
    }

    // If no materials loaded, add a default one
    let has_materials = !world.materials.is_empty();
    if !has_materials {
        world.add_material(Material::default());
    }

    // Create root node
    let root = world.add_node(SceneNode::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Root".to_string()),
    ));
    world.root = Some(root);

    // Convert each model to a mesh + scene node
    for model in &models {
        let mesh = convert_mesh(model, has_materials);
        let mesh_idx = world.add_mesh(mesh);
        let child = world.add_node(SceneNode::with_mesh(&model.name, mesh_idx));
        world.set_parent(child, root);
    }

    Ok(world)
}

/// Load an OBJ model from in-memory bytes (for web).
pub fn load_obj_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    let mut reader = std::io::BufReader::new(bytes);
    let (models, _materials_result) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        // No MTL loader for in-memory — would need separate bytes
        |_mtl_path| Err(tobj::LoadError::GenericFailure),
    )
    .map_err(|e| IoError::ObjError(format!("{name}: {e}")))?;

    let mut world = World::new();
    world.source_path = Some(name.to_string());
    world.add_material(Material::default());

    let root = world.add_node(SceneNode::new(name));
    world.root = Some(root);

    for model in &models {
        let mesh = convert_mesh(model, false);
        let mesh_idx = world.add_mesh(mesh);
        let child = world.add_node(SceneNode::with_mesh(&model.name, mesh_idx));
        world.set_parent(child, root);
    }

    Ok(world)
}

fn convert_material(mat: &tobj::Material, parent_dir: &Path) -> Material {
    let diffuse = mat
        .diffuse
        .map(|d| Color4f::rgb(d[0], d[1], d[2]))
        .unwrap_or(Color4f::new(0.8, 0.8, 0.8, 1.0));
    let ambient = mat
        .ambient
        .map(|a| Color4f::rgb(a[0], a[1], a[2]))
        .unwrap_or(Color4f::new(0.2, 0.2, 0.2, 1.0));
    let specular = mat
        .specular
        .map(|s| Color4f::rgb(s[0], s[1], s[2]))
        .unwrap_or(Color4f::new(1.0, 1.0, 1.0, 1.0));

    let opacity = mat.dissolve.unwrap_or(1.0);
    let shininess = mat.shininess.unwrap_or(50.0);

    let texture_path = mat.diffuse_texture.as_ref().and_then(|t| {
        if t.is_empty() {
            None
        } else {
            Some(parent_dir.join(t).to_string_lossy().into_owned())
        }
    });

    let texture_data = texture_path.as_ref().and_then(|p| {
        crate::texture::load_texture_from_file(Path::new(p))
    });

    Material {
        id: EntityId::new(),
        name: mat.name.clone(),
        ambient,
        diffuse: Color4f::new(diffuse.r, diffuse.g, diffuse.b, opacity),
        specular,
        emissive: Color4f::BLACK,
        shininess,
        opacity,
        texture_path,
        texture_data,
        is_modified: false,
    }
}

fn convert_mesh(model: &tobj::Model, has_external_materials: bool) -> Mesh {
    let tobj_mesh = &model.mesh;

    // Determine material ranges
    let material_ranges = if !tobj_mesh.face_arities.is_empty() || tobj_mesh.material_id.is_some() {
        // Single material for the whole mesh
        let mat_idx = if has_external_materials {
            tobj_mesh.material_id.unwrap_or(0)
        } else {
            0
        };
        vec![MaterialRange {
            material_index: mat_idx,
            start: 0,
            count: tobj_mesh.indices.len() as u32,
        }]
    } else {
        vec![MaterialRange {
            material_index: 0,
            start: 0,
            count: tobj_mesh.indices.len() as u32,
        }]
    };

    Mesh {
        id: EntityId::new(),
        name: model.name.clone(),
        positions: tobj_mesh.positions.clone(),
        normals: tobj_mesh.normals.clone(),
        tex_coords: tobj_mesh.texcoords.clone(),
        indices: tobj_mesh.indices.clone(),
        line_indices: Vec::new(),
        material_ranges,
        lod: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_obj(dir: &Path) -> std::path::PathBuf {
        let obj_path = dir.join("cube.obj");
        let mut f = std::fs::File::create(&obj_path).unwrap();
        write!(
            f,
            "\
# Simple cube
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0
vn  0.0  0.0  1.0
vn  0.0  0.0 -1.0
vn  1.0  0.0  0.0
vn -1.0  0.0  0.0
vn  0.0  1.0  0.0
vn  0.0 -1.0  0.0
f 1//1 2//1 3//1
f 1//1 3//1 4//1
f 5//2 7//2 6//2
f 5//2 8//2 7//2
f 2//3 6//3 7//3
f 2//3 7//3 3//3
f 1//4 4//4 8//4
f 1//4 8//4 5//4
f 4//5 3//5 7//5
f 4//5 7//5 8//5
f 1//6 5//6 6//6
f 1//6 6//6 2//6
"
        )
        .unwrap();
        obj_path
    }

    #[test]
    fn test_load_obj_cube() {
        let dir = tempfile::tempdir().unwrap();
        let obj_path = write_test_obj(dir.path());
        let world = load_obj(&obj_path).unwrap();

        assert_eq!(world.meshes.len(), 1, "cube should produce 1 mesh");
        let mesh = &world.meshes[0];
        // single_index mode duplicates vertices per unique position/normal combo
        // A cube with per-face normals: 6 faces * 2 triangles * 3 verts = up to 36,
        // but shared within each face = 24 unique pos/normal combos
        assert!(mesh.vertex_count() >= 8, "at least 8 vertices");
        assert_eq!(mesh.face_count(), 12, "cube has 12 triangles");
        assert!(!mesh.normals.is_empty(), "normals should be present");
        assert_eq!(world.materials.len(), 1, "default material");
    }

    #[test]
    fn test_load_obj_from_bytes() {
        let obj_data = b"\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";
        let world = load_obj_from_bytes(obj_data, "triangle.obj").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert_eq!(world.meshes[0].face_count(), 1);
        assert_eq!(world.meshes[0].vertex_count(), 3);
    }
}
