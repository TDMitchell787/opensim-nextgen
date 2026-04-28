use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

const BLENDER_PATH: &str = "/Applications/Blender.app/Contents/MacOS/Blender";

pub struct BlenderWorker {
    temp_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Obj,
    Dae,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GarmentType {
    Shirt,
    Dress,
    Pants,
    Jacket,
    Skirt,
}

#[derive(Debug, Clone)]
pub struct GarmentSpec {
    pub garment_type: GarmentType,
    pub z_min: f32,
    pub z_max: f32,
    pub include_arms: bool,
    pub offset: f32,
    pub subdivision_level: u32,
}

impl GarmentType {
    pub fn spec(&self, fit: &str, hem_length: Option<f32>) -> GarmentSpec {
        let offset = match fit {
            "tight" => 0.004,
            "loose" => 0.012,
            _ => 0.008,
        };
        match self {
            GarmentType::Shirt => GarmentSpec {
                garment_type: *self,
                z_min: 0.47,
                z_max: 0.88,
                include_arms: true,
                offset,
                subdivision_level: 1,
            },
            GarmentType::Dress => GarmentSpec {
                garment_type: *self,
                z_min: hem_length.unwrap_or(0.3).clamp(0.1, 0.47),
                z_max: 0.88,
                include_arms: true,
                offset,
                subdivision_level: 1,
            },
            GarmentType::Pants => GarmentSpec {
                garment_type: *self,
                z_min: 0.03,
                z_max: 0.47,
                include_arms: false,
                offset,
                subdivision_level: 1,
            },
            GarmentType::Jacket => GarmentSpec {
                garment_type: *self,
                z_min: 0.40,
                z_max: 0.88,
                include_arms: true,
                offset,
                subdivision_level: 1,
            },
            GarmentType::Skirt => GarmentSpec {
                garment_type: *self,
                z_min: hem_length.unwrap_or(0.3).clamp(0.1, 0.47),
                z_max: 0.47,
                include_arms: false,
                offset,
                subdivision_level: 1,
            },
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "shirt" => Some(GarmentType::Shirt),
            "dress" => Some(GarmentType::Dress),
            "pants" => Some(GarmentType::Pants),
            "jacket" => Some(GarmentType::Jacket),
            "skirt" => Some(GarmentType::Skirt),
            _ => None,
        }
    }
}

impl BlenderWorker {
    pub fn new() -> Result<Self> {
        let temp_dir =
            std::env::temp_dir().join(format!("opensim_blender_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        Ok(Self { temp_dir })
    }

    pub async fn generate(&self, python_code: &str) -> Result<PathBuf> {
        self.generate_with_format(python_code, ExportFormat::Obj)
            .await
    }

    pub async fn run_script(&self, python_code: &str) -> Result<String> {
        let blender = PathBuf::from(BLENDER_PATH);
        if !blender.exists() {
            bail!("Blender not found at {}", BLENDER_PATH);
        }
        let script_path = self.temp_dir.join("run_script.py");
        let full_script = format!("import bpy\n{}\n", python_code);
        std::fs::write(&script_path, &full_script)?;

        let output = tokio::process::Command::new(BLENDER_PATH)
            .args(["--background", "--python", &script_path.to_string_lossy()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("SNAPSHOT_STATUE")
                || line.contains("Error")
                || line.contains("Traceback")
            {
                info!("[BLENDER] {}", line);
            }
        }

        if !output.status.success() {
            bail!("Blender script failed (exit {}): {}", output.status, stderr);
        }

        Ok(stdout)
    }

    pub async fn generate_with_format(
        &self,
        python_code: &str,
        format: ExportFormat,
    ) -> Result<PathBuf> {
        let blender = PathBuf::from(BLENDER_PATH);
        if !blender.exists() {
            bail!("Blender not found at {}", BLENDER_PATH);
        }

        let (output_path, export_cmd) = match format {
            ExportFormat::Obj => {
                let p = self.temp_dir.join("output.obj");
                let cmd = format!("bpy.ops.wm.obj_export(filepath='{}', export_selected_objects=True, up_axis='Z', forward_axis='X')",
                    p.to_string_lossy().replace('\\', "/"));
                (p, cmd)
            }
            ExportFormat::Dae => {
                let p = self.temp_dir.join("output.dae");
                let cmd = format!(
                    "bpy.ops.wm.collada_export(filepath='{}', apply_modifiers=True, selected=True, include_armatures=True)",
                    p.to_string_lossy().replace('\\', "/"));
                (p, cmd)
            }
        };

        let script_path = self.temp_dir.join("generate.py");

        let preamble = match format {
            ExportFormat::Obj => {
                "import bpy\nbpy.ops.wm.read_factory_settings(use_empty=True)\n".to_string()
            }
            ExportFormat::Dae => "import bpy\n".to_string(),
        };

        let full_script = format!("{}{}\n{}\n", preamble, python_code, export_cmd);

        std::fs::write(&script_path, &full_script)?;

        let output = tokio::process::Command::new(BLENDER_PATH)
            .args(["--background", "--python", &script_path.to_string_lossy()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Blender failed (exit {}): {}", output.status, stderr);
        }

        if !output_path.exists() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}\n{}", stdout, stderr);
            let relevant: Vec<&str> = combined
                .lines()
                .filter(|l| {
                    l.contains("Error")
                        || l.contains("Traceback")
                        || l.contains("BLEND")
                        || l.contains("ARMATURE")
                        || l.contains("BODY")
                        || l.contains("PRE-EXPORT")
                        || l.contains("SELECTION")
                        || l.contains("SHIRT")
                        || l.contains("AttributeError")
                        || l.contains("Exception")
                        || l.contains("RuntimeError")
                })
                .collect();
            let detail = if relevant.is_empty() {
                let last_lines: Vec<&str> = combined.lines().rev().take(20).collect();
                last_lines
                    .into_iter()
                    .rev()
                    .collect::<Vec<&str>>()
                    .join("\n")
            } else {
                relevant.join("\n")
            };
            bail!(
                "Blender did not produce output file: {} — {}",
                output_path.display(),
                detail
            );
        }

        let blender_stdout = String::from_utf8_lossy(&output.stdout);
        for line in blender_stdout.lines() {
            if line.contains("STATUE")
                || line.contains("MAPPED")
                || line.contains("BVH")
                || line.contains("PRE-EXPORT")
                || line.contains("DECIMATE")
            {
                info!("[BLENDER] {}", line.trim());
            }
        }

        info!(
            "[BLENDER] Generated mesh ({}): {}",
            if format == ExportFormat::Dae {
                "DAE"
            } else {
                "OBJ"
            },
            output_path.display()
        );

        let stage_dir = std::path::PathBuf::from("mesh");
        if !stage_dir.exists() {
            let _ = std::fs::create_dir_all(&stage_dir);
        }
        let stage_name = format!(
            "latest_output.{}",
            if format == ExportFormat::Dae {
                "dae"
            } else {
                "obj"
            }
        );
        let stage_path = stage_dir.join(&stage_name);
        match std::fs::copy(&output_path, &stage_path) {
            Ok(_) => info!("[BLENDER] Staged copy: {}", stage_path.display()),
            Err(e) => warn!("[BLENDER] Failed to stage copy: {}", e),
        }

        Ok(output_path)
    }

    pub fn get_template(name: &str, params: &HashMap<String, String>) -> Result<String> {
        let template = match name {
            "table" => TEMPLATE_TABLE,
            "chair" => TEMPLATE_CHAIR,
            "shelf" => TEMPLATE_SHELF,
            "arch" => TEMPLATE_ARCH,
            "staircase" => TEMPLATE_STAIRCASE,
            "stone" => TEMPLATE_STONE,
            "stone_ring" => TEMPLATE_STONE_RING,
            "boulder" => TEMPLATE_BOULDER,
            "column" => TEMPLATE_COLUMN,
            "path" | "walkway" | "cobblestone_path" | "serpentine_path" => TEMPLATE_PATH,
            "shirt" => TEMPLATE_SHIRT_BODYCLONE,
            "shirt_legacy" => TEMPLATE_SHIRT_LEGACY,
            "pants" => TEMPLATE_PANTS_BODYCLONE,
            "pants_legacy" => TEMPLATE_PANTS_LEGACY,
            "dress" => TEMPLATE_DRESS_BODYCLONE,
            "jacket" => TEMPLATE_JACKET_BODYCLONE,
            "skirt" => TEMPLATE_SKIRT_BODYCLONE,
            "bodysuit" => TEMPLATE_BODYSUIT_BODYCLONE,
            "bodysuit_posed" | "statue" => TEMPLATE_BODYSUIT_POSED,
            "snapshot_statue" => TEMPLATE_SNAPSHOT_STATUE,
            _ => bail!("Unknown Blender template: '{}' (available: table, chair, shelf, arch, staircase, stone, stone_ring, boulder, column, path, shirt, pants, dress, jacket, skirt, bodysuit, bodysuit_posed, statue, snapshot_statue)", name),
        };

        let mut result = template.to_string();

        let body_type = params.get("BODY").map(|s| s.as_str()).unwrap_or("ruth2");
        let body_path = body_blend_path(body_type);
        result = result.replace("{{BODY_BLEND_PATH}}", &body_path);

        if let Some(pose_name) = params.get("POSE") {
            let bvh_path = resolve_pose_bvh(pose_name);
            info!(
                "[BLENDER] Resolved pose '{}' -> BVH path: '{}' (exists={})",
                pose_name,
                bvh_path,
                std::path::Path::new(&bvh_path).exists()
            );
            result = result.replace("{{BVH_PATH}}", &bvh_path);
        } else {
            info!(
                "[BLENDER] No POSE param found in: {:?}",
                params.keys().collect::<Vec<_>>()
            );
        }

        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result = apply_defaults(&result);
        Ok(result)
    }

    pub async fn generate_multi_lod(&self, python_code: &str) -> Result<Vec<PathBuf>> {
        let blender = PathBuf::from(BLENDER_PATH);
        if !blender.exists() {
            bail!("Blender not found at {}", BLENDER_PATH);
        }

        let lod_ratios = [(0, 1.0f32), (1, 0.5f32), (2, 0.25f32), (3, 0.10f32)];
        let mut lod_paths = Vec::new();

        for &(lod_level, ratio) in &lod_ratios {
            let output_path = self.temp_dir.join(format!("output_lod{}.dae", lod_level));
            let export_cmd = format!(
                "bpy.ops.wm.collada_export(filepath='{}', apply_modifiers=True, selected=True, include_armatures=True)",
                output_path.to_string_lossy().replace('\\', "/")
            );

            let decimate_code = if lod_level == 0 {
                String::new()
            } else {
                format!(
                    r#"
garment = None
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        garment = obj
        break
if garment:
    bpy.context.view_layer.objects.active = garment
    mod_dec = garment.modifiers.new('LOD_Decimate', 'DECIMATE')
    mod_dec.ratio = {:.2}
    bpy.ops.object.modifier_apply(modifier='LOD_Decimate')
"#,
                    ratio
                )
            };

            let full_script = format!(
                "import bpy\n{}\n{}\n{}\n",
                python_code, decimate_code, export_cmd
            );

            let script_path = self.temp_dir.join(format!("generate_lod{}.py", lod_level));
            std::fs::write(&script_path, &full_script)?;

            let output = tokio::process::Command::new(BLENDER_PATH)
                .args(["--background", "--python", &script_path.to_string_lossy()])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await?;

            if !output.status.success() || !output_path.exists() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("[BLENDER] LOD{} generation failed: {}", lod_level, stderr);
                if lod_level == 0 {
                    bail!("LOD0 generation failed: {}", stderr);
                }
                continue;
            }

            info!(
                "[BLENDER] Generated LOD{}: {}",
                lod_level,
                output_path.display()
            );
            lod_paths.push(output_path);
        }

        Ok(lod_paths)
    }

    pub fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

impl Drop for BlenderWorker {
    fn drop(&mut self) {
        self.cleanup();
    }
}

pub fn is_clothing_template(name: &str) -> bool {
    matches!(
        name,
        "shirt" | "pants" | "dress" | "jacket" | "skirt" | "shirt_legacy" | "pants_legacy"
    )
}

pub fn body_blend_path(body_type: &str) -> String {
    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let base = format!("{}/../..", instance_dir);
    let parent = format!("{}/../../..", instance_dir);

    let candidates: Vec<String> = if body_type == "roth2" {
        vec![
            format!(
                "{}/content/Roth2-master/Mesh/Roth2V2DevWithArmature.blend",
                parent
            ),
            format!(
                "{}/content/Roth2-master/Mesh/Roth2V2DevWithArmature.blend",
                base
            ),
            "content/Roth2-master/Mesh/Roth2V2DevWithArmature.blend".to_string(),
        ]
    } else {
        vec![
            format!("{}/Ruth2_v4/Ruth2v4Dev_PartialLindenSkeleton.blend", parent),
            format!("{}/Ruth2_v4/Ruth2v4Dev_PartialLindenSkeleton.blend", base),
            "../Ruth2_v4/Ruth2v4Dev_PartialLindenSkeleton.blend".to_string(),
            "../../Ruth2_v4/Ruth2v4Dev_PartialLindenSkeleton.blend".to_string(),
            "../../../Ruth2_v4/Ruth2v4Dev_PartialLindenSkeleton.blend".to_string(),
            format!("{}/Ruth2_v4/Ruth2v4Dev.blend", parent),
            format!("{}/Ruth2_v4/Ruth2v4Dev.blend", base),
            "../Ruth2_v4/Ruth2v4Dev.blend".to_string(),
            "../../Ruth2_v4/Ruth2v4Dev.blend".to_string(),
            "../../../Ruth2_v4/Ruth2v4Dev.blend".to_string(),
        ]
    };

    for c in &candidates {
        let p = std::path::Path::new(c);
        if p.exists() {
            return p
                .canonicalize()
                .unwrap_or_else(|_| p.to_path_buf())
                .to_string_lossy()
                .to_string();
        }
    }
    candidates[0].clone()
}

fn armature_path() -> String {
    body_blend_path("ruth2")
}

fn resolve_pose_bvh(pose_name: &str) -> String {
    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let base = format!("{}/../..", instance_dir);
    let parent = format!("{}/../../..", instance_dir);

    let name_lower = pose_name.to_lowercase();
    let bvh_file = if name_lower.ends_with(".bvh") {
        name_lower.clone()
    } else {
        format!("{}.bvh", name_lower)
    };

    let candidates = vec![
        format!("{}/assets/poses/{}", parent, bvh_file),
        format!("{}/assets/poses/{}", base, bvh_file),
        format!("assets/poses/{}", bvh_file),
        format!("../assets/poses/{}", bvh_file),
        bvh_file.clone(),
    ];

    for c in &candidates {
        let p = std::path::Path::new(c);
        if p.exists() {
            return p
                .canonicalize()
                .unwrap_or_else(|_| p.to_path_buf())
                .to_string_lossy()
                .to_string();
        }
    }
    candidates[0].clone()
}

pub fn list_available_poses() -> Vec<String> {
    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let search_dirs = vec![
        format!("{}/../../assets/poses", instance_dir),
        format!("{}/../../../assets/poses", instance_dir),
        "assets/poses".to_string(),
        "../assets/poses".to_string(),
    ];

    for dir in &search_dirs {
        let p = std::path::Path::new(dir);
        if p.is_dir() {
            if let Ok(entries) = std::fs::read_dir(p) {
                return entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "bvh"))
                    .map(|e| {
                        e.path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
                    .collect();
            }
        }
    }
    Vec::new()
}

fn apply_defaults(script: &str) -> String {
    let arm_path = armature_path();
    let defaults = [
        ("{{WIDTH}}", "1.0"),
        ("{{DEPTH}}", "0.6"),
        ("{{HEIGHT}}", "0.8"),
        ("{{LEG_RADIUS}}", "0.03"),
        ("{{SEAT_W}}", "0.5"),
        ("{{SEAT_D}}", "0.5"),
        ("{{SEAT_H}}", "0.5"),
        ("{{BACK_H}}", "0.5"),
        ("{{SHELVES}}", "3"),
        ("{{SEGMENTS}}", "16"),
        ("{{RADIUS}}", "1.5"),
        ("{{STEPS}}", "12"),
        ("{{STEP_W}}", "0.8"),
        ("{{SIZE}}", "0.5"),
        ("{{ROUGHNESS}}", "0.15"),
        ("{{SUBDIVISIONS}}", "3"),
        ("{{RING_RADIUS}}", "1.5"),
        ("{{STONE_SIZE}}", "0.25"),
        ("{{STONE_COUNT}}", "19"),
        ("{{DECIMATE}}", "0.5"),
        ("{{COL_RADIUS}}", "0.15"),
        ("{{COL_HEIGHT}}", "2.0"),
        ("{{FLUTING}}", "12"),
        ("{{PATH_LENGTH}}", "10.0"),
        ("{{PATH_WIDTH}}", "2.0"),
        ("{{PATH_HEIGHT}}", "0.5"),
        ("{{PATH_CURVE}}", "1"),
        ("{{SLEEVE_LENGTH}}", "1.0"),
        ("{{FIT}}", "normal"),
        ("{{COLLAR}}", "crew"),
        ("{{LEG_LENGTH}}", "1.0"),
        ("{{WAIST}}", "mid"),
        ("{{TEXTURE_PATH}}", ""),
        ("{{BODY}}", "ruth2"),
        ("{{NECKLINE}}", "crew"),
        ("{{HEM_LENGTH}}", "0.3"),
        ("{{FLARE}}", "0.3"),
        ("{{SIMULATE}}", "false"),
        ("{{POSE}}", "standing"),
        ("{{BVH_PATH}}", ""),
        ("{{MANIFEST_PATH}}", ""),
        ("{{FRAME}}", "0"),
    ];
    let mut result = script.to_string();
    for (placeholder, default) in defaults {
        result = result.replace(placeholder, default);
    }
    result = result.replace("{{ARMATURE_PATH}}", &arm_path);
    result
}

const TEMPLATE_TABLE: &str = r#"
import bpy
import math

W = {{WIDTH}}
D = {{DEPTH}}
H = {{HEIGHT}}
LR = {{LEG_RADIUS}}

bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, H))
top = bpy.context.active_object
top.scale = (W/2, D/2, 0.03)
bpy.ops.object.transform_apply(scale=True)

legs = []
for x, y in [(-W/2+LR, -D/2+LR), (W/2-LR, -D/2+LR), (-W/2+LR, D/2-LR), (W/2-LR, D/2-LR)]:
    bpy.ops.mesh.primitive_cylinder_add(radius=LR, depth=H, location=(x, y, H/2))
    legs.append(bpy.context.active_object)

for leg in legs:
    leg.select_set(True)
top.select_set(True)
bpy.context.view_layer.objects.active = top
bpy.ops.object.join()
"#;

const TEMPLATE_CHAIR: &str = r#"
import bpy
import math

SW = {{SEAT_W}}
SD = {{SEAT_D}}
SH = {{SEAT_H}}
BH = {{BACK_H}}
LR = 0.025

bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, SH))
seat = bpy.context.active_object
seat.scale = (SW/2, SD/2, 0.025)
bpy.ops.object.transform_apply(scale=True)

