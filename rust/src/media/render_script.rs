use std::path::Path;
use anyhow::Result;

use super::RenderJob;
use crate::ai::npc_avatar::{CameraWaypoint, CinemaLight};

pub fn generate_render_script(job: &RenderJob, meshes_dir: &Path, frames_dir: &Path) -> Result<String> {
    let mut script = String::with_capacity(8192);

    script.push_str("import bpy\nimport os\nimport math\n\n");

    script.push_str("bpy.ops.wm.read_factory_settings(use_empty=True)\n");
    script.push_str("for obj in bpy.data.objects:\n    bpy.data.objects.remove(obj)\n\n");

    let meshes_str = meshes_dir.display().to_string().replace('\\', "/");
    let frames_str = frames_dir.display().to_string().replace('\\', "/");

    script.push_str(&format!("meshes_dir = '{}'\n", meshes_str));
    script.push_str(&format!("frames_dir = '{}'\n\n", frames_str));

    script.push_str("import glob\nobj_files = sorted(glob.glob(os.path.join(meshes_dir, '*.obj')))\n");
    script.push_str("for obj_file in obj_files:\n");
    script.push_str("    bpy.ops.wm.obj_import(filepath=obj_file)\n\n");

    script.push_str("manifest_path = os.path.join(meshes_dir, 'manifest.json')\n");
    script.push_str("import json\n");
    script.push_str("if os.path.exists(manifest_path):\n");
    script.push_str("    with open(manifest_path, 'r') as f:\n");
    script.push_str("        manifest = json.load(f)\n");
    script.push_str("    imported = [o for o in bpy.data.objects if o.type == 'MESH']\n");
    script.push_str("    for i, obj_data in enumerate(manifest.get('objects', [])):\n");
    script.push_str("        if i < len(imported):\n");
    script.push_str("            obj = imported[i]\n");
    script.push_str("            pos = obj_data.get('position', [0,0,0])\n");
    script.push_str("            obj.location = (pos[0], pos[1], pos[2])\n");
    script.push_str("            scale = obj_data.get('scale', [1,1,1])\n");
    script.push_str("            obj.scale = (scale[0], scale[1], scale[2])\n");
    script.push_str("            rot = obj_data.get('rotation', [0,0,0,1])\n");
    script.push_str("            obj.rotation_mode = 'QUATERNION'\n");
    script.push_str("            obj.rotation_quaternion = (rot[3], rot[0], rot[1], rot[2])\n\n");

    script.push_str(&generate_camera_script(&job.waypoints, job.settings.frames_per_waypoint));

    for (i, light) in job.lights.iter().enumerate() {
        script.push_str(&generate_light_script(light, i));
    }

    if job.lights.is_empty() {
        script.push_str("sun = bpy.data.lights.new(name='Sun', type='SUN')\n");
        script.push_str("sun_obj = bpy.data.objects.new('Sun', sun)\n");
        script.push_str("bpy.context.scene.collection.objects.link(sun_obj)\n");
        script.push_str("sun.energy = 3.0\n");
        script.push_str("sun_obj.rotation_euler = (0.8, 0.2, 0.5)\n\n");
    }

    script.push_str(&generate_render_settings_script(job, &frames_str));

    script.push_str("bpy.ops.render.render(animation=True)\n");
    script.push_str("print('Render complete')\n");

    Ok(script)
}

