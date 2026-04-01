use anyhow::{bail, Result};
use std::path::Path;
use tracing::info;

use super::encoder::{MeshFace, MeshGeometry};
use super::collada_skin;

pub fn import_file(path: &str) -> Result<MeshGeometry> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        bail!("File not found: {}", path);
    }

    let ext = file_path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "obj" => load_obj_file(path),
        "stl" => load_stl_file(path),
        "dae" => {
            match import_dae_with_skin(path) {
                Ok(geom) => Ok(geom),
                Err(_) => import_dae_geometry_only(path),
            }
        }
        "gltf" | "glb" | "ply" | "off" | "3dxml" => load_via_glc(path),
        _ => bail!("Unsupported format: .{} (supported: obj, stl, dae, gltf, glb, ply, off, 3dxml)", ext),
    }
}

pub fn import_obj_from_string(obj_data: &str) -> Result<MeshGeometry> {
    let mut reader = std::io::BufReader::new(obj_data.as_bytes());
    let (models, _materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    if models.is_empty() {
        bail!("OBJ contains no meshes");
    }

    let mut faces = Vec::new();
    for model in &models {
        let mesh = &model.mesh;
        let face = tobj_mesh_to_face(mesh)?;
        faces.push(face);
    }

    info!("[GLC_BRIDGE] Parsed OBJ string: {} faces", faces.len());
    Ok(MeshGeometry { faces, skin_info: None })
}

fn load_obj_file(path: &str) -> Result<MeshGeometry> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )?;

    if models.is_empty() {
        bail!("OBJ file contains no meshes: {}", path);
    }

    let mut faces = Vec::new();
    for model in &models {
        let mesh = &model.mesh;
        let face = tobj_mesh_to_face(mesh)?;
        faces.push(face);
    }

    info!("[GLC_BRIDGE] Loaded OBJ '{}': {} model(s), {} total faces",
        path, models.len(), faces.len());
    Ok(MeshGeometry { faces, skin_info: None })
}

fn tobj_mesh_to_face(mesh: &tobj::Mesh) -> Result<MeshFace> {
    let vertex_count = mesh.positions.len() / 3;
    if vertex_count == 0 {
        bail!("Mesh has no vertices");
    }

    let positions: Vec<[f32; 3]> = mesh.positions
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    let normals: Vec<[f32; 3]> = if mesh.normals.len() == mesh.positions.len() {
        mesh.normals.chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect()
    } else {
        generate_flat_normals(&positions, &mesh.indices)
    };

    let tex_coords: Vec<[f32; 2]> = if mesh.texcoords.len() / 2 == vertex_count {
        mesh.texcoords.chunks_exact(2)
            .map(|c| [c[0], c[1]])
            .collect()
    } else {
        generate_planar_uvs(&positions)
    };

    Ok(MeshFace {
        positions,
        normals,
        tex_coords,
        indices: mesh.indices.clone(),
        joint_weights: None,
        original_position_indices: None,
    })
}

fn load_stl_file(path: &str) -> Result<MeshGeometry> {
    let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
    let stl = stl_io::read_stl(&mut file)?;

    let vertex_count = stl.vertices.len();
    if vertex_count == 0 {
        bail!("STL file has no vertices: {}", path);
    }

    let positions: Vec<[f32; 3]> = stl.vertices.iter()
        .map(|v| [v[0], v[1], v[2]])
        .collect();

    let mut indices = Vec::new();
    let mut normals_per_vertex = vec![[0.0f32; 3]; vertex_count];

    for face in &stl.faces {
        let n = face.normal;
        for &idx in &face.vertices {
            indices.push(idx as u32);
            let i = idx as usize;
            if i < normals_per_vertex.len() {
                normals_per_vertex[i] = [n[0], n[1], n[2]];
            }
        }
    }

    let tex_coords = generate_planar_uvs(&positions);

    info!("[GLC_BRIDGE] Loaded STL '{}': {} vertices, {} faces",
        path, vertex_count, stl.faces.len());

    Ok(MeshGeometry {
        faces: vec![MeshFace {
            positions,
            normals: normals_per_vertex,
            tex_coords,
            indices,
            joint_weights: None,
            original_position_indices: None,
        }],
        skin_info: None,
    })
}