legs = []
for x, y in [(-SW/2+LR, -SD/2+LR), (SW/2-LR, -SD/2+LR), (-SW/2+LR, SD/2-LR), (SW/2-LR, SD/2-LR)]:
    bpy.ops.mesh.primitive_cylinder_add(radius=LR, depth=SH, location=(x, y, SH/2))
    legs.append(bpy.context.active_object)

bpy.ops.mesh.primitive_cube_add(size=1, location=(0, -SD/2+0.015, SH+BH/2))
back = bpy.context.active_object
back.scale = (SW/2, 0.015, BH/2)
bpy.ops.object.transform_apply(scale=True)

for obj in legs + [back]:
    obj.select_set(True)
seat.select_set(True)
bpy.context.view_layer.objects.active = seat
bpy.ops.object.join()
"#;

const TEMPLATE_SHELF: &str = r#"
import bpy
import math

W = {{WIDTH}}
D = {{DEPTH}}
H = {{HEIGHT}}
N = int({{SHELVES}})

parts = []
for i in range(N):
    z = (i / (N - 1)) * H if N > 1 else H / 2
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, z))
    shelf = bpy.context.active_object
    shelf.scale = (W/2, D/2, 0.01)
    bpy.ops.object.transform_apply(scale=True)
    parts.append(shelf)

for x in [-W/2+0.01, W/2-0.01]:
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, 0, H/2))
    side = bpy.context.active_object
    side.scale = (0.01, D/2, H/2)
    bpy.ops.object.transform_apply(scale=True)
    parts.append(side)

for p in parts[1:]:
    p.select_set(True)
parts[0].select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
"#;

const TEMPLATE_ARCH: &str = r#"
import bpy
import math

W = {{WIDTH}}
H = {{HEIGHT}}
D = {{DEPTH}}
SEG = int({{SEGMENTS}})

parts = []
for x in [-W/2, W/2]:
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, 0, (H-W/2)/2))
    col = bpy.context.active_object
    col.scale = (0.1, D/2, (H-W/2)/2)
    bpy.ops.object.transform_apply(scale=True)
    parts.append(col)

verts = []
for i in range(SEG + 1):
    angle = math.pi * i / SEG
    x = math.cos(angle) * W / 2
    z = H - W/2 + math.sin(angle) * W / 2
    verts.append((x, -D/2, z))
    verts.append((x, D/2, z))

faces_list = []
for i in range(SEG):
    a = i * 2
    faces_list.append((a, a+2, a+3, a+1))

mesh = bpy.data.meshes.new("arch_curve")
mesh.from_pydata(verts, [], faces_list)
mesh.update()
arch_obj = bpy.data.objects.new("arch_curve", mesh)
bpy.context.collection.objects.link(arch_obj)
parts.append(arch_obj)

for p in parts:
    p.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
"#;

const TEMPLATE_STAIRCASE: &str = r#"
import bpy
import math

R = {{RADIUS}}
H = {{HEIGHT}}
STEPS = int({{STEPS}})
STEP_W = {{STEP_W}}

parts = []
for i in range(STEPS):
    angle = 2 * math.pi * i / STEPS
    z = H * i / STEPS
    x = math.cos(angle) * R / 2
    y = math.sin(angle) * R / 2
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, y, z + 0.05))
    step = bpy.context.active_object
    step.scale = (STEP_W/2, 0.15, 0.05)
    step.rotation_euler = (0, 0, angle)
    bpy.ops.object.transform_apply(scale=True, rotation=True)
    parts.append(step)

bpy.ops.mesh.primitive_cylinder_add(radius=0.05, depth=H, location=(0, 0, H/2))
pole = bpy.context.active_object
parts.append(pole)

for p in parts:
    p.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
"#;

const TEMPLATE_STONE: &str = r#"
import bpy
import bmesh
import random

SIZE = {{SIZE}}
ROUGHNESS = {{ROUGHNESS}}
SUBDIV = int({{SUBDIVISIONS}})
DECIMATE_RATIO = {{DECIMATE}}

bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=SUBDIV, radius=SIZE/2, location=(0, 0, 0))
stone = bpy.context.active_object