fn generate_camera_script(waypoints: &[CameraWaypoint], frames_per_wp: u32) -> String {
    let mut s = String::new();

    s.push_str("cam_data = bpy.data.cameras.new(name='Camera')\n");
    s.push_str("cam_obj = bpy.data.objects.new('Camera', cam_data)\n");
    s.push_str("bpy.context.scene.collection.objects.link(cam_obj)\n");
    s.push_str("bpy.context.scene.camera = cam_obj\n\n");

    if waypoints.is_empty() {
        s.push_str("cam_obj.location = (128, 128, 35)\n");
        s.push_str("cam_data.lens = 35\n");
        return s;
    }

    for (i, wp) in waypoints.iter().enumerate() {
        let frame = i as u32 * frames_per_wp + 1;

        s.push_str(&format!("cam_obj.location = ({}, {}, {})\n", wp.position[0], wp.position[1], wp.position[2]));
        s.push_str(&format!("cam_obj.keyframe_insert(data_path='location', frame={})\n", frame));

        let dx = wp.focus[0] - wp.position[0];
        let dy = wp.focus[1] - wp.position[1];
        let dz = wp.focus[2] - wp.position[2];
        let dist = (dx*dx + dy*dy + dz*dz).sqrt().max(0.001);
        let pitch = (dz / dist).asin();
        let yaw = dy.atan2(dx);
        s.push_str(&format!(
            "cam_obj.rotation_euler = ({:.4}, 0, {:.4})\n",
            std::f32::consts::FRAC_PI_2 - pitch, yaw + std::f32::consts::FRAC_PI_2
        ));
        s.push_str(&format!("cam_obj.rotation_mode = 'XYZ'\n"));
        s.push_str(&format!("cam_obj.keyframe_insert(data_path='rotation_euler', frame={})\n", frame));

        let focal_length = fov_to_focal_length(wp.fov);
        s.push_str(&format!("cam_data.lens = {:.2}\n", focal_length));
        s.push_str(&format!("cam_data.keyframe_insert(data_path='lens', frame={})\n", frame));

        if wp.dwell > 0.0 {
            let hold_frame = frame + (wp.dwell * 24.0) as u32;
            s.push_str(&format!("cam_obj.keyframe_insert(data_path='location', frame={})\n", hold_frame));
            s.push_str(&format!("cam_obj.keyframe_insert(data_path='rotation_euler', frame={})\n", hold_frame));
        }
    }
    s.push('\n');
    s
}

fn generate_light_script(light: &CinemaLight, index: usize) -> String {
    let mut s = String::new();
    let name = format!("Light_{}", index);

    s.push_str(&format!("light_data = bpy.data.lights.new(name='{}', type='POINT')\n", name));
    s.push_str(&format!("light_obj = bpy.data.objects.new('{}', light_data)\n", name));
    s.push_str("bpy.context.scene.collection.objects.link(light_obj)\n");
    s.push_str(&format!("light_obj.location = ({}, {}, {})\n",
        light.position[0], light.position[1], light.position[2]));
    s.push_str(&format!("light_data.energy = {}\n", light.intensity * 1000.0));
    s.push_str(&format!("light_data.color = ({}, {}, {})\n",
        light.color[0], light.color[1], light.color[2]));
    s.push_str(&format!("light_data.shadow_soft_size = {}\n", light.radius));
    s.push_str(&format!("light_data.use_shadow = True\n\n"));

    s
}

fn generate_render_settings_script(job: &RenderJob, frames_str: &str) -> String {
    let mut s = String::new();
    let (w, h) = job.settings.resolution;
    let total_frames = if job.waypoints.is_empty() { 1 } else {
        job.waypoints.len() as u32 * job.settings.frames_per_waypoint
    };

    s.push_str("scene = bpy.context.scene\n");
    s.push_str(&format!("scene.render.resolution_x = {}\n", w));
    s.push_str(&format!("scene.render.resolution_y = {}\n", h));
    s.push_str("scene.render.resolution_percentage = 100\n");
    s.push_str(&format!("scene.render.fps = {}\n", job.settings.fps));
    s.push_str(&format!("scene.frame_start = 1\n"));
    s.push_str(&format!("scene.frame_end = {}\n", total_frames));

    match job.settings.render_engine.as_str() {
        "CYCLES" => {
            s.push_str("scene.render.engine = 'CYCLES'\n");
            s.push_str(&format!("scene.cycles.samples = {}\n", job.settings.samples));
            s.push_str("scene.cycles.use_denoising = True\n");
        }
        _ => {
            s.push_str("scene.render.engine = 'BLENDER_EEVEE_NEXT'\n");
            s.push_str(&format!("scene.eevee.taa_render_samples = {}\n", job.settings.samples));
        }
    }

    s.push_str("scene.render.image_settings.file_format = 'PNG'\n");
    s.push_str("scene.render.image_settings.color_mode = 'RGBA'\n");
    s.push_str(&format!("scene.render.filepath = os.path.join('{}', 'frame_')\n\n", frames_str));

    s
}