fn load_via_glc(path: &str) -> Result<MeshGeometry> {
    let world = glc_io::load_model(Path::new(path))
        .map_err(|e| anyhow::anyhow!("GLC load failed for '{}': {}", path, e))?;

    if world.meshes.is_empty() {
        bail!("GLC model contains no meshes: {}", path);
    }

    let mut all_faces = Vec::new();
    for mesh in &world.meshes {
        if mesh.positions.is_empty() {
            continue;
        }
        let ranges: Vec<(u32, u32)> = mesh.material_ranges.iter()
            .map(|r| (r.start, r.count))
            .collect();
        let geom = flat_vecs_to_geometry(
            &mesh.positions,
            &mesh.normals,
            &mesh.tex_coords,
            &mesh.indices,
            &ranges,
        );
        all_faces.extend(geom.faces);
    }

    if all_faces.is_empty() {
        bail!("GLC model has no geometry data: {}", path);
    }

    info!("[GLC_BRIDGE] Loaded via GLC '{}': {} meshes, {} faces",
        path, world.meshes.len(), all_faces.len());
    Ok(MeshGeometry { faces: all_faces, skin_info: None })
}

pub fn generate_flat_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32, 0.0, 1.0]; positions.len()];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-8);
        let nn = [n[0] / len, n[1] / len, n[2] / len];
        normals[i0] = nn;
        normals[i1] = nn;
        normals[i2] = nn;
    }
    normals
}

pub fn generate_planar_uvs(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    if positions.is_empty() {
        return vec![];
    }
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for p in positions {
        if p[0] < min_x { min_x = p[0]; }
        if p[0] > max_x { max_x = p[0]; }
        if p[1] < min_y { min_y = p[1]; }
        if p[1] > max_y { max_y = p[1]; }
    }
    let range_x = (max_x - min_x).max(1e-6);
    let range_y = (max_y - min_y).max(1e-6);
    positions.iter()
        .map(|p| [(p[0] - min_x) / range_x, (p[1] - min_y) / range_y])
        .collect()
}

pub fn flat_vecs_to_geometry(
    positions: &[f32],
    normals: &[f32],
    tex_coords: &[f32],
    indices: &[u32],
    material_ranges: &[(u32, u32)],
) -> MeshGeometry {
    let pos_arr: Vec<[f32; 3]> = positions.chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let norm_arr: Vec<[f32; 3]> = if normals.len() == positions.len() {
        normals.chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect()
    } else {
        generate_flat_normals(&pos_arr, indices)
    };
    let tc_arr: Vec<[f32; 2]> = if tex_coords.len() / 2 == pos_arr.len() {
        tex_coords.chunks_exact(2)
            .map(|c| [c[0], c[1]])
            .collect()
    } else {
        generate_planar_uvs(&pos_arr)
    };

    if material_ranges.is_empty() || material_ranges.len() == 1 {
        return MeshGeometry {
            faces: vec![MeshFace {
                positions: pos_arr,
                normals: norm_arr,
                tex_coords: tc_arr,
                indices: indices.to_vec(),
                joint_weights: None,
                original_position_indices: None,
            }],
            skin_info: None,
        };
    }

    let mut faces = Vec::new();
    for &(start, count) in material_ranges {
        let range_indices = &indices[start as usize..(start + count) as usize];
        let mut index_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut new_positions = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_tex_coords = Vec::new();
        let mut new_indices = Vec::new();

        for &idx in range_indices {
            let new_idx = *index_map.entry(idx).or_insert_with(|| {
                let ni = new_positions.len() as u32;
                if (idx as usize) < pos_arr.len() {
                    new_positions.push(pos_arr[idx as usize]);
                    new_normals.push(norm_arr[idx as usize]);
                    new_tex_coords.push(tc_arr[idx as usize]);
                }
                ni
            });
            new_indices.push(new_idx);
        }

        faces.push(MeshFace {
            positions: new_positions,
            normals: new_normals,
            tex_coords: new_tex_coords,
            indices: new_indices,
            joint_weights: None,
            original_position_indices: None,
        });
    }

    MeshGeometry { faces, skin_info: None }
}