random.seed(42)
bm = bmesh.new()
bm.from_mesh(stone.data)
for v in bm.verts:
    disp = ROUGHNESS * SIZE
    v.co.x += random.uniform(-disp, disp)
    v.co.y += random.uniform(-disp, disp)
    v.co.z += random.uniform(-disp * 0.5, disp * 0.5)
bm.to_mesh(stone.data)
bm.free()

if DECIMATE_RATIO < 1.0:
    mod = stone.modifiers.new('Decimate', 'DECIMATE')
    mod.ratio = DECIMATE_RATIO
    bpy.ops.object.modifier_apply(modifier='Decimate')

stone.location.z = 0
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
"#;

const TEMPLATE_STONE_RING: &str = r#"
import bpy
import bmesh
import math
import random

RING_R = {{RING_RADIUS}}
STONE_R = {{STONE_SIZE}}
COUNT = int({{STONE_COUNT}})
ROUGHNESS = {{ROUGHNESS}}
DECIMATE_RATIO = {{DECIMATE}}

parts = []
for i in range(COUNT):
    angle = 2 * math.pi * i / COUNT
    x = math.cos(angle) * RING_R
    y = math.sin(angle) * RING_R
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=STONE_R, location=(x, y, 0))
    stone = bpy.context.active_object

    random.seed(i * 137)
    bm = bmesh.new()
    bm.from_mesh(stone.data)
    for v in bm.verts:
        disp = ROUGHNESS * STONE_R
        v.co.x += random.uniform(-disp, disp)
        v.co.y += random.uniform(-disp, disp)
        v.co.z += random.uniform(-disp * 0.6, disp * 0.6)
    bm.to_mesh(stone.data)
    bm.free()

    stone.rotation_euler = (random.uniform(-0.15, 0.15), random.uniform(-0.15, 0.15), random.uniform(0, 6.28))
    sx = random.uniform(0.8, 1.2)
    sy = random.uniform(0.8, 1.2)
    sz = random.uniform(0.6, 0.9)
    stone.scale = (sx, sy, sz)
    bpy.ops.object.transform_apply(rotation=True, scale=True)
    parts.append(stone)

bpy.ops.object.select_all(action='DESELECT')
for p in parts:
    p.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()

if DECIMATE_RATIO < 1.0:
    mod = bpy.context.active_object.modifiers.new('Decimate', 'DECIMATE')
    mod.ratio = DECIMATE_RATIO
    bpy.ops.object.modifier_apply(modifier='Decimate')

bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
"#;

const TEMPLATE_BOULDER: &str = r#"
import bpy
import bmesh
import random

SIZE = {{SIZE}}
ROUGHNESS = {{ROUGHNESS}}

bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=3, radius=SIZE/2, location=(0, 0, 0))
boulder = bpy.context.active_object

random.seed(99)
bm = bmesh.new()
bm.from_mesh(boulder.data)
for v in bm.verts:
    disp = ROUGHNESS * SIZE
    v.co.x *= random.uniform(0.7, 1.3)
    v.co.y *= random.uniform(0.7, 1.3)
    v.co.z *= random.uniform(0.5, 0.9)
    v.co.x += random.uniform(-disp, disp)
    v.co.y += random.uniform(-disp, disp)
    v.co.z += random.uniform(-disp * 0.3, disp * 0.3)
bm.to_mesh(boulder.data)
bm.free()

mod = boulder.modifiers.new('Smooth', 'SMOOTH')
mod.factor = 0.5
mod.iterations = 2
bpy.ops.object.modifier_apply(modifier='Smooth')

boulder.location.z = 0
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
"#;

const TEMPLATE_COLUMN: &str = r#"
import bpy
import math

R = {{COL_RADIUS}}
H = {{COL_HEIGHT}}
FLUTES = int({{FLUTING}})

parts = []

bpy.ops.mesh.primitive_cylinder_add(vertices=FLUTES*2, radius=R, depth=H, location=(0, 0, H/2))
shaft = bpy.context.active_object
parts.append(shaft)

bpy.ops.mesh.primitive_cylinder_add(vertices=32, radius=R*1.3, depth=H*0.06, location=(0, 0, 0))
base = bpy.context.active_object
parts.append(base)

bpy.ops.mesh.primitive_cylinder_add(vertices=32, radius=R*1.2, depth=H*0.04, location=(0, 0, H))
cap = bpy.context.active_object
parts.append(cap)

bpy.ops.object.select_all(action='DESELECT')
for p in parts:
    p.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
"#;

const TEMPLATE_PATH: &str = r#"
import bpy
import bmesh
import math
import random

LENGTH = {{PATH_LENGTH}}
WIDTH = {{PATH_WIDTH}}
HEIGHT = {{PATH_HEIGHT}}
CURVE = {{PATH_CURVE}}
STONE_R = 0.12
ROUGHNESS = 0.15

AMP = WIDTH * 0.8 * (1 if CURVE != 0 else 0)

def path_point(t):
    x = t * LENGTH
    y = math.sin(t * math.pi) * AMP if CURVE != 0 else 0
    return x, y

def path_angle(t):
    if CURVE == 0:
        return 0
    dt = 0.001
    x0, y0 = path_point(t)
    x1, y1 = path_point(min(t + dt, 1.0))
    return math.atan2(y1 - y0, x1 - x0)

random.seed(42)
parts = []
stones_per_row = max(int(WIDTH / (STONE_R * 2.2)), 3)
rows = max(int(LENGTH / (STONE_R * 2.2)), 10)

for row in range(rows):
    t = (row + 0.5) / rows
    cx, cy = path_point(t)
    ang = path_angle(t)
    perp = ang + math.pi / 2
    for col in range(stones_per_row):
        offset = (col - (stones_per_row - 1) / 2.0) * STONE_R * 2.2
        jx = random.uniform(-STONE_R * 0.3, STONE_R * 0.3)
        jy = random.uniform(-STONE_R * 0.3, STONE_R * 0.3)
        sx = cx + math.cos(perp) * offset + jx
        sy = cy + math.sin(perp) * offset + jy

        r = STONE_R * random.uniform(0.7, 1.1)
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=r, location=(sx, sy, HEIGHT * 0.5))
        stone = bpy.context.active_object

        bm = bmesh.new()
        bm.from_mesh(stone.data)
        disp = ROUGHNESS * r
        for v in bm.verts:
            v.co.x += random.uniform(-disp, disp)
            v.co.y += random.uniform(-disp, disp)
            v.co.z += random.uniform(-disp * 0.3, disp * 0.3)
            if v.co.z < 0:
                v.co.z = 0
        bm.to_mesh(stone.data)
        bm.free()

        stone.scale = (random.uniform(0.85, 1.15), random.uniform(0.85, 1.15), random.uniform(0.3, 0.5))
        stone.rotation_euler = (0, 0, random.uniform(0, 6.28))
        bpy.ops.object.transform_apply(rotation=True, scale=True)
        parts.append(stone)

bpy.ops.object.select_all(action='DESELECT')
for p in parts:
    p.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()

mod = bpy.context.active_object.modifiers.new('Decimate', 'DECIMATE')
mod.ratio = 0.5
bpy.ops.object.modifier_apply(modifier='Decimate')

bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
"#;

const TEMPLATE_SHIRT_LEGACY: &str = r#"
import bpy
import math
from mathutils.kdtree import KDTree

SLEEVE = {{SLEEVE_LENGTH}}
FIT = "{{FIT}}"
COLLAR = "{{COLLAR}}"

