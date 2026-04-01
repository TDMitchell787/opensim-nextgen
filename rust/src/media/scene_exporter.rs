use std::path::Path;
use std::io::Write;
use anyhow::Result;
use tracing::info;

use super::{RenderJob, SceneExportObject};

pub fn export_scene(job: &RenderJob, meshes_dir: &Path) -> Result<()> {
    for (i, obj) in job.scene_objects.iter().enumerate() {
        let obj_path = meshes_dir.join(format!("object_{}.obj", i));
        let mtl_path = meshes_dir.join(format!("object_{}.mtl", i));

        generate_prim_obj(obj, &obj_path, &mtl_path, i)?;
    }

    if let Some(ref heightmap) = job.terrain_heightmap {
        let terrain_path = meshes_dir.join("terrain.obj");
        generate_terrain_obj(heightmap, &terrain_path)?;
    }

    let manifest = generate_manifest(job);
    let manifest_path = meshes_dir.join("manifest.json");
    std::fs::write(&manifest_path, &manifest)?;

    info!("[MEDIA] Exported {} objects + terrain to {}", job.scene_objects.len(), meshes_dir.display());
    Ok(())
}

fn generate_prim_obj(obj: &SceneExportObject, obj_path: &Path, mtl_path: &Path, index: usize) -> Result<()> {
    let mtl_name = format!("material_{}", index);

    let mut mtl_file = std::fs::File::create(mtl_path)?;
    writeln!(mtl_file, "newmtl {}", mtl_name)?;
    writeln!(mtl_file, "Kd {} {} {}", obj.color[0], obj.color[1], obj.color[2])?;
    writeln!(mtl_file, "d {}", obj.color[3])?;
    writeln!(mtl_file, "illum 2")?;

    let mut f = std::fs::File::create(obj_path)?;
    writeln!(f, "# OpenSim Next Scene Export")?;
    writeln!(f, "mtllib object_{}.mtl", index)?;

    match obj.shape_type.as_str() {
        "box" | "1" => write_box_vertices(&mut f, obj)?,
        "cylinder" | "3" => write_cylinder_vertices(&mut f, obj, 16)?,
        "sphere" | "5" => write_sphere_vertices(&mut f, obj, 12, 8)?,
        "torus" | "4" => write_torus_vertices(&mut f, obj, 16, 8)?,
        _ => write_box_vertices(&mut f, obj)?,
    }

    writeln!(f, "usemtl {}", mtl_name)?;

    Ok(())
}

fn write_box_vertices(f: &mut std::fs::File, _obj: &SceneExportObject) -> Result<()> {
    let verts = [
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5],
        [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
    ];
    for v in &verts {
        writeln!(f, "v {} {} {}", v[0], v[1], v[2])?;
    }
    let faces = [
        [1,2,3,4], [5,8,7,6], [1,5,6,2],
        [2,6,7,3], [3,7,8,4], [4,8,5,1],
    ];
    for face in &faces {
        writeln!(f, "f {} {} {} {}", face[0], face[1], face[2], face[3])?;
    }
    Ok(())
}

fn write_cylinder_vertices(f: &mut std::fs::File, _obj: &SceneExportObject, segments: u32) -> Result<()> {
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = 0.5 * angle.cos();
        let y = 0.5 * angle.sin();
        writeln!(f, "v {} {} -0.5", x, y)?;
        writeln!(f, "v {} {} 0.5", x, y)?;
    }
    writeln!(f, "v 0.0 0.0 -0.5")?;
    writeln!(f, "v 0.0 0.0 0.5")?;

    let bottom_center = segments * 2 + 1;
    let top_center = segments * 2 + 2;

    for i in 0..segments {
        let bl = i * 2 + 1;
        let tl = i * 2 + 2;
        let br = ((i + 1) % segments) * 2 + 1;
        let tr = ((i + 1) % segments) * 2 + 2;
        writeln!(f, "f {} {} {} {}", bl, br, tr, tl)?;
        writeln!(f, "f {} {} {}", bottom_center, br, bl)?;
        writeln!(f, "f {} {} {}", top_center, tl, tr)?;
    }
    Ok(())
}