pub fn import_dae_with_skin(path: &str) -> Result<MeshGeometry> {
    let dae_xml = std::fs::read_to_string(path)?;

    let y_up = super::collada_geometry::is_collada_y_up(&dae_xml);
    info!("[GLC_BRIDGE] COLLADA up_axis: {}", if y_up { "Y_UP" } else { "Z_UP" });

    let skin_data = collada_skin::parse_collada_skin(&dae_xml)?;
    let (skin_info, all_vertex_weights) = collada_skin::to_mesh_skin_info_with_axis(&skin_data, y_up);

    let mut geom = match super::collada_geometry::parse_collada_geometry_with_axis(&dae_xml, y_up) {
        Ok(faces) => {
            info!("[GLC_BRIDGE] Native Collada parser: {} face(s)", faces.len());
            MeshGeometry { faces, skin_info: None }
        }
        Err(e) => {
            info!("[GLC_BRIDGE] Native Collada parse failed ({}), falling back to GLC", e);
            load_via_glc(path)?
        }
    };

    if !all_vertex_weights.is_empty() {
        for face in &mut geom.faces {
            if let Some(ref pos_indices) = face.original_position_indices {
                let face_weights: Vec<super::encoder::VertexWeights> = pos_indices.iter()
                    .map(|&pi| {
                        if pi < all_vertex_weights.len() {
                            all_vertex_weights[pi].clone()
                        } else {
                            super::encoder::VertexWeights { influences: vec![] }
                        }
                    })
                    .collect();
                face.joint_weights = Some(face_weights);
            } else {
                let fallback: Vec<super::encoder::VertexWeights> = (0..face.positions.len())
                    .map(|i| {
                        let src_idx = i % all_vertex_weights.len();
                        all_vertex_weights[src_idx].clone()
                    })
                    .collect();
                face.joint_weights = Some(fallback);
            }
        }
    }

    geom.skin_info = Some(skin_info);

    info!("[GLC_BRIDGE] Loaded DAE with skin '{}': {} faces, {} joints",
        path, geom.faces.len(), geom.skin_info.as_ref().map_or(0, |s| s.joint_names.len()));

    Ok(geom)
}

pub fn import_dae_geometry_only(path: &str) -> Result<MeshGeometry> {
    let dae_xml = std::fs::read_to_string(path)?;
    let y_up = super::collada_geometry::is_collada_y_up(&dae_xml);
    info!("[GLC_BRIDGE] COLLADA geometry-only import, up_axis: {}", if y_up { "Y_UP" } else { "Z_UP" });

    let faces = super::collada_geometry::parse_collada_geometry_with_axis(&dae_xml, y_up)?;
    info!("[GLC_BRIDGE] Geometry-only DAE '{}': {} faces, {} verts",
        path, faces.len(), faces.iter().map(|f| f.positions.len()).sum::<usize>());

    Ok(MeshGeometry { faces, skin_info: None })
}

pub fn import_ruth2_part(part_name: &str) -> Result<MeshGeometry> {
    let path = super::blender_worker::ruth2_dae_path(part_name)
        .ok_or_else(|| anyhow::anyhow!("Ruth2_v4 DAE not found for part '{}'", part_name))?;
    let geom = import_dae_with_skin(&path)?;
    let total_verts: usize = geom.faces.iter().map(|f| f.positions.len()).sum();
    let joint_count = geom.skin_info.as_ref().map_or(0, |s| s.joint_names.len());
    info!("[RUTH2] Imported '{}': {} faces, {} vertices, {} joints",
        part_name, geom.faces.len(), total_verts, joint_count);
    Ok(geom)
}

pub fn import_ruth2_body_set(parts: &[&str]) -> Result<Vec<(String, MeshGeometry)>> {
    let mut results = Vec::new();
    for &part in parts {
        let geom = import_ruth2_part(part)?;
        results.push((part.to_string(), geom));
    }
    info!("[RUTH2] Batch import: {} parts loaded", results.len());
    Ok(results)
}