arm_path = "{{ARMATURE_PATH}}"
import os
print(f"BLEND FILE: {arm_path} exists={os.path.isfile(arm_path)}")
bpy.ops.wm.open_mainfile(filepath=arm_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    print(f"  OBJ: {obj.name} type={obj.type} vgroups={len(obj.vertex_groups) if obj.type=='MESH' else 'N/A'}")
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

print(f"ARMATURE: {armature.name if armature else 'NONE'}")
print(f"BODY: {body.name if body else 'NONE'} (verts={body_vc})")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

if armature is None:
    raise RuntimeError("No armature found in blend file")

fit_scale = 1.01 if FIT == "tight" else (1.06 if FIT == "loose" else 1.03)

verts = []
faces = []
rings = 20
segs = 24
torso_h = 0.55
waist_r = 0.125 * fit_scale
chest_r = 0.145 * fit_scale
shoulder_r = 0.065 * fit_scale

collar_offset = 0.02 if COLLAR == "v-neck" else (0.0 if COLLAR == "crew" else 0.01)

for ring in range(rings + 1):
    t = ring / rings
    z = t * torso_h
    if t < 0.3:
        r = waist_r + (chest_r - waist_r) * (t / 0.3)
    elif t < 0.7:
        r = chest_r
    else:
        r = chest_r - (chest_r - shoulder_r) * ((t - 0.7) / 0.3) - collar_offset * t
    for seg in range(segs):
        angle = 2 * math.pi * seg / segs
        x = math.cos(angle) * r
        y = math.sin(angle) * r
        verts.append((x, y, z))

for ring in range(rings):
    for seg in range(segs):
        a = ring * segs + seg
        b = ring * segs + (seg + 1) % segs
        c = (ring + 1) * segs + (seg + 1) % segs
        d = (ring + 1) * segs + seg
        faces.append((a, b, c, d))

if SLEEVE > 0.1:
    sleeve_segs = 12
    sleeve_rings = max(int(8 * SLEEVE), 4)
    sleeve_len = 0.35 * SLEEVE
    for side in [-1, 1]:
        base_idx = len(verts)
        sx = side * 0.19
        sz = torso_h * 0.85
        for ring in range(sleeve_rings + 1):
            t2 = ring / sleeve_rings
            sr = 0.07 * (1.0 - 0.3 * t2)
            for seg in range(sleeve_segs):
                angle = 2 * math.pi * seg / sleeve_segs
                dx = math.cos(angle) * sr
                dy = math.sin(angle) * sr
                verts.append((sx + side * t2 * sleeve_len, dy, sz + dx))
        for ring in range(sleeve_rings):
            for seg in range(sleeve_segs):
                a = base_idx + ring * sleeve_segs + seg
                b = base_idx + ring * sleeve_segs + (seg + 1) % sleeve_segs
                c = base_idx + (ring + 1) * sleeve_segs + (seg + 1) % sleeve_segs
                d = base_idx + (ring + 1) * sleeve_segs + seg
                faces.append((a, b, c, d))

mesh = bpy.data.meshes.new("shirt_mesh")
mesh.from_pydata(verts, [], faces)
mesh.update()

uv_layer = mesh.uv_layers.new(name="UVMap")
for face in mesh.polygons:
    for li in face.loop_indices:
        loop = mesh.loops[li]
        v = mesh.vertices[loop.vertex_index]
        u = math.atan2(v.co.y, v.co.x) / (2 * math.pi) + 0.5
        v_coord = v.co.z / torso_h if torso_h > 0 else 0
        uv_layer.data[li].uv = (u, max(0.0, min(1.0, v_coord)))

mat = bpy.data.materials.new("ShirtMaterial")
mat.use_nodes = True
bsdf = mat.node_tree.nodes["Principled BSDF"]
tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0:
    import os
    if os.path.isfile(tex_path):
        tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
        tex_node.image = bpy.data.images.load(tex_path)
        mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
mesh.materials.append(mat)

shirt = bpy.data.objects.new("Shirt", mesh)
bpy.context.collection.objects.link(shirt)
shirt.location = (0, 0, 0.85)

shirt.parent = armature
mod = shirt.modifiers.new('Armature', 'ARMATURE')
mod.object = armature
bpy.context.view_layer.update()

if body:
    body_mesh = body.data
    kd = KDTree(len(body_mesh.vertices))
    for i, v in enumerate(body_mesh.vertices):
        kd.insert(body.matrix_world @ v.co, i)
    kd.balance()
    vg_names = {vg.index: vg.name for vg in body.vertex_groups}
    body_weights = {}
    for bv in body_mesh.vertices:
        wlist = []
        for ge in bv.groups:
            if ge.weight > 0 and ge.group in vg_names:
                wlist.append((vg_names[ge.group], ge.weight))
        body_weights[bv.index] = wlist
    used_groups = set()
    shirt_weights = {}
    shirt_mesh = shirt.data
    for sv in shirt_mesh.vertices:
        world_co = shirt.matrix_world @ sv.co
        results = kd.find_n(world_co, 4)
        weight_sums = {}
        total_w = 0.0
        for co, idx, dist in results:
            d = max(dist, 0.0001)
            inv_d = 1.0 / d
            total_w += inv_d
            for gname, gw in body_weights.get(idx, []):
                weight_sums[gname] = weight_sums.get(gname, 0.0) + gw * inv_d
        assigns = {}
        if total_w > 0 and weight_sums:
            for name, ws in weight_sums.items():
                w = ws / total_w
                if w > 0.001:
                    assigns[name] = w
                    used_groups.add(name)
        if not assigns:
            assigns["mChest"] = 1.0
            used_groups.add("mChest")
        shirt_weights[sv.index] = assigns
    for gname in used_groups:
        shirt.vertex_groups.new(name=gname)
    for vi, assigns in shirt_weights.items():
        for gname, w in assigns.items():
            shirt.vertex_groups[gname].add([vi], w, 'REPLACE')
    bpy.data.objects.remove(body, do_unlink=True)
else:
    for obj in bpy.context.view_layer.objects:
        obj.select_set(False)
    shirt.select_set(True)
    armature.select_set(True)
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.parent_set(type='ARMATURE_AUTO')

print(f"PRE-EXPORT: shirt={shirt.name if shirt else 'NONE'} armature={armature.name if armature else 'NONE'}")
print(f"  shirt vgroups={len(shirt.vertex_groups)} verts={len(shirt.data.vertices)}")
print(f"  objects in scene: {[o.name for o in bpy.data.objects]}")
bpy.context.view_layer.update()
for obj in bpy.context.view_layer.objects:
    obj.select_set(False)
shirt.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = shirt
print("SELECTION DONE - ready for export")
"#;

const TEMPLATE_PANTS_LEGACY: &str = r#"
import bpy
import math

LEG_LEN = {{LEG_LENGTH}}
FIT = "{{FIT}}"
WAIST = "{{WAIST}}"

bpy.ops.wm.open_mainfile(filepath="{{ARMATURE_PATH}}")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH':
        bpy.data.objects.remove(obj, do_unlink=True)

armature = None
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE':
        armature = obj
        break

if armature is None:
    raise RuntimeError("No armature found in blend file")

fit_scale = 1.02 if FIT == "tight" else (1.06 if FIT == "loose" else 1.04)

waist_offset = 0.02 if WAIST == "high" else (-0.02 if WAIST == "low" else 0.0)

verts = []
faces = []
segs = 12

waist_r = 0.14 * fit_scale
hip_r = 0.16 * fit_scale
waist_z = 0.85 + waist_offset

waist_rings = 4
for ring in range(waist_rings + 1):
    t = ring / waist_rings
    z = waist_z - t * 0.15
    r = waist_r + (hip_r - waist_r) * t
    for seg in range(segs):
        angle = 2 * math.pi * seg / segs
        x = math.cos(angle) * r
        y = math.sin(angle) * r
        verts.append((x, y, z))

for ring in range(waist_rings):
    for seg in range(segs):
        a = ring * segs + seg
        b = ring * segs + (seg + 1) % segs
        c = (ring + 1) * segs + (seg + 1) % segs
        d = (ring + 1) * segs + seg
        faces.append((a, b, c, d))

full_leg_len = 0.75 * LEG_LEN
leg_rings = max(int(10 * LEG_LEN), 4)
thigh_r = 0.08 * fit_scale
knee_r = 0.06 * fit_scale
ankle_r = 0.05 * fit_scale
start_z = waist_z - 0.15
leg_sep = 0.07

for side in [-1, 1]:
    base_idx = len(verts)
    for ring in range(leg_rings + 1):
        t = ring / leg_rings
        z = start_z - t * full_leg_len
        if t < 0.4:
            r = thigh_r - (thigh_r - knee_r) * (t / 0.4)
        else:
            r = knee_r - (knee_r - ankle_r) * ((t - 0.4) / 0.6)
        cx = side * leg_sep
        for seg in range(segs):
            angle = 2 * math.pi * seg / segs
            x = cx + math.cos(angle) * r
            y = math.sin(angle) * r
            verts.append((x, y, z))
    for ring in range(leg_rings):
        for seg in range(segs):
            a = base_idx + ring * segs + seg
            b = base_idx + ring * segs + (seg + 1) % segs
            c = base_idx + (ring + 1) * segs + (seg + 1) % segs
            d = base_idx + (ring + 1) * segs + seg
            faces.append((a, b, c, d))

mesh = bpy.data.meshes.new("pants_mesh")
mesh.from_pydata(verts, [], faces)
mesh.update()
pants = bpy.data.objects.new("Pants", mesh)
bpy.context.collection.objects.link(pants)

bpy.context.view_layer.objects.active = pants
pants.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = armature
bpy.ops.object.parent_set(type='ARMATURE_AUTO')

bpy.ops.object.mode_set(mode='OBJECT')
bpy.ops.object.select_all(action='DESELECT')
pants.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = pants
"#;

const BODYCLONE_PREAMBLE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
print(f"BLEND FILE: {body_path} exists={os.path.isfile(body_path)}")
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

print(f"ARMATURE: {armature.name}")
print(f"BODY: {body.name} (verts={body_vc})")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()

body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min
print(f"BODY Z range: {body_z_min:.3f} to {body_z_max:.3f} (height={body_height:.3f})")
"#;

const BODYCLONE_SELECT_AND_OFFSET: &str = r#"
FIT = "{{FIT}}"
fit_offset = 0.002 if FIT == "tight" else (0.008 if FIT == "loose" else 0.004)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Garment"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + Z_MIN_NORM * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height
print(f"GARMENT Z range: {z_lo:.3f} to {z_hi:.3f}")

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    if INCLUDE_ARMS:
        keep = z_lo <= avg_z <= z_hi
    else:
        avg_x_abs = sum(abs(v.x) for v in world_verts) / len(world_verts)
        keep = z_lo <= avg_z <= z_hi and avg_x_abs < ARM_X_THRESHOLD
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')

bm.verts.ensure_lookup_table()
for v in bm.verts:
    if v.normal.length > 0.001:
        v.co += v.normal.normalized() * fit_offset

bm.to_mesh(garment.data)
bm.free()
garment.data.update()
print(f"GARMENT: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces after clipping")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.context.view_layer.objects.active = garment
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts (ratio={60000.0/vc:.3f})")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0 and os.path.isfile(tex_path):
    mat = bpy.data.materials.new("GarmentMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex_node.image = bpy.data.images.load(tex_path)
    mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
    garment.data.materials.clear()
    garment.data.materials.append(mat)

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
print(f"PRE-EXPORT: garment vgroups={len(garment.vertex_groups)} verts={len(garment.data.vertices)}")
"#;

const TEMPLATE_SHIRT_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
print(f"BLEND FILE: {body_path} exists={os.path.isfile(body_path)}")
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()
body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min

SLEEVE = {{SLEEVE_LENGTH}}
FIT = "{{FIT}}"
COLLAR = "{{COLLAR}}"
Z_MIN_NORM = 0.50
Z_MAX_NORM = 0.88
INCLUDE_ARMS = SLEEVE > 0.05
TORSO_X_LIMIT = 0.14

sleeve_x_cutoff = 0.14 + SLEEVE * 0.49

thickness = 0.003 if FIT == "tight" else (0.008 if FIT == "loose" else 0.005)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Shirt"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + Z_MIN_NORM * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    avg_x_abs = sum(abs(v.x) for v in world_verts) / len(world_verts)
    if INCLUDE_ARMS:
        keep = z_lo <= avg_z <= z_hi
        if keep and avg_x_abs > sleeve_x_cutoff:
            keep = False
    else:
        keep = z_lo <= avg_z <= z_hi and avg_x_abs < TORSO_X_LIMIT
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')
bm.to_mesh(garment.data)
bm.free()
garment.data.update()
print(f"SHIRT after cut: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.transform.rotate(value=math.radians(-90), orient_axis='Z', orient_type='GLOBAL')
bpy.ops.object.mode_set(mode='OBJECT')
print("SHIRT rotated 90 CW around Z (Blender -Y front -> SL +X front)")

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = thickness
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
sol.use_rim = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"SHIRT after solidify: {len(garment.data.vertices)} verts")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0 and os.path.isfile(tex_path):
    mat = bpy.data.materials.new("ShirtMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex_node.image = bpy.data.images.load(tex_path)
    mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
    garment.data.materials.clear()
    garment.data.materials.append(mat)

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"SHIRT UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
print(f"PRE-EXPORT: shirt vgroups={len(garment.vertex_groups)} verts={len(garment.data.vertices)}")
"#;

const TEMPLATE_PANTS_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()
body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min

LEG_LEN = {{LEG_LENGTH}}
FIT = "{{FIT}}"
WAIST = "{{WAIST}}"
Z_MIN_NORM = 0.03
Z_MAX_NORM = 0.47 + (0.02 if WAIST == "high" else (-0.02 if WAIST == "low" else 0.0))

fit_offset = 0.012 if FIT == "tight" else (0.035 if FIT == "loose" else 0.024)
z_min_actual = Z_MIN_NORM + (1.0 - LEG_LEN) * 0.20

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Pants"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + z_min_actual * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    keep = z_lo <= avg_z <= z_hi
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')

bm.to_mesh(garment.data)
bm.free()
garment.data.update()

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = fit_offset
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"PANTS: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.context.view_layer.objects.active = garment
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts (ratio={60000.0/vc:.3f})")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"PANTS UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
"#;

const TEMPLATE_DRESS_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()
body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min

SLEEVE = {{SLEEVE_LENGTH}}
FIT = "{{FIT}}"
HEM = float({{HEM_LENGTH}})
FLARE_AMT = float({{FLARE}})
SIMULATE = "{{SIMULATE}}" == "true"
Z_MIN_NORM = max(0.10, min(0.47, HEM))
Z_MAX_NORM = 0.88
INCLUDE_ARMS = SLEEVE > 0.1
ARM_X_THRESHOLD = 0.14

fit_offset = 0.012 if FIT == "tight" else (0.035 if FIT == "loose" else 0.024)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Dress"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + Z_MIN_NORM * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height
waist_z = body_z_min + 0.47 * body_height

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    if INCLUDE_ARMS:
        keep = z_lo <= avg_z <= z_hi
    else:
        avg_x_abs = sum(abs(v.x) for v in world_verts) / len(world_verts)
        keep = z_lo <= avg_z <= z_hi and avg_x_abs < ARM_X_THRESHOLD
    if SLEEVE < 0.9 and INCLUDE_ARMS:
        avg_x_abs = sum(abs(v.x) for v in world_verts) / len(world_verts)
        sleeve_cutoff = 0.14 + SLEEVE * 0.49
        if avg_x_abs > sleeve_cutoff:
            keep = False
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')

if FLARE_AMT > 0:
    bm.verts.ensure_lookup_table()
    for v in bm.verts:
        world_co = garment.matrix_world @ v.co
        if v.normal.length > 0.001 and world_co.z < waist_z:
            t = (waist_z - world_co.z) / max(waist_z - z_lo, 0.01)
            flare = FLARE_AMT * t * t * 0.05
            horiz_normal = v.normal.copy()
            horiz_normal.z = 0
            if horiz_normal.length > 0.001:
                v.co += horiz_normal.normalized() * flare

bm.to_mesh(garment.data)
bm.free()
garment.data.update()

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = fit_offset
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"DRESS: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.context.view_layer.objects.active = garment
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts (ratio={60000.0/vc:.3f})")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0 and os.path.isfile(tex_path):
    mat = bpy.data.materials.new("DressMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex_node.image = bpy.data.images.load(tex_path)
    mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
    garment.data.materials.clear()
    garment.data.materials.append(mat)

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"DRESS UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
"#;

const TEMPLATE_JACKET_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()
body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min

FIT = "{{FIT}}"
Z_MIN_NORM = 0.40
Z_MAX_NORM = 0.88

fit_offset = 0.012 if FIT == "tight" else (0.035 if FIT == "loose" else 0.024)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Jacket"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + Z_MIN_NORM * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    keep = z_lo <= avg_z <= z_hi
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')

bm.to_mesh(garment.data)
bm.free()
garment.data.update()

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = fit_offset
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"JACKET: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.context.view_layer.objects.active = garment
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts (ratio={60000.0/vc:.3f})")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"JACKET UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
"#;

const TEMPLATE_SKIRT_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()
body_z_coords = [body.matrix_world @ v.co for v in body.data.vertices]
body_z_min = min(v.z for v in body_z_coords)
body_z_max = max(v.z for v in body_z_coords)
body_height = body_z_max - body_z_min

FIT = "{{FIT}}"
HEM = float({{HEM_LENGTH}})
FLARE_AMT = float({{FLARE}})
Z_MIN_NORM = max(0.10, min(0.47, HEM))
Z_MAX_NORM = 0.47

fit_offset = 0.012 if FIT == "tight" else (0.035 if FIT == "loose" else 0.024)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Skirt"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()

z_lo = body_z_min + Z_MIN_NORM * body_height
z_hi = body_z_min + Z_MAX_NORM * body_height

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='DESELECT')
bpy.ops.object.mode_set(mode='OBJECT')

bm = bmesh.new()
bm.from_mesh(garment.data)
bm.verts.ensure_lookup_table()
bm.faces.ensure_lookup_table()

faces_to_delete = []
for face in bm.faces:
    world_verts = [garment.matrix_world @ v.co for v in face.verts]
    avg_z = sum(v.z for v in world_verts) / len(world_verts)
    keep = z_lo <= avg_z <= z_hi
    if not keep:
        faces_to_delete.append(face)

bmesh.ops.delete(bm, geom=faces_to_delete, context='FACES')

if FLARE_AMT > 0:
    bm.verts.ensure_lookup_table()
    for v in bm.verts:
        world_co = garment.matrix_world @ v.co
        if v.normal.length > 0.001:
            t = (z_hi - world_co.z) / max(z_hi - z_lo, 0.01)
            flare = FLARE_AMT * t * t * 0.05
            horiz_normal = v.normal.copy()
            horiz_normal.z = 0
            if horiz_normal.length > 0.001:
                v.co += horiz_normal.normalized() * flare

bm.to_mesh(garment.data)
bm.free()
garment.data.update()

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = fit_offset
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"SKIRT: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.context.view_layer.objects.active = garment
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts (ratio={60000.0/vc:.3f})")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"SKIRT UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
"#;

const TEMPLATE_BODYSUIT_BODYCLONE: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
print(f"BLEND FILE: {body_path} exists={os.path.isfile(body_path)}")
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj != body:
        bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.update()

FIT = "{{FIT}}"
thickness = 0.002 if FIT == "tight" else (0.006 if FIT == "loose" else 0.003)

garment = body.copy()
garment.data = body.data.copy()
garment.name = "Bodysuit"
bpy.context.collection.objects.link(garment)
bpy.context.view_layer.update()
print(f"BODYSUIT clone: {len(garment.data.vertices)} verts, {len(garment.data.polygons)} faces")

bpy.context.view_layer.objects.active = garment
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.transform.rotate(value=math.radians(-90), orient_axis='Z', orient_type='GLOBAL')
bpy.ops.object.mode_set(mode='OBJECT')
print("BODYSUIT rotated 90 CW around Z (Blender -Y front -> SL +X front)")

bpy.context.view_layer.objects.active = garment
sol = garment.modifiers.new('Solidify', 'SOLIDIFY')
sol.thickness = thickness
sol.offset = -1.0
sol.use_even_offset = True
sol.use_quality_normals = True
sol.use_rim = True
bpy.ops.object.modifier_apply(modifier='Solidify')
print(f"BODYSUIT after solidify: {len(garment.data.vertices)} verts")

vc = len(garment.data.vertices)
if vc > 60000:
    dec = garment.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(garment.data.vertices)} verts")

garment.parent = armature
arm_mod = garment.modifiers.new('Armature', 'ARMATURE')
arm_mod.object = armature

tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0 and os.path.isfile(tex_path):
    mat = bpy.data.materials.new("BodysuitMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex_node.image = bpy.data.images.load(tex_path)
    mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
    garment.data.materials.clear()
    garment.data.materials.append(mat)

bpy.data.objects.remove(body, do_unlink=True)

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
garment.select_set(True)
armature.select_set(True)
bpy.context.view_layer.objects.active = garment
uv_layers = garment.data.uv_layers
print(f"BODYSUIT UV: layers={len(uv_layers)} names={[l.name for l in uv_layers]}")
print(f"PRE-EXPORT: bodysuit vgroups={len(garment.vertex_groups)} verts={len(garment.data.vertices)}")
"#;

const TEMPLATE_BODYSUIT_POSED: &str = r#"
import bpy
import bmesh
import math
import os

body_path = "{{BODY_BLEND_PATH}}"
bvh_path = "{{BVH_PATH}}"
frame_num = {{FRAME}}
print(f"STATUE: blend={body_path} bvh={bvh_path} frame={frame_num}")
bpy.ops.wm.open_mainfile(filepath=body_path)

armature = None
body = None
body_vc = 0
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE' and armature is None:
        armature = obj
    elif obj.type == 'MESH' and len(obj.vertex_groups) > 0:
        vc = len(obj.data.vertices)
        if vc > body_vc:
            body = obj
            body_vc = vc

if armature is None:
    raise RuntimeError("No armature found in blend file")
if body is None:
    raise RuntimeError("No body mesh found in blend file")

STATUE_PARTS = ["Ruth2v4Body", "Ruth2v4Head", "Ruth2v4Hands", "Ruth2v4FeetFlat"]
parts_to_join = []
remove_list = []
for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj.name in STATUE_PARTS:
        parts_to_join.append(obj)
    elif obj.type == 'MESH':
        remove_list.append(obj)

for obj in remove_list:
    bpy.data.objects.remove(obj, do_unlink=True)

bpy.context.view_layer.objects.active = body
bpy.ops.object.mode_set(mode='OBJECT')

if len(parts_to_join) > 1:
    for obj in bpy.data.objects:
        obj.select_set(False)
    for p in parts_to_join:
        p.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    body = bpy.context.active_object
    print(f"STATUE joined {len(parts_to_join)} parts -> {len(body.data.vertices)} verts")
elif len(parts_to_join) == 0:
    raise RuntimeError("No body parts found")

bpy.context.view_layer.update()

bpy.context.view_layer.objects.active = body
body.select_set(True)
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.mesh.remove_doubles(threshold=0.001)
bpy.ops.object.mode_set(mode='OBJECT')
print(f"STATUE: merged duplicate verts at seams -> {len(body.data.vertices)} verts")

for mod in list(body.modifiers):
    if mod.type in ('DATA_TRANSFER', 'SURFACE_DEFORM'):
        bpy.ops.object.modifier_remove(modifier=mod.name)
        print(f"STATUE: removed modifier {mod.name}")

has_armature_mod = False
for mod in body.modifiers:
    if mod.type == 'ARMATURE':
        has_armature_mod = True
        break
if not has_armature_mod:
    arm_mod = body.modifiers.new('Armature', 'ARMATURE')
    arm_mod.object = armature
    print("STATUE: added armature modifier")

body.name = "Statue"
bpy.context.view_layer.update()
print(f"STATUE: working directly on body mesh, {len(body.data.vertices)} verts, {len(body.modifiers)} modifiers")

BVH_TO_BENTO = {
    "hip": "mPelvis",
    "abdomen": "mTorso",
    "chest": "mChest",
    "neck": "mNeck",
    "head": "mHead",
    "figureHair": "mSkull",
    "neckDummy": None,
    "lCollar": "mCollarLeft",
    "lShldr": "mShoulderLeft",
    "lForeArm": "mElbowLeft",
    "lHand": "mWristLeft",
    "rCollar": "mCollarRight",
    "rShldr": "mShoulderRight",
    "rForeArm": "mElbowRight",
    "rHand": "mWristRight",
    "lThigh": "mHipLeft",
    "lShin": "mKneeLeft",
    "lFoot": "mAnkleLeft",
    "rThigh": "mHipRight",
    "rShin": "mKneeRight",
    "rFoot": "mAnkleRight",
}

import json as _json
from mathutils import Quaternion as _Quat

pose_json_path = bvh_path.replace('.bvh', '.pose.json') if bvh_path else ''
pose_applied = False

print(f"STATUE: BVH path: '{bvh_path}' exists={os.path.isfile(bvh_path) if bvh_path else False}")
print(f"STATUE: pose JSON: '{pose_json_path}' exists={os.path.isfile(pose_json_path) if pose_json_path else False}")
if pose_json_path and os.path.isfile(pose_json_path):
    print(f"STATUE: pose JSON found — using NEG_90Z verified path (preferred over BVH)")
    _use_pose_json = True
elif bvh_path and os.path.isfile(bvh_path):
    _use_pose_json = False
else:
    _use_pose_json = False

if not _use_pose_json and bvh_path and os.path.isfile(bvh_path):
    bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = None

    existing_arms = set(o.name for o in bpy.data.objects if o.type == 'ARMATURE')

    bpy.ops.import_anim.bvh(
        filepath=bvh_path,
        target='ARMATURE',
        global_scale=0.01,
        frame_start=1,
        use_fps_scale=False,
        update_scene_fps=False,
        update_scene_duration=False,
        use_cyclic=False,
        rotate_mode='NATIVE',
    )
    print(f"STATUE: BVH imported from {bvh_path}")

    bvh_arm = None
    for obj in bpy.data.objects:
        if obj.type == 'ARMATURE' and obj.name not in existing_arms:
            bvh_arm = obj
            break
    if bvh_arm is None:
        for obj in bpy.data.objects:
            if obj.type == 'ARMATURE' and obj != armature:
                bvh_arm = obj
                break
    print(f"STATUE: BVH armature found: {bvh_arm.name if bvh_arm else 'NONE'} (existing: {existing_arms})")

    if bvh_arm:
        frame_end = int(bpy.context.scene.frame_end)
        frame_start = int(bpy.context.scene.frame_start)
        if frame_num >= 0:
            use_frame = frame_start + frame_num
        else:
            use_frame = frame_start + 14
            print(f"STATUE: auto-selecting transitional frame ({use_frame}/{frame_end})")
        bpy.context.scene.frame_set(use_frame)
        bpy.context.view_layer.update()
        print(f"STATUE: using frame {use_frame} (range={frame_start}-{frame_end})")

        bpy.context.view_layer.objects.active = armature
        bpy.ops.object.mode_set(mode='POSE')

        mapped = 0
        for bvh_bone_name, bento_name in BVH_TO_BENTO.items():
            if bento_name is None:
                continue
            bvh_pbone = bvh_arm.pose.bones.get(bvh_bone_name)
            target_pbone = armature.pose.bones.get(bento_name)
            if bvh_pbone and target_pbone:
                con = target_pbone.constraints.new('COPY_ROTATION')
                con.target = bvh_arm
                con.subtarget = bvh_bone_name
                con.target_space = 'WORLD'
                con.owner_space = 'WORLD'
                mapped += 1
                print(f"  CONSTRAINT: {bvh_bone_name} -> {bento_name}")

        bpy.context.view_layer.update()
        print(f"STATUE: {mapped} Copy Rotation constraints added")

        for bvh_bone_name, bento_name in BVH_TO_BENTO.items():
            if bento_name is None:
                continue
            target_pbone = armature.pose.bones.get(bento_name)
            if target_pbone and len(target_pbone.constraints) > 0:
                mat = target_pbone.matrix.copy()
                for con in list(target_pbone.constraints):
                    target_pbone.constraints.remove(con)
                target_pbone.matrix = mat

        if pose_json_path and os.path.isfile(pose_json_path):
            import json as _json2
            from mathutils import Quaternion as _Q2, Vector as _V2
            with open(pose_json_path, 'r', encoding='utf-8') as pf2:
                pj = _json2.load(pf2)
            ploc = pj.get("joints", {}).get("mPelvis", {}).get("location")
            if ploc:
                ppb = armature.pose.bones.get("mPelvis")
                if ppb:
                    rq = ppb.bone.matrix_local.to_quaternion()
                    blend_f = min(1.0, (use_frame - frame_start) / 30.0)
                    sl_v = _V2((ploc[0] * blend_f, ploc[1] * blend_f, ploc[2] * blend_f))
                    ppb.location = rq.inverted() @ sl_v
                    print(f"STATUE: pelvis location (blend={blend_f:.2f}): {ploc}")

        bpy.context.view_layer.update()
        print(f"STATUE: constraints baked and removed")

        bpy.ops.object.mode_set(mode='OBJECT')
        bpy.data.objects.remove(bvh_arm, do_unlink=True)
        pose_applied = True
    else:
        print("STATUE: WARNING — no separate BVH armature found")

elif _use_pose_json and pose_json_path and os.path.isfile(pose_json_path):
    with open(pose_json_path, 'r', encoding='utf-8') as pf:
        pose_data = _json.load(pf)
    joints = pose_data.get("joints", {})
    print(f"STATUE: pose JSON from {pose_json_path} ({len(joints)} joints)")

    from mathutils import Matrix as _Mat, Vector as _Vec

    BONE_ORDER = [
        "mPelvis",
        "mTorso", "mChest", "mNeck", "mHead", "mSkull",
        "mEyeLeft", "mEyeRight",
        "mCollarLeft", "mShoulderLeft", "mElbowLeft", "mWristLeft",
        "mCollarRight", "mShoulderRight", "mElbowRight", "mWristRight",
        "mHipLeft", "mKneeLeft", "mAnkleLeft", "mFootLeft",
        "mHipRight", "mKneeRight", "mAnkleRight", "mFootRight",
    ]

    NEG_90Z = _Quat((0.7071068, 0, 0, -0.7071068))

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')

    for pbone in armature.pose.bones:
        pbone.rotation_mode = 'QUATERNION'
        pbone.rotation_quaternion = _Quat((1, 0, 0, 0))
        pbone.location = _Vec((0, 0, 0))
    bpy.context.view_layer.update()

    mapped = 0
    for bone_name in BONE_ORDER:
        if bone_name not in joints:
            continue
        jdata = joints[bone_name]
        if "rotation" not in jdata:
            continue
        target_pbone = armature.pose.bones.get(bone_name)
        if target_pbone is None:
            continue

        bone = target_pbone.bone
        r = jdata["rotation"]
        sl_q = _Quat((r[3], r[0], r[1], r[2]))

        converted = NEG_90Z @ sl_q @ NEG_90Z.conjugated()

        bone_rest_mat = bone.matrix_local.to_3x3().to_4x4()
        bone_rest_inv = bone_rest_mat.inverted()
        basis_mat = bone_rest_inv @ converted.to_matrix().to_4x4() @ bone_rest_mat
        target_pbone.rotation_quaternion = basis_mat.to_quaternion()

        bpy.context.view_layer.update()
        q = basis_mat.to_quaternion()
        print(f"  MAPPED {bone_name}: basis w={q.w:.4f} x={q.x:.4f} y={q.y:.4f} z={q.z:.4f}")
        mapped += 1

    pelvis_data = joints.get("mPelvis", {})
    pelvis_loc = pelvis_data.get("location")
    if pelvis_loc:
        ppb = armature.pose.bones.get("mPelvis")
        if ppb:
            bone_rest_inv = ppb.bone.matrix_local.to_3x3().inverted()
            sl_loc = _Vec((pelvis_loc[0], pelvis_loc[1], pelvis_loc[2]))
            ppb.location = bone_rest_inv @ sl_loc
            print(f"STATUE: pelvis location SL={sl_loc} -> local={ppb.location}")

    bpy.context.view_layer.update()
    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"STATUE: {mapped} bones mapped via -90deg Z + conjugation")
    pose_applied = True

else:
    print(f"STATUE: no pose file found (bvh='{bvh_path}') — using T-pose")

depsgraph = bpy.context.evaluated_depsgraph_get()
depsgraph.update()
body_eval = body.evaluated_get(depsgraph)
new_mesh = bpy.data.meshes.new_from_object(body_eval)
old_mesh = body.data
body.data = new_mesh
bpy.data.meshes.remove(old_mesh)
print(f"STATUE: baked posed mesh via depsgraph, {len(body.data.vertices)} verts")

for mod in list(body.modifiers):
    bpy.ops.object.modifier_remove(modifier=mod.name)
    print(f"STATUE: removed modifier '{mod.name}' (already baked)")

body.parent = None
if armature:
    bpy.data.objects.remove(armature, do_unlink=True)
    armature = None
    print("STATUE: armature removed (static mesh only)")

body.data.materials.clear()
statue_mat = bpy.data.materials.new("StatueMaterial")
statue_mat.use_nodes = True
body.data.materials.append(statue_mat)
for poly in body.data.polygons:
    poly.material_index = 0
print(f"STATUE: single material, {len(body.data.polygons)} polygons")

for vg in list(body.vertex_groups):
    body.vertex_groups.remove(vg)

tex_path = "{{TEXTURE_PATH}}"
if tex_path and len(tex_path) > 0 and os.path.isfile(tex_path):
    bsdf = statue_mat.node_tree.nodes["Principled BSDF"]
    tex_node = statue_mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex_node.image = bpy.data.images.load(tex_path)
    statue_mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])