fn fov_to_focal_length(fov_degrees: f32) -> f32 {
    let sensor_width = 36.0_f32;
    let fov_rad = fov_degrees.to_radians();
    sensor_width / (2.0 * (fov_rad / 2.0).tan())
}

pub fn generate_save_blend_script(blend_path: &std::path::Path) -> String {
    format!(
        "import bpy\nbpy.ops.wm.save_as_mainfile(filepath='{}')\nprint('Blend file saved')\n",
        blend_path.display().to_string().replace('\\', "/")
    )
}

pub fn generate_compositor_script(
    render_image_path: &Path,
    title: Option<&str>,
    subtitle: Option<&str>,
    output_path: &Path,
    resolution: (u32, u32),
) -> String {
    let mut s = String::with_capacity(4096);
    let (w, h) = resolution;

    s.push_str("import bpy\n\n");
    s.push_str("bpy.ops.wm.read_factory_settings(use_empty=True)\n");
    s.push_str("scene = bpy.context.scene\n");
    s.push_str(&format!("scene.render.resolution_x = {}\n", w));
    s.push_str(&format!("scene.render.resolution_y = {}\n", h));
    s.push_str("scene.render.resolution_percentage = 100\n");
    s.push_str("scene.use_nodes = True\n");
    s.push_str("tree = scene.node_tree\n");
    s.push_str("for node in tree.nodes:\n    tree.nodes.remove(node)\n\n");

    let img_path_str = render_image_path.display().to_string().replace('\\', "/");
    s.push_str(&format!("img = bpy.data.images.load('{}')\n", img_path_str));
    s.push_str("img_node = tree.nodes.new('CompositorNodeImage')\n");
    s.push_str("img_node.image = img\n");
    s.push_str("img_node.location = (0, 300)\n\n");

    s.push_str("comp_node = tree.nodes.new('CompositorNodeComposite')\n");
    s.push_str("comp_node.location = (800, 300)\n\n");

    s.push_str("cc_node = tree.nodes.new('CompositorNodeColorCorrection')\n");
    s.push_str("cc_node.location = (300, 300)\n");
    s.push_str("cc_node.master_saturation = 1.1\n");
    s.push_str("cc_node.master_gain = 1.05\n\n");

    s.push_str("tree.links.new(img_node.outputs[0], cc_node.inputs[0])\n");
    s.push_str("tree.links.new(cc_node.outputs[0], comp_node.inputs[0])\n\n");

    if title.is_some() || subtitle.is_some() {
        s.push_str("# Text overlays via image save with metadata\n");
        s.push_str("# (Blender compositor doesn't have native text nodes;\n");
        s.push_str("#  text will be burned in via post-processing if GIMP available)\n\n");
    }

    let out_path_str = output_path.display().to_string().replace('\\', "/");
    s.push_str("scene.render.image_settings.file_format = 'PNG'\n");
    s.push_str("scene.render.image_settings.color_mode = 'RGBA'\n");
    s.push_str(&format!("scene.render.filepath = '{}'\n", out_path_str));
    s.push_str("scene.frame_start = 1\n");
    s.push_str("scene.frame_end = 1\n");
    s.push_str("bpy.ops.render.render(write_still=True)\n");
    s.push_str("print('Compositor output complete')\n");

    s
}