pub fn import_multi_lod_dae(paths: &[std::path::PathBuf]) -> Result<super::encoder::MultiLodGeometry> {
    if paths.is_empty() {
        bail!("No LOD files provided");
    }

    let load_lod = |path: &std::path::PathBuf| -> Result<MeshGeometry> {
        let path_str = path.to_string_lossy();
        import_dae_with_skin(&path_str)
    };

    let high = load_lod(&paths[0])?;

    let medium = if paths.len() > 1 {
        load_lod(&paths[1]).unwrap_or_else(|_| high.clone())
    } else {
        high.clone()
    };

    let low = if paths.len() > 2 {
        load_lod(&paths[2]).unwrap_or_else(|_| medium.clone())
    } else {
        medium.clone()
    };

    let lowest = if paths.len() > 3 {
        load_lod(&paths[3]).unwrap_or_else(|_| low.clone())
    } else {
        low.clone()
    };

    let high_verts: usize = high.faces.iter().map(|f| f.positions.len()).sum();
    let med_verts: usize = medium.faces.iter().map(|f| f.positions.len()).sum();
    let low_verts: usize = low.faces.iter().map(|f| f.positions.len()).sum();
    let lowest_verts: usize = lowest.faces.iter().map(|f| f.positions.len()).sum();

    info!("[GLC_BRIDGE] Multi-LOD: high={} med={} low={} lowest={} vertices",
        high_verts, med_verts, low_verts, lowest_verts);

    Ok(super::encoder::MultiLodGeometry { high, medium, low, lowest })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_normals_triangle() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];
        let normals = generate_flat_normals(&positions, &indices);
        assert_eq!(normals.len(), 3);
        assert!((normals[0][2] - 1.0).abs() < 0.01, "Z normal should be ~1.0");
    }

    #[test]
    fn test_planar_uvs() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let uvs = generate_planar_uvs(&positions);
        assert_eq!(uvs.len(), 3);
        assert!((uvs[0][0]).abs() < 0.01, "min should map to 0");
        assert!((uvs[1][0] - 1.0).abs() < 0.01, "max should map to 1");
    }

    #[test]
    fn test_flat_vecs_single_face() {
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let tex_coords = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let indices = vec![0, 1, 2];
        let geom = flat_vecs_to_geometry(&positions, &normals, &tex_coords, &indices, &[]);
        assert_eq!(geom.faces.len(), 1);
        assert_eq!(geom.faces[0].positions.len(), 3);
        assert_eq!(geom.faces[0].indices.len(), 3);
    }

    #[test]
    fn test_flat_vecs_multi_material() {
        let positions = vec![
            0.0, 0.0, 0.0,  1.0, 0.0, 0.0,  0.0, 1.0, 0.0,
            2.0, 0.0, 0.0,  3.0, 0.0, 0.0,  2.0, 1.0, 0.0,
        ];
        let normals: Vec<f32> = vec![];
        let tex_coords: Vec<f32> = vec![];
        let indices = vec![0, 1, 2, 3, 4, 5];
        let ranges = vec![(0u32, 3u32), (3, 3)];
        let geom = flat_vecs_to_geometry(&positions, &normals, &tex_coords, &indices, &ranges);
        assert_eq!(geom.faces.len(), 2);
        assert_eq!(geom.faces[0].positions.len(), 3);
        assert_eq!(geom.faces[1].positions.len(), 3);
    }

    #[test]
    fn test_obj_string_parse() {
        let obj_data = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n";
        let geom = import_obj_from_string(obj_data).expect("OBJ parse failed");
        assert_eq!(geom.faces.len(), 1);
        assert_eq!(geom.faces[0].positions.len(), 3);
        assert_eq!(geom.faces[0].indices.len(), 3);
    }

    #[test]
    fn test_import_ruth2_body() {
        if super::super::blender_worker::ruth2_base_dir().is_none() { return; }
        let geom = import_ruth2_part("body").expect("Ruth2v4Body.dae import failed");
        assert!(geom.faces.len() >= 1, "Body should have at least 1 face");
        assert!(geom.skin_info.is_some(), "Body should have skin data");
        let joints = geom.skin_info.as_ref().unwrap().joint_names.len();
        assert!(joints >= 50, "Body should have 50+ joints, got {}", joints);
    }

    #[test]
    fn test_import_ruth2_headless() {
        if super::super::blender_worker::ruth2_base_dir().is_none() { return; }
        let geom = import_ruth2_part("headless").expect("Ruth2v4Headless.dae import failed");
        assert!(geom.skin_info.is_some(), "Headless should have skin data");
    }

    #[test]
    fn test_import_ruth2_head() {
        if super::super::blender_worker::ruth2_base_dir().is_none() { return; }
        let geom = import_ruth2_part("head").expect("Ruth2v4Head.dae import failed");
        assert!(geom.faces.len() >= 1, "Head should have at least 1 face");
    }

    #[test]
    fn test_import_ruth2_body_set() {
        if super::super::blender_worker::ruth2_base_dir().is_none() { return; }
        let parts = import_ruth2_body_set(&["body", "hands", "head"])
            .expect("Body set import failed");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].0, "body");
        assert_eq!(parts[1].0, "hands");
        assert_eq!(parts[2].0, "head");
    }

    #[test]
    fn test_import_ruth2_unknown_part_fails() {
        assert!(import_ruth2_part("nonexistent").is_err());
    }

    #[test]
    fn test_obj_string_quad_triangulated() {
        let obj_data = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 1.0 1.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3 4\n";
        let geom = import_obj_from_string(obj_data).expect("OBJ quad parse failed");
        assert_eq!(geom.faces.len(), 1);
        assert_eq!(geom.faces[0].indices.len(), 6, "Quad should triangulate to 6 indices");
    }
}