vc = len(body.data.vertices)
if vc > 60000:
    dec = body.modifiers.new('SafeDecimate', 'DECIMATE')
    dec.ratio = 60000.0 / vc
    bpy.ops.object.modifier_apply(modifier='SafeDecimate')
    print(f"DECIMATE: {vc} -> {len(body.data.vertices)} verts")

bpy.context.view_layer.update()
bpy.ops.object.select_all(action='DESELECT')
body.select_set(True)
bpy.context.view_layer.objects.active = body
print(f"PRE-EXPORT: statue verts={len(body.data.vertices)} materials={len(body.data.materials)} vgroups={len(body.vertex_groups)}")
"#;

const TEMPLATE_SNAPSHOT_STATUE: &str = r#"
import bpy
import bmesh
import os
import json
import math

manifest_path = "{{MANIFEST_PATH}}"
print(f"SNAPSHOT_STATUE: loading manifest from {manifest_path}")

with open(manifest_path, 'r', encoding='utf-8') as f:
    manifest = json.load(f)

avatar_name = manifest.get("avatar_name", "Avatar")
pieces = manifest.get("pieces", [])
baked_textures = manifest.get("baked_textures", {})
body_blend_path = manifest.get("body_blend_path", "")
pose_json_path = manifest.get("pose_json_path", "")
frame_num = manifest.get("frame", 0)