fn write_sphere_vertices(f: &mut std::fs::File, _obj: &SceneExportObject, lon_segs: u32, lat_segs: u32) -> Result<()> {
    writeln!(f, "v 0.0 0.0 0.5")?;
    for j in 1..lat_segs {
        let phi = std::f32::consts::PI * j as f32 / lat_segs as f32;
        let z = 0.5 * phi.cos();
        let r = 0.5 * phi.sin();
        for i in 0..lon_segs {
            let theta = std::f32::consts::TAU * i as f32 / lon_segs as f32;
            writeln!(f, "v {} {} {}", r * theta.cos(), r * theta.sin(), z)?;
        }
    }
    writeln!(f, "v 0.0 0.0 -0.5")?;

    for i in 0..lon_segs {
        let next = (i + 1) % lon_segs;
        writeln!(f, "f 1 {} {}", i + 2, next + 2)?;
    }
    for j in 0..(lat_segs - 2) {
        for i in 0..lon_segs {
            let next = (i + 1) % lon_segs;
            let a = j * lon_segs + i + 2;
            let b = j * lon_segs + next + 2;
            let c = (j + 1) * lon_segs + next + 2;
            let d = (j + 1) * lon_segs + i + 2;
            writeln!(f, "f {} {} {} {}", a, b, c, d)?;
        }
    }
    let bottom = (lat_segs - 1) * lon_segs + 2;
    for i in 0..lon_segs {
        let next = (i + 1) % lon_segs;
        let ring_start = (lat_segs - 2) * lon_segs + 2;
        writeln!(f, "f {} {} {}", bottom, ring_start + next, ring_start + i)?;
    }
    Ok(())
}

fn write_torus_vertices(f: &mut std::fs::File, _obj: &SceneExportObject, major_segs: u32, minor_segs: u32) -> Result<()> {
    let major_r = 0.375;
    let minor_r = 0.125;

    for i in 0..major_segs {
        let theta = std::f32::consts::TAU * i as f32 / major_segs as f32;
        for j in 0..minor_segs {
            let phi = std::f32::consts::TAU * j as f32 / minor_segs as f32;
            let x = (major_r + minor_r * phi.cos()) * theta.cos();
            let y = (major_r + minor_r * phi.cos()) * theta.sin();
            let z = minor_r * phi.sin();
            writeln!(f, "v {} {} {}", x, y, z)?;
        }
    }

    for i in 0..major_segs {
        let next_i = (i + 1) % major_segs;
        for j in 0..minor_segs {
            let next_j = (j + 1) % minor_segs;
            let a = i * minor_segs + j + 1;
            let b = next_i * minor_segs + j + 1;
            let c = next_i * minor_segs + next_j + 1;
            let d = i * minor_segs + next_j + 1;
            writeln!(f, "f {} {} {} {}", a, b, c, d)?;
        }
    }
    Ok(())
}

fn generate_terrain_obj(heightmap: &[f32], path: &Path) -> Result<()> {
    let size = (heightmap.len() as f32).sqrt() as usize;
    if size == 0 { return Ok(()); }

    let mut f = std::fs::File::create(path)?;
    writeln!(f, "# OpenSim Next Terrain Export")?;

    let step = if size > 64 { size / 64 } else { 1 };
    let grid_size = size / step;

    for z in (0..size).step_by(step) {
        for x in (0..size).step_by(step) {
            let h = heightmap[z * size + x];
            writeln!(f, "v {} {} {}", x as f32, h, z as f32)?;
        }
    }

    for z in 0..(grid_size - 1) {
        for x in 0..(grid_size - 1) {
            let a = z * grid_size + x + 1;
            let b = a + 1;
            let c = (z + 1) * grid_size + x + 2;
            let d = c - 1;
            writeln!(f, "f {} {} {} {}", a, b, c, d)?;
        }
    }

    info!("[MEDIA] Terrain exported: {}x{} grid (step={})", grid_size, grid_size, step);
    Ok(())
}

fn generate_manifest(job: &RenderJob) -> String {
    let objects: Vec<serde_json::Value> = job.scene_objects.iter().enumerate().map(|(i, obj)| {
        serde_json::json!({
            "index": i,
            "name": obj.name,
            "file": format!("object_{}.obj", i),
            "position": obj.position,
            "rotation": obj.rotation,
            "scale": obj.scale,
            "shape_type": obj.shape_type,
        })
    }).collect();

    serde_json::json!({
        "job_id": job.job_id.to_string(),
        "region_id": job.region_id.to_string(),
        "object_count": job.scene_objects.len(),
        "has_terrain": job.terrain_heightmap.is_some(),
        "objects": objects,
    }).to_string()
}