pub fn generate_photo_script(
    meshes_dir: &Path,
    camera_pos: [f32; 3],
    camera_focus: [f32; 3],
    fov: f32,
    f_stop: f32,
    lights: &[CinemaLight],
    output_path: &Path,
    resolution: (u32, u32),
    samples: u32,
) -> String {
    let mut s = String::with_capacity(4096);
    let (w, h) = resolution;
    let meshes_str = meshes_dir.display().to_string().replace('\\', "/");
    let out_str = output_path.display().to_string().replace('\\', "/");

    s.push_str("import bpy\nimport os\nimport glob\nimport json\nimport math\n\n");
    s.push_str("bpy.ops.wm.read_factory_settings(use_empty=True)\n");
    s.push_str("for obj in bpy.data.objects:\n    bpy.data.objects.remove(obj)\n\n");

    s.push_str(&format!("meshes_dir = '{}'\n", meshes_str));
    s.push_str("obj_files = sorted(glob.glob(os.path.join(meshes_dir, '*.obj')))\n");
    s.push_str("for obj_file in obj_files:\n    bpy.ops.wm.obj_import(filepath=obj_file)\n\n");

    s.push_str("manifest_path = os.path.join(meshes_dir, 'manifest.json')\n");
    s.push_str("if os.path.exists(manifest_path):\n");
    s.push_str("    with open(manifest_path, 'r') as f:\n");
    s.push_str("        manifest = json.load(f)\n");
    s.push_str("    imported = [o for o in bpy.data.objects if o.type == 'MESH']\n");
    s.push_str("    for i, obj_data in enumerate(manifest.get('objects', [])):\n");
    s.push_str("        if i < len(imported):\n");
    s.push_str("            obj = imported[i]\n");
    s.push_str("            pos = obj_data.get('position', [0,0,0])\n");
    s.push_str("            obj.location = (pos[0], pos[1], pos[2])\n");
    s.push_str("            scale = obj_data.get('scale', [1,1,1])\n");
    s.push_str("            obj.scale = (scale[0], scale[1], scale[2])\n\n");

    s.push_str("cam_data = bpy.data.cameras.new(name='PhotoCam')\n");
    s.push_str("cam_obj = bpy.data.objects.new('PhotoCam', cam_data)\n");
    s.push_str("bpy.context.scene.collection.objects.link(cam_obj)\n");
    s.push_str("bpy.context.scene.camera = cam_obj\n");
    s.push_str(&format!("cam_obj.location = ({}, {}, {})\n", camera_pos[0], camera_pos[1], camera_pos[2]));

    let dx = camera_focus[0] - camera_pos[0];
    let dy = camera_focus[1] - camera_pos[1];
    let dz = camera_focus[2] - camera_pos[2];
    let dist = (dx*dx + dy*dy + dz*dz).sqrt().max(0.001);
    let pitch = (dz / dist).asin();
    let yaw = dy.atan2(dx);
    s.push_str(&format!("cam_obj.rotation_mode = 'XYZ'\n"));
    s.push_str(&format!("cam_obj.rotation_euler = ({:.4}, 0, {:.4})\n",
        std::f32::consts::FRAC_PI_2 - pitch, yaw + std::f32::consts::FRAC_PI_2));

    let focal_length = fov_to_focal_length(fov);
    s.push_str(&format!("cam_data.lens = {:.2}\n", focal_length));
    s.push_str("cam_data.dof.use_dof = True\n");
    s.push_str(&format!("cam_data.dof.aperture_fstop = {:.1}\n\n", f_stop));

    for (i, light) in lights.iter().enumerate() {
        s.push_str(&generate_light_script(light, i));
    }

    if lights.is_empty() {
        s.push_str("sun = bpy.data.lights.new(name='Sun', type='SUN')\n");
        s.push_str("sun_obj = bpy.data.objects.new('Sun', sun)\n");
        s.push_str("bpy.context.scene.collection.objects.link(sun_obj)\n");
        s.push_str("sun.energy = 3.0\n");
        s.push_str("sun_obj.rotation_euler = (0.8, 0.2, 0.5)\n\n");
    }

    s.push_str("scene = bpy.context.scene\n");
    s.push_str(&format!("scene.render.resolution_x = {}\n", w));
    s.push_str(&format!("scene.render.resolution_y = {}\n", h));
    s.push_str("scene.render.resolution_percentage = 100\n");
    s.push_str("scene.render.engine = 'CYCLES'\n");
    s.push_str(&format!("scene.cycles.samples = {}\n", samples));
    s.push_str("scene.cycles.use_denoising = True\n");
    s.push_str("scene.render.image_settings.file_format = 'PNG'\n");
    s.push_str("scene.render.image_settings.color_mode = 'RGBA'\n");
    s.push_str(&format!("scene.render.filepath = '{}'\n", out_str));
    s.push_str("bpy.ops.render.render(write_still=True)\n");
    s.push_str("print('Photo render complete')\n");

    s
}