for obj in list(bpy.data.objects):
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in list(bpy.data.meshes):
    bpy.data.meshes.remove(mesh, do_unlink=True)
for mat in list(bpy.data.materials):
    bpy.data.materials.remove(mat, do_unlink=True)
for img in list(bpy.data.images):
    bpy.data.images.remove(img, do_unlink=True)

armature = None
body_meshes = []

KEEP_MESHES = {"Ruth2v4Body", "Ruth2v4Head", "Ruth2v4Hands", "Ruth2v4FeetFlat", "Ruth2v4EyeBall_L", "Ruth2v4Eyeball_R", "Ruth2v4Eyelashes"}

if body_blend_path and os.path.isfile(body_blend_path):
    bpy.ops.wm.open_mainfile(filepath=body_blend_path)
    remove_objs = []
    for obj in bpy.data.objects:
        if obj.type == 'ARMATURE' and armature is None:
            armature = obj
        elif obj.type == 'MESH' and obj.name in KEEP_MESHES:
            body_meshes.append(obj)
        elif obj.type == 'MESH':
            remove_objs.append(obj)
    for obj in remove_objs:
        bpy.data.objects.remove(obj, do_unlink=True)
    body_names = [m.name for m in body_meshes]
    print(f"SNAPSHOT_STATUE: loaded body blend — armature={armature is not None} body_parts={body_names}")

all_meshes = list(body_meshes)

for piece in pieces:
    dae_path = piece.get("dae_path", "")
    piece_name = piece.get("name", "unknown")
    if not dae_path or not os.path.isfile(dae_path):
        print(f"SNAPSHOT_STATUE: skipping {piece_name} — DAE not found: {dae_path}")
        continue

    existing = set(o.name for o in bpy.data.objects)
    bpy.ops.wm.collada_import(filepath=dae_path)
    new_objs = [o for o in bpy.data.objects if o.name not in existing]

    imported_meshes = [o for o in new_objs if o.type == 'MESH']
    imported_arms = [o for o in new_objs if o.type == 'ARMATURE']

    for arm in imported_arms:
        bpy.data.objects.remove(arm, do_unlink=True)

    textures = piece.get("textures", [])
    for mesh_obj in imported_meshes:
        if armature and len(mesh_obj.vertex_groups) > 0:
            for mod in list(mesh_obj.modifiers):
                if mod.type == 'ARMATURE':
                    bpy.ops.object.select_all(action='DESELECT')
                    mesh_obj.select_set(True)
                    bpy.context.view_layer.objects.active = mesh_obj
                    bpy.ops.object.modifier_remove(modifier=mod.name)
            arm_mod = mesh_obj.modifiers.new('Armature', 'ARMATURE')
            arm_mod.object = armature
            mesh_obj.parent = armature
            print(f"SNAPSHOT_STATUE: bound {piece_name} to Ruth2 armature ({len(mesh_obj.vertex_groups)} vgroups)")

        for fi, tex_path in enumerate(textures):
            if tex_path and os.path.isfile(tex_path) and fi < len(mesh_obj.data.materials):
                mat = mesh_obj.data.materials[fi]
                if mat and mat.use_nodes:
                    bsdf = mat.node_tree.nodes.get("Principled BSDF")
                    if bsdf:
                        tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
                        tex_node.image = bpy.data.images.load(tex_path)
                        mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
            elif tex_path and os.path.isfile(tex_path) and fi >= len(mesh_obj.data.materials):
                new_mat = bpy.data.materials.new(f"{piece_name}_face{fi}")
                new_mat.use_nodes = True
                bsdf = new_mat.node_tree.nodes.get("Principled BSDF")
                if bsdf:
                    tex_node = new_mat.node_tree.nodes.new('ShaderNodeTexImage')
                    tex_node.image = bpy.data.images.load(tex_path)
                    new_mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
                mesh_obj.data.materials.append(new_mat)

        all_meshes.append(mesh_obj)
        print(f"SNAPSHOT_STATUE: imported {piece_name}: {len(mesh_obj.data.vertices)} verts, {len(mesh_obj.data.materials)} materials")

if body_meshes and baked_textures:
    head_tex = baked_textures.get("head", "")
    upper_tex = baked_textures.get("upper", "")
    lower_tex = baked_textures.get("lower", "")
    eyes_tex = baked_textures.get("eyes", "")

    BODY_TEX_MAP = {
        "Ruth2v4Body": upper_tex,
        "Ruth2v4Head": head_tex,
        "Ruth2v4Hands": upper_tex,
        "Ruth2v4FeetFlat": lower_tex,
        "Ruth2v4EyeBall_L": eyes_tex,
        "Ruth2v4Eyeball_R": eyes_tex,
        "Ruth2v4Eyelashes": head_tex,
    }
    for bm in body_meshes:
        tex_path = BODY_TEX_MAP.get(bm.name, "")
        if tex_path and os.path.isfile(tex_path):
            bm.data.materials.clear()
            mat = bpy.data.materials.new(f"Baked_{bm.name}")
            mat.use_nodes = True
            bsdf = mat.node_tree.nodes.get("Principled BSDF")
            if bsdf:
                tex_node = mat.node_tree.nodes.new('ShaderNodeTexImage')
                tex_node.image = bpy.data.images.load(tex_path)
                mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
            bm.data.materials.append(mat)
    print(f"SNAPSHOT_STATUE: applied baked textures to {len(body_meshes)} body parts (head={bool(head_tex)} upper={bool(upper_tex)} lower={bool(lower_tex)} eyes={bool(eyes_tex)})")

world_centers = {}

if armature and pose_json_path and os.path.isfile(pose_json_path):
    from mathutils import Quaternion as _Quat, Matrix as _Mat, Vector as _Vec
    import json as _json2

    with open(pose_json_path, 'r', encoding='utf-8') as pf:
        pose_data = _json2.load(pf)
    joints = pose_data.get("joints", {})

    NEG_90Z = _Quat((0.7071068, 0, 0, -0.7071068))

    BONE_ORDER = [
        "mPelvis",
        "mTorso", "mChest", "mNeck", "mHead", "mSkull",
        "mEyeLeft", "mEyeRight",
        "mCollarLeft", "mShoulderLeft", "mElbowLeft", "mWristLeft",
        "mCollarRight", "mShoulderRight", "mElbowRight", "mWristRight",
        "mHipLeft", "mKneeLeft", "mAnkleLeft", "mFootLeft",
        "mHipRight", "mKneeRight", "mAnkleRight", "mFootRight",
    ]

    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='POSE')
    for pbone in armature.pose.bones:
        pbone.rotation_mode = 'QUATERNION'
        pbone.rotation_quaternion = _Quat((1, 0, 0, 0))
        pbone.location = _Vec((0, 0, 0))
    bpy.context.view_layer.update()

    mapped = 0
    for bone_name in BONE_ORDER:
        if bone_name not in joints:
            continue
        jdata = joints[bone_name]
        if "rotation" not in jdata:
            continue
        target_pbone = armature.pose.bones.get(bone_name)
        if target_pbone is None:
            continue
        bone = target_pbone.bone
        r = jdata["rotation"]
        sl_q = _Quat((r[3], r[0], r[1], r[2]))
        converted = NEG_90Z @ sl_q @ NEG_90Z.conjugated()
        bone_rest_mat = bone.matrix_local.to_3x3().to_4x4()
        bone_rest_inv = bone_rest_mat.inverted()
        basis_mat = bone_rest_inv @ converted.to_matrix().to_4x4() @ bone_rest_mat
        target_pbone.rotation_quaternion = basis_mat.to_quaternion()
        bpy.context.view_layer.update()
        mapped += 1

    pelvis_loc = joints.get("mPelvis", {}).get("location")
    if pelvis_loc:
        ppb = armature.pose.bones.get("mPelvis")
        if ppb:
            bone_rest_inv = ppb.bone.matrix_local.to_3x3().inverted()
            sl_loc = _Vec((pelvis_loc[0], pelvis_loc[1], pelvis_loc[2]))
            ppb.location = bone_rest_inv @ sl_loc

    bpy.context.view_layer.update()
    bpy.ops.object.mode_set(mode='OBJECT')
    print(f"SNAPSHOT_STATUE: pose applied — {mapped} bones via NEG_90Z")

    depsgraph = bpy.context.evaluated_depsgraph_get()
    depsgraph.update()
    baked_count = 0
    world_centers = {}
    for mesh_obj in all_meshes:
        if mesh_obj.name not in [o.name for o in bpy.data.objects]:
            continue
        has_armature_mod = any(m.type == 'ARMATURE' for m in mesh_obj.modifiers)
        if not has_armature_mod:
            continue
        bpy.context.view_layer.objects.active = mesh_obj
        mesh_obj.select_set(True)
        obj_eval = mesh_obj.evaluated_get(depsgraph)
        new_mesh_data = bpy.data.meshes.new_from_object(obj_eval)
        old_mesh_data = mesh_obj.data
        mesh_obj.data = new_mesh_data
        bpy.data.meshes.remove(old_mesh_data)
        mw = mesh_obj.matrix_world
        verts_world = [mw @ v.co for v in new_mesh_data.vertices]
        if verts_world:
            wxs = [v.x for v in verts_world]
            wys = [v.y for v in verts_world]
            wzs = [v.z for v in verts_world]
            wcx = (min(wxs) + max(wxs)) / 2.0
            wcy = (min(wys) + max(wys)) / 2.0
            wcz = (min(wzs) + max(wzs)) / 2.0
            world_centers[mesh_obj.name] = (wcx, wcy, wcz)
            print(f"SNAPSHOT_STATUE: {mesh_obj.name} world center=[{wcx:.4f},{wcy:.4f},{wcz:.4f}] world_range X[{min(wxs):.4f},{max(wxs):.4f}] Y[{min(wys):.4f},{max(wys):.4f}] Z[{min(wzs):.4f},{max(wzs):.4f}]")
        else:
            world_centers[mesh_obj.name] = (0.0, 0.0, 0.0)
        for mod in list(mesh_obj.modifiers):
            bpy.ops.object.select_all(action='DESELECT')
            mesh_obj.select_set(True)
            bpy.context.view_layer.objects.active = mesh_obj
            bpy.ops.object.modifier_remove(modifier=mod.name)
        mesh_obj.parent = None
        baked_count += 1
    print(f"SNAPSHOT_STATUE: pose baked {baked_count}/{len(all_meshes)} meshes")

    bpy.data.objects.remove(armature, do_unlink=True)
    armature = None
    for obj in list(bpy.data.objects):
        if obj.type == 'ARMATURE':
            bpy.data.objects.remove(obj, do_unlink=True)

for mesh_obj in all_meshes:
    for vg in list(mesh_obj.vertex_groups):
        mesh_obj.vertex_groups.remove(vg)

for mesh_obj in all_meshes:
    if mesh_obj.name not in [o.name for o in bpy.data.objects]:
        continue
    bpy.ops.object.select_all(action='DESELECT')
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
print(f"SNAPSHOT_STATUE: applied transforms to {len(all_meshes)} meshes")

output_dir = os.path.dirname(manifest_path)
parts_dir = os.path.join(output_dir, "parts")
os.makedirs(parts_dir, exist_ok=True)

part_manifest = []
total_verts = 0
total_polys = 0
exported = 0

from mathutils import Vector as _ExpVec

for idx, mesh_obj in enumerate(all_meshes):
    if mesh_obj.name not in [o.name for o in bpy.data.objects]:
        continue
    vc = len(mesh_obj.data.vertices)
    pc = len(mesh_obj.data.polygons)
    mc = len(mesh_obj.data.materials)

    if mc > 8:
        for poly in mesh_obj.data.polygons:
            poly.material_index = poly.material_index % 8
        while len(mesh_obj.data.materials) > 8:
            mesh_obj.data.materials.pop(index=8)
        mc = len(mesh_obj.data.materials)

    if vc > 65000:
        bpy.ops.object.select_all(action='DESELECT')
        mesh_obj.select_set(True)
        bpy.context.view_layer.objects.active = mesh_obj
        dec = mesh_obj.modifiers.new('Dec', 'DECIMATE')
        dec.ratio = 64000.0 / vc
        bpy.ops.object.modifier_apply(modifier='Dec')
        vc = len(mesh_obj.data.vertices)

    wc = world_centers.get(mesh_obj.name, (0.0, 0.0, 0.0))
    cx, cy, cz = wc[0], wc[1], wc[2]
    print(f"SNAPSHOT_STATUE: part {idx} '{mesh_obj.name}' world_center=[{cx:.4f},{cy:.4f},{cz:.4f}]")

    bpy.ops.object.select_all(action='DESELECT')
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj

    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.mesh.delete_loose(use_verts=True, use_edges=True, use_faces=False)
    bpy.ops.object.mode_set(mode='OBJECT')

    bpy.ops.object.shade_smooth()

    vc = len(mesh_obj.data.vertices)
    pc = len(mesh_obj.data.polygons)
    print(f"SNAPSHOT_STATUE: preflight {mesh_obj.name}: {vc} verts, {pc} polys after cleanup")

    part_path = os.path.join(parts_dir, f"part_{idx:03d}.obj")
    bpy.ops.wm.obj_export(filepath=part_path, export_selected_objects=True, up_axis='Z', forward_axis='X')

    part_manifest.append({
        "path": part_path,
        "name": mesh_obj.name,
        "verts": vc,
        "mats": mc,
        "offset": [cx, cy, cz],
    })
    total_verts += vc
    total_polys += pc
    exported += 1
    print(f"SNAPSHOT_STATUE: exported part {idx}: {mesh_obj.name} ({vc} verts, {mc} mats, offset=[{cx:.3f},{cy:.3f},{cz:.3f}])")

parts_manifest_path = os.path.join(output_dir, "parts_manifest.json")
import json as _json_out
with open(parts_manifest_path, 'w') as pf:
    _json_out.dump(part_manifest, pf)

print(f"SNAPSHOT_STATUE: COMPLETE — {avatar_name}: {exported} parts, {total_verts} total verts, {total_polys} total polys")
"#;

pub fn ruth2_base_dir() -> Option<String> {
    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let base = format!("{}/../..", instance_dir);
    let parent = format!("{}/../../..", instance_dir);

    let candidates = vec![
        format!("{}/Ruth2_v4", parent),
        format!("{}/Ruth2_v4", base),
        "../Ruth2_v4".to_string(),
        "../../Ruth2_v4".to_string(),
        "../../../Ruth2_v4".to_string(),
    ];

    for c in &candidates {
        let p = std::path::Path::new(c);
        if p.is_dir() {
            return Some(
                p.canonicalize()
                    .unwrap_or_else(|_| p.to_path_buf())
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    None
}

pub fn ruth2_dae_path(part: &str) -> Option<String> {
    let base = ruth2_base_dir()?;
    let filename = match part {
        "full" | "ruth2" => "Ruth2v4.dae",
        "body" => "Ruth2v4Body.dae",
        "headless" => "Ruth2v4Headless.dae",
        "head" => "Ruth2v4Head.dae",
        "head_vneck" => "Ruth2v4HeadVNeck.dae",
        "business" => "Ruth2v4Business.dae",
        "business_headless" => "Ruth2v4BusinessHeadless.dae",
        "hands" => "Ruth2v4Hands.dae",
        "feet_flat" => "Ruth2v4FeetFlat.dae",
        "feet_medium" => "Ruth2v4FeetMedium.dae",
        "feet_high" => "Ruth2v4FeetHigh.dae",
        "eyeballs" => "Ruth2v4Eyeballs.dae",
        "eyelashes" => "Ruth2v4Eyelashes.dae",
        "fingernails_short" => "Ruth2v4FingernailsShort.dae",
        "fingernails_med" => "Ruth2v4FingernailsMed.dae",
        "fingernails_long" => "Ruth2v4FingernailsLong.dae",
        "fingernails_oval" => "Ruth2v4FingernailsOval.dae",
        "fingernails_pointed" => "Ruth2v4FingernailsPointed.dae",
        "feet_flat_toenails" => "Ruth2v4FeetFlatToenails.dae",
        "feet_med_toenails" => "Ruth2v4FeetMedToenails.dae",
        "feet_high_toenails" => "Ruth2v4FeetHighToenails.dae",
        _ => return None,
    };
    let path = format!("{}/DAE/{}", base, filename);
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

pub fn ruth2_uv_path(region: &str) -> Option<String> {
    let base = ruth2_base_dir()?;
    let filename = match region {
        "upper" => "Ruth2v4UV_Upper.png",
        "lower" => "Ruth2v4UV_Lower.png",
        "head" => "Ruth2v4UV_Head.png",
        "eyeball" => "Ruth2v4UV_EyeBall.png",
        "eyelashes" => "Ruth2v4UV_Eyelashes.png",
        "feet_flat" => "Ruth2v4UV_FeetFlat.png",
        "feet_medium" => "Ruth2v4UV_FeetMedium.png",
        "feet_high" => "Ruth2v4UV_FeetHigh.png",
        "fingernails_short" => "Ruth2v4UV_FingernailsShort.png",
        "fingernails_med" => "Ruth2v4UV_FingernailsMed.png",
        "fingernails_long" => "Ruth2v4UV_FingernailsLong.png",
        "fingernails_oval" => "Ruth2v4UV_FingernailsOval.png",
        "fingernails_pointed" => "Ruth2v4UV_FingernailsPointed.png",
        "toenails" => "Ruth2v4UV_Toenails.png",
        _ => return None,
    };
    let path = format!("{}/UV/{}", base, filename);
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

pub fn ruth2_texture_path(region: &str) -> Option<String> {
    let base = ruth2_base_dir()?;
    let filename = match region {
        "upper" => "SL-Avatar-Upper-1024.png",
        "lower" => "SL-Avatar-Lower-1024.png",
        "head" => "SL-Avatar-Head-1024.png",
        _ => return None,
    };
    let path = format!("{}/textures/{}", base, filename);
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

pub fn ruth2_all_dae_parts() -> Vec<&'static str> {
    vec![
        "full",
        "body",
        "headless",
        "head",
        "head_vneck",
        "business",
        "business_headless",
        "hands",
        "feet_flat",
        "feet_medium",
        "feet_high",
        "eyeballs",
        "eyelashes",
        "fingernails_short",
        "fingernails_med",
        "fingernails_long",
        "fingernails_oval",
        "fingernails_pointed",
        "feet_flat_toenails",
        "feet_med_toenails",
        "feet_high_toenails",
    ]
}

pub fn ruth2_all_uv_regions() -> Vec<&'static str> {
    vec![
        "upper",
        "lower",
        "head",
        "eyeball",
        "eyelashes",
        "feet_flat",
        "feet_medium",
        "feet_high",
        "fingernails_short",
        "fingernails_med",
        "fingernails_long",
        "fingernails_oval",
        "fingernails_pointed",
        "toenails",
    ]
}

pub fn available_templates() -> Vec<&'static str> {
    vec![
        "table",
        "chair",
        "shelf",
        "arch",
        "staircase",
        "stone",
        "stone_ring",
        "boulder",
        "column",
        "path",
        "shirt",
        "pants",
        "dress",
        "jacket",
        "skirt",
        "shirt_legacy",
        "pants_legacy",
        "snapshot_statue",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_template_substitution() {
        let mut params = HashMap::new();
        params.insert("WIDTH".to_string(), "2.0".to_string());
        params.insert("HEIGHT".to_string(), "1.0".to_string());
        let script = BlenderWorker::get_template("table", &params).expect("template failed");
        assert!(script.contains("W = 2.0"));
        assert!(script.contains("H = 1.0"));
        assert!(!script.contains("{{WIDTH}}"));
        assert!(!script.contains("{{HEIGHT}}"));
        assert!(script.contains("D = 0.6"), "Default DEPTH should apply");
    }

    #[test]
    fn test_chair_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("chair", &params).expect("template failed");
        assert!(script.contains("SW = 0.5"));
        assert!(script.contains("back"));
    }

    #[test]
    fn test_unknown_template_error() {
        let params = HashMap::new();
        assert!(BlenderWorker::get_template("nonexistent", &params).is_err());
    }

    #[test]
    fn test_all_templates_resolve() {
        let params = HashMap::new();
        for name in available_templates() {
            let result = BlenderWorker::get_template(name, &params);
            assert!(result.is_ok(), "Template '{}' should resolve", name);
            let script = result.unwrap();
            assert!(
                !script.contains("{{"),
                "Template '{}' has unresolved placeholder: {}",
                name,
                script.lines().find(|l| l.contains("{{")).unwrap_or("")
            );
        }
    }

    #[test]
    fn test_shirt_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("shirt", &params).expect("shirt template failed");
        assert!(
            script.contains("Shirt"),
            "Body-clone shirt should create 'Shirt' object"
        );
        assert!(script.contains("body_path"), "Body-clone uses body_path");
    }

    #[test]
    fn test_shirt_legacy_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("shirt_legacy", &params)
            .expect("legacy shirt template failed");
        assert!(script.contains("SLEEVE = 1.0"));
        assert!(script.contains("shirt_mesh"));
    }

    #[test]
    fn test_pants_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("pants", &params).expect("pants template failed");
        assert!(
            script.contains("Pants"),
            "Body-clone pants should create 'Pants' object"
        );
        assert!(script.contains("body_path"), "Body-clone uses body_path");
    }

    #[test]
    fn test_pants_legacy_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("pants_legacy", &params)
            .expect("legacy pants template failed");
        assert!(script.contains("LEG_LEN = 1.0"));
        assert!(script.contains("pants_mesh"));
    }

    #[test]
    fn test_dress_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("dress", &params).expect("dress template failed");
        assert!(
            script.contains("Dress"),
            "Dress template should create 'Dress' object"
        );
        assert!(
            script.contains("FLARE_AMT"),
            "Dress should support FLARE parameter"
        );
    }

    #[test]
    fn test_jacket_template() {
        let params = HashMap::new();
        let script =
            BlenderWorker::get_template("jacket", &params).expect("jacket template failed");
        assert!(
            script.contains("Jacket"),
            "Jacket template should create 'Jacket' object"
        );
    }

    #[test]
    fn test_skirt_template() {
        let params = HashMap::new();
        let script = BlenderWorker::get_template("skirt", &params).expect("skirt template failed");
        assert!(
            script.contains("Skirt"),
            "Skirt template should create 'Skirt' object"
        );
        assert!(
            script.contains("FLARE_AMT"),
            "Skirt should support FLARE parameter"
        );
    }

    #[test]
    fn test_is_clothing_template() {
        assert!(is_clothing_template("shirt"));
        assert!(is_clothing_template("pants"));
        assert!(is_clothing_template("dress"));
        assert!(is_clothing_template("jacket"));
        assert!(is_clothing_template("skirt"));
        assert!(is_clothing_template("shirt_legacy"));
        assert!(is_clothing_template("pants_legacy"));
        assert!(!is_clothing_template("table"));
        assert!(!is_clothing_template("chair"));
    }

    #[test]
    fn test_garment_type_spec() {
        let shirt = GarmentType::Shirt.spec("normal", None);
        assert!((shirt.z_min - 0.47).abs() < 0.01);
        assert!((shirt.z_max - 0.88).abs() < 0.01);
        assert!(shirt.include_arms);
        assert!((shirt.offset - 0.008).abs() < 0.001);

        let tight_shirt = GarmentType::Shirt.spec("tight", None);
        assert!((tight_shirt.offset - 0.004).abs() < 0.001);

        let dress = GarmentType::Dress.spec("normal", Some(0.2));
        assert!((dress.z_min - 0.2).abs() < 0.01);
        assert!(dress.include_arms);

        let pants = GarmentType::Pants.spec("loose", None);
        assert!(!pants.include_arms);
        assert!((pants.offset - 0.012).abs() < 0.001);
    }

    #[test]
    fn test_garment_type_from_name() {
        assert_eq!(GarmentType::from_name("shirt"), Some(GarmentType::Shirt));
        assert_eq!(GarmentType::from_name("dress"), Some(GarmentType::Dress));
        assert_eq!(GarmentType::from_name("pants"), Some(GarmentType::Pants));
        assert_eq!(GarmentType::from_name("jacket"), Some(GarmentType::Jacket));
        assert_eq!(GarmentType::from_name("skirt"), Some(GarmentType::Skirt));
        assert_eq!(GarmentType::from_name("table"), None);
    }

    #[test]
    fn test_body_blend_path_returns_string() {
        let ruth_path = body_blend_path("ruth2");
        assert!(!ruth_path.is_empty());
        let roth_path = body_blend_path("roth2");
        assert!(!roth_path.is_empty());
    }

    #[test]
    fn test_ruth2_base_dir_resolves() {
        let dir = ruth2_base_dir();
        assert!(dir.is_some(), "Ruth2_v4 base directory should resolve");
        let dir_path = dir.unwrap();
        assert!(
            std::path::Path::new(&dir_path).is_dir(),
            "Ruth2_v4 dir should exist: {}",
            dir_path
        );
    }

    #[test]
    fn test_ruth2_dae_path_all_parts() {
        if ruth2_base_dir().is_none() {
            return;
        }
        for part in ruth2_all_dae_parts() {
            let path = ruth2_dae_path(part);
            assert!(path.is_some(), "DAE part '{}' should resolve", part);
            assert!(
                std::path::Path::new(path.as_ref().unwrap()).exists(),
                "DAE file for '{}' should exist: {:?}",
                part,
                path
            );
        }
    }

    #[test]
    fn test_ruth2_uv_path_all_regions() {
        if ruth2_base_dir().is_none() {
            return;
        }
        for region in ruth2_all_uv_regions() {
            let path = ruth2_uv_path(region);
            assert!(path.is_some(), "UV region '{}' should resolve", region);
            assert!(
                std::path::Path::new(path.as_ref().unwrap()).exists(),
                "UV file for '{}' should exist: {:?}",
                region,
                path
            );
        }
    }

    #[test]
    fn test_ruth2_texture_path_all() {
        if ruth2_base_dir().is_none() {
            return;
        }
        for region in &["upper", "lower", "head"] {
            let path = ruth2_texture_path(region);
            assert!(path.is_some(), "SL texture '{}' should resolve", region);
            assert!(
                std::path::Path::new(path.as_ref().unwrap()).exists(),
                "SL texture for '{}' should exist: {:?}",
                region,
                path
            );
        }
    }

    #[test]
    fn test_ruth2_dae_path_unknown_returns_none() {
        assert!(ruth2_dae_path("nonexistent_part").is_none());
    }

    #[test]
    fn test_ruth2_uv_path_unknown_returns_none() {
        assert!(ruth2_uv_path("nonexistent_region").is_none());
    }

    #[test]
    fn test_ruth2_texture_path_unknown_returns_none() {
        assert!(ruth2_texture_path("nonexistent").is_none());
    }

    #[test]
    fn test_all_garment_specs_valid() {
        let fits = ["tight", "normal", "loose"];
        let types = [
            GarmentType::Shirt,
            GarmentType::Dress,
            GarmentType::Pants,
            GarmentType::Jacket,
            GarmentType::Skirt,
        ];
        for gt in &types {
            for fit in &fits {
                let spec = gt.spec(fit, None);
                assert!(
                    spec.z_min < spec.z_max,
                    "{:?} {} has z_min >= z_max",
                    gt,
                    fit
                );
                assert!(spec.offset > 0.0, "{:?} {} has zero offset", gt, fit);
            }
        }
    }
}
