use anyhow::Result;
use std::fmt::Write as FmtWrite;

use super::encoder::{MeshGeometry, MeshSkinInfo};

pub struct DaeWriterOptions {
    pub mesh_name: String,
    pub texture_files: Vec<Option<String>>,
    pub up_axis: UpAxis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpAxis {
    YUp,
    ZUp,
}

impl Default for DaeWriterOptions {
    fn default() -> Self {
        Self {
            mesh_name: "mesh".to_string(),
            texture_files: Vec::new(),
            up_axis: UpAxis::ZUp,
        }
    }
}

pub fn write_dae(geometry: &MeshGeometry, options: &DaeWriterOptions) -> Result<String> {
    let mut xml = String::with_capacity(64 * 1024);

    write_header(&mut xml, options)?;
    write_images(&mut xml, options)?;
    write_effects(&mut xml, geometry, options)?;
    write_materials(&mut xml, geometry, options)?;
    write_geometries(&mut xml, geometry, options)?;

    if let Some(ref skin) = geometry.skin_info {
        write_controllers(&mut xml, geometry, skin, options)?;
    }

    write_visual_scene(&mut xml, geometry, options)?;
    write_footer(&mut xml)?;

    Ok(xml)
}

fn write_header(xml: &mut String, options: &DaeWriterOptions) -> Result<()> {
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">\n");
    xml.push_str("  <asset>\n");
    xml.push_str("    <contributor>\n");
    xml.push_str("      <author>OpenSim Next Snapshot Statue Pipeline</author>\n");
    xml.push_str("    </contributor>\n");
    xml.push_str("    <created>2026-01-01T00:00:00</created>\n");
    match options.up_axis {
        UpAxis::YUp => xml.push_str("    <up_axis>Y_UP</up_axis>\n"),
        UpAxis::ZUp => xml.push_str("    <up_axis>Z_UP</up_axis>\n"),
    }
    xml.push_str("    <unit name=\"meter\" meter=\"1.0\"/>\n");
    xml.push_str("  </asset>\n");
    Ok(())
}

fn write_images(xml: &mut String, options: &DaeWriterOptions) -> Result<()> {
    let has_images = options.texture_files.iter().any(|t| t.is_some());
    if !has_images {
        return Ok(());
    }

    xml.push_str("  <library_images>\n");
    for (i, tex) in options.texture_files.iter().enumerate() {
        if let Some(ref path) = tex {
            write!(xml, "    <image id=\"image_face{i}\" name=\"tex_face{i}\">\n")?;
            write!(xml, "      <init_from>{path}</init_from>\n")?;
            xml.push_str("    </image>\n");
        }
    }
    xml.push_str("  </library_images>\n");
    Ok(())
}

fn write_effects(xml: &mut String, geometry: &MeshGeometry, options: &DaeWriterOptions) -> Result<()> {
    xml.push_str("  <library_effects>\n");
    for i in 0..geometry.faces.len() {
        write!(xml, "    <effect id=\"effect_face{i}\">\n")?;
        xml.push_str("      <profile_COMMON>\n");
        if let Some(Some(_)) = options.texture_files.get(i) {
            write!(xml, "        <newparam sid=\"surface_face{i}\">\n")?;
            xml.push_str("          <surface type=\"2D\">\n");
            write!(xml, "            <init_from>image_face{i}</init_from>\n")?;
            xml.push_str("          </surface>\n");
            xml.push_str("        </newparam>\n");
            write!(xml, "        <newparam sid=\"sampler_face{i}\">\n")?;
            xml.push_str("          <sampler2D>\n");
            write!(xml, "            <source>surface_face{i}</source>\n")?;
            xml.push_str("          </sampler2D>\n");
            xml.push_str("        </newparam>\n");
        }
        xml.push_str("        <technique sid=\"common\">\n");
        xml.push_str("          <phong>\n");
        xml.push_str("            <diffuse>\n");
        if let Some(Some(_)) = options.texture_files.get(i) {
            write!(xml, "              <texture texture=\"sampler_face{i}\" texcoord=\"UVMap\"/>\n")?;
        } else {
            xml.push_str("              <color>0.8 0.8 0.8 1.0</color>\n");
        }
        xml.push_str("            </diffuse>\n");
        xml.push_str("          </phong>\n");
        xml.push_str("        </technique>\n");
        xml.push_str("      </profile_COMMON>\n");
        xml.push_str("    </effect>\n");
    }
    xml.push_str("  </library_effects>\n");
    Ok(())
}

fn write_materials(xml: &mut String, geometry: &MeshGeometry, _options: &DaeWriterOptions) -> Result<()> {
    xml.push_str("  <library_materials>\n");
    for i in 0..geometry.faces.len() {
        write!(xml, "    <material id=\"material_face{i}\" name=\"Face{i}\">\n")?;
        write!(xml, "      <instance_effect url=\"#effect_face{i}\"/>\n")?;
        xml.push_str("    </material>\n");
    }
    xml.push_str("  </library_materials>\n");
    Ok(())
}

fn write_geometries(xml: &mut String, geometry: &MeshGeometry, options: &DaeWriterOptions) -> Result<()> {
    let name = &options.mesh_name;
    xml.push_str("  <library_geometries>\n");
    write!(xml, "    <geometry id=\"{name}-mesh\" name=\"{name}\">\n")?;
    xml.push_str("      <mesh>\n");

    let mut vertex_offsets: Vec<usize> = Vec::new();
    let mut total_positions = 0usize;
    let mut total_normals = 0usize;
    let mut total_uvs = 0usize;
    let has_normals = geometry.faces.iter().any(|f| !f.normals.is_empty());
    let has_uvs = geometry.faces.iter().any(|f| !f.tex_coords.is_empty());

    for face in &geometry.faces {
        vertex_offsets.push(total_positions);
        total_positions += face.positions.len();
        total_normals += face.normals.len();
        total_uvs += face.tex_coords.len();
    }

    let pos_id = format!("{name}-positions");
    write!(xml, "        <source id=\"{pos_id}\">\n")?;
    write!(xml, "          <float_array id=\"{pos_id}-array\" count=\"{}\">", total_positions * 3)?;
    let mut first = true;
    for face in &geometry.faces {
        for p in &face.positions {
            if !first { xml.push(' '); }
            first = false;
            write!(xml, "{} {} {}", p[0], p[1], p[2])?;
        }
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{pos_id}-array\" count=\"{total_positions}\" stride=\"3\">\n")?;
    xml.push_str("              <param name=\"X\" type=\"float\"/>\n");
    xml.push_str("              <param name=\"Y\" type=\"float\"/>\n");
    xml.push_str("              <param name=\"Z\" type=\"float\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    if has_normals {
        let norm_id = format!("{name}-normals");
        write!(xml, "        <source id=\"{norm_id}\">\n")?;
        write!(xml, "          <float_array id=\"{norm_id}-array\" count=\"{}\">", total_normals * 3)?;
        let mut first = true;
        for face in &geometry.faces {
            for n in &face.normals {
                if !first { xml.push(' '); }
                first = false;
                write!(xml, "{} {} {}", n[0], n[1], n[2])?;
            }
        }
        xml.push_str("</float_array>\n");
        xml.push_str("          <technique_common>\n");
        write!(xml, "            <accessor source=\"#{norm_id}-array\" count=\"{total_normals}\" stride=\"3\">\n")?;
        xml.push_str("              <param name=\"X\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"Y\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"Z\" type=\"float\"/>\n");
        xml.push_str("            </accessor>\n");
        xml.push_str("          </technique_common>\n");
        xml.push_str("        </source>\n");
    }

    if has_uvs {
        let uv_id = format!("{name}-map0");
        write!(xml, "        <source id=\"{uv_id}\">\n")?;
        write!(xml, "          <float_array id=\"{uv_id}-array\" count=\"{}\">", total_uvs * 2)?;
        let mut first = true;
        for face in &geometry.faces {
            for uv in &face.tex_coords {
                if !first { xml.push(' '); }
                first = false;
                write!(xml, "{} {}", uv[0], uv[1])?;
            }
        }
        xml.push_str("</float_array>\n");
        xml.push_str("          <technique_common>\n");
        write!(xml, "            <accessor source=\"#{uv_id}-array\" count=\"{total_uvs}\" stride=\"2\">\n")?;
        xml.push_str("              <param name=\"S\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"T\" type=\"float\"/>\n");
        xml.push_str("            </accessor>\n");
        xml.push_str("          </technique_common>\n");
        xml.push_str("        </source>\n");
    }

    let vert_id = format!("{name}-vertices");
    write!(xml, "        <vertices id=\"{vert_id}\">\n")?;
    write!(xml, "          <input semantic=\"POSITION\" source=\"#{pos_id}\"/>\n")?;
    xml.push_str("        </vertices>\n");

    for (fi, face) in geometry.faces.iter().enumerate() {
        let base = vertex_offsets[fi];
        let tri_count = face.indices.len() / 3;
        write!(xml, "        <triangles material=\"material_face{fi}\" count=\"{tri_count}\">\n")?;

        let mut input_offset = 0;
        write!(xml, "          <input semantic=\"VERTEX\" source=\"#{vert_id}\" offset=\"{input_offset}\"/>\n")?;
        input_offset += 1;

        if has_normals {
            let norm_id = format!("{name}-normals");
            write!(xml, "          <input semantic=\"NORMAL\" source=\"#{norm_id}\" offset=\"{input_offset}\"/>\n")?;
            input_offset += 1;
        }

        if has_uvs {
            let uv_id = format!("{name}-map0");
            write!(xml, "          <input semantic=\"TEXCOORD\" source=\"#{uv_id}\" offset=\"{input_offset}\" set=\"0\"/>\n")?;
            input_offset += 1;
        }

        let stride = input_offset;
        xml.push_str("          <p>");
        for (ti, idx) in face.indices.iter().enumerate() {
            if ti > 0 { xml.push(' '); }
            let v = *idx as usize + base;
            for _s in 0..stride {
                if _s > 0 { xml.push(' '); }
                write!(xml, "{v}")?;
            }
        }
        xml.push_str("</p>\n");
        xml.push_str("        </triangles>\n");
    }

    xml.push_str("      </mesh>\n");
    xml.push_str("    </geometry>\n");
    xml.push_str("  </library_geometries>\n");
    Ok(())
}

fn write_controllers(
    xml: &mut String,
    geometry: &MeshGeometry,
    skin: &MeshSkinInfo,
    options: &DaeWriterOptions,
) -> Result<()> {
    let name = &options.mesh_name;
    xml.push_str("  <library_controllers>\n");
    write!(xml, "    <controller id=\"{name}-skin\" name=\"{name}-skin\">\n")?;
    write!(xml, "      <skin source=\"#{name}-mesh\">\n")?;

    xml.push_str("        <bind_shape_matrix>");
    for (i, v) in skin.bind_shape_matrix.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{v}")?;
    }
    xml.push_str("</bind_shape_matrix>\n");

    let joint_count = skin.joint_names.len();
    write!(xml, "        <source id=\"{name}-skin-joints\">\n")?;
    write!(xml, "          <Name_array id=\"{name}-skin-joints-array\" count=\"{joint_count}\">")?;
    for (i, jn) in skin.joint_names.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        xml.push_str(jn);
    }
    xml.push_str("</Name_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-joints-array\" count=\"{joint_count}\" stride=\"1\">\n")?;
    xml.push_str("              <param name=\"JOINT\" type=\"name\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    let ibm_count = skin.inverse_bind_matrices.len() * 16;
    write!(xml, "        <source id=\"{name}-skin-bind-poses\">\n")?;
    write!(xml, "          <float_array id=\"{name}-skin-bind-poses-array\" count=\"{ibm_count}\">")?;
    for (mi, mat) in skin.inverse_bind_matrices.iter().enumerate() {
        if mi > 0 { xml.push(' '); }
        for (vi, v) in mat.iter().enumerate() {
            if vi > 0 { xml.push(' '); }
            write!(xml, "{v}")?;
        }
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-bind-poses-array\" count=\"{}\" stride=\"16\">\n", skin.inverse_bind_matrices.len())?;
    xml.push_str("              <param name=\"TRANSFORM\" type=\"float4x4\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    let mut all_weights: Vec<f32> = Vec::new();
    let mut vcounts: Vec<usize> = Vec::new();
    let mut v_pairs: Vec<String> = Vec::new();

    for face in &geometry.faces {
        if let Some(ref jw) = face.joint_weights {
            for vw in jw {
                vcounts.push(vw.influences.len());
                for inf in &vw.influences {
                    let weight_idx = all_weights.len();
                    all_weights.push(inf.weight);
                    v_pairs.push(format!("{} {weight_idx}", inf.joint_index));
                }
            }
        }
    }

    let weight_count = all_weights.len();
    write!(xml, "        <source id=\"{name}-skin-weights\">\n")?;
    write!(xml, "          <float_array id=\"{name}-skin-weights-array\" count=\"{weight_count}\">")?;
    for (i, w) in all_weights.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{w}")?;
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-weights-array\" count=\"{weight_count}\" stride=\"1\">\n")?;
    xml.push_str("              <param name=\"WEIGHT\" type=\"float\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    xml.push_str("        <joints>\n");
    write!(xml, "          <input semantic=\"JOINT\" source=\"#{name}-skin-joints\"/>\n")?;
    write!(xml, "          <input semantic=\"INV_BIND_MATRIX\" source=\"#{name}-skin-bind-poses\"/>\n")?;
    xml.push_str("        </joints>\n");

    let total_verts: usize = vcounts.len();
    write!(xml, "        <vertex_weights count=\"{total_verts}\">\n")?;
    write!(xml, "          <input semantic=\"JOINT\" source=\"#{name}-skin-joints\" offset=\"0\"/>\n")?;
    write!(xml, "          <input semantic=\"WEIGHT\" source=\"#{name}-skin-weights\" offset=\"1\"/>\n")?;
    xml.push_str("          <vcount>");
    for (i, vc) in vcounts.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{vc}")?;
    }
    xml.push_str("</vcount>\n");
    xml.push_str("          <v>");
    for (i, pair) in v_pairs.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        xml.push_str(pair);
    }
    xml.push_str("</v>\n");
    xml.push_str("        </vertex_weights>\n");

    xml.push_str("      </skin>\n");
    xml.push_str("    </controller>\n");
    xml.push_str("  </library_controllers>\n");
    Ok(())
}

fn write_visual_scene(xml: &mut String, geometry: &MeshGeometry, options: &DaeWriterOptions) -> Result<()> {
    let name = &options.mesh_name;
    xml.push_str("  <library_visual_scenes>\n");
    xml.push_str("    <visual_scene id=\"Scene\" name=\"Scene\">\n");

    if let Some(ref skin) = geometry.skin_info {
        write!(xml, "      <node id=\"Armature\" name=\"Armature\" type=\"NODE\">\n")?;
        xml.push_str("        <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n");
        write_skeleton_joints(xml, skin)?;
        write!(xml, "        <node id=\"{name}\" name=\"{name}\" type=\"NODE\">\n")?;
        write!(xml, "          <instance_controller url=\"#{name}-skin\">\n")?;
        xml.push_str("            <skeleton>#Armature</skeleton>\n");
        xml.push_str("            <bind_material>\n");
        xml.push_str("              <technique_common>\n");
        for i in 0..geometry.faces.len() {
            write!(xml, "                <instance_material symbol=\"material_face{i}\" target=\"#material_face{i}\"/>\n")?;
        }
        xml.push_str("              </technique_common>\n");
        xml.push_str("            </bind_material>\n");
        xml.push_str("          </instance_controller>\n");
        xml.push_str("        </node>\n");
        xml.push_str("      </node>\n");
    } else {
        write!(xml, "      <node id=\"{name}\" name=\"{name}\" type=\"NODE\">\n")?;
        xml.push_str("        <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n");
        write!(xml, "        <instance_geometry url=\"#{name}-mesh\">\n")?;
        xml.push_str("          <bind_material>\n");
        xml.push_str("            <technique_common>\n");
        for i in 0..geometry.faces.len() {
            write!(xml, "              <instance_material symbol=\"material_face{i}\" target=\"#material_face{i}\"/>\n")?;
        }
        xml.push_str("            </technique_common>\n");
        xml.push_str("          </bind_material>\n");
        xml.push_str("        </instance_geometry>\n");
        xml.push_str("      </node>\n");
    }

    xml.push_str("    </visual_scene>\n");
    xml.push_str("  </library_visual_scenes>\n");
    Ok(())
}

fn write_skeleton_joints(xml: &mut String, skin: &MeshSkinInfo) -> Result<()> {
    let hierarchy: &[(&str, Option<&str>)] = &[
        ("mPelvis", None),
        ("mTorso", Some("mPelvis")),
        ("mChest", Some("mTorso")),
        ("mNeck", Some("mChest")),
        ("mHead", Some("mNeck")),
        ("mSkull", Some("mHead")),
        ("mEyeRight", Some("mHead")),
        ("mEyeLeft", Some("mHead")),
        ("mCollarLeft", Some("mChest")),
        ("mShoulderLeft", Some("mCollarLeft")),
        ("mElbowLeft", Some("mShoulderLeft")),
        ("mWristLeft", Some("mElbowLeft")),
        ("mCollarRight", Some("mChest")),
        ("mShoulderRight", Some("mCollarRight")),
        ("mElbowRight", Some("mShoulderRight")),
        ("mWristRight", Some("mElbowRight")),
        ("mHipLeft", Some("mPelvis")),
        ("mKneeLeft", Some("mHipLeft")),
        ("mAnkleLeft", Some("mKneeLeft")),
        ("mFootLeft", Some("mAnkleLeft")),
        ("mToeLeft", Some("mFootLeft")),
        ("mHipRight", Some("mPelvis")),
        ("mKneeRight", Some("mHipRight")),
        ("mAnkleRight", Some("mKneeRight")),
        ("mFootRight", Some("mAnkleRight")),
        ("mToeRight", Some("mFootRight")),
    ];

    let collision_volumes: &[(&str, &str)] = &[
        ("HEAD", "mHead"), ("NECK", "mNeck"), ("CHEST", "mChest"),
        ("LEFT_PEC", "mChest"), ("RIGHT_PEC", "mChest"),
        ("UPPER_BACK", "mChest"), ("BELLY", "mTorso"),
        ("LOWER_BACK", "mTorso"), ("PELVIS", "mPelvis"),
        ("BUTT", "mPelvis"),
        ("L_CLAVICLE", "mCollarLeft"), ("L_UPPER_ARM", "mShoulderLeft"),
        ("L_LOWER_ARM", "mElbowLeft"), ("L_HAND", "mWristLeft"),
        ("R_CLAVICLE", "mCollarRight"), ("R_UPPER_ARM", "mShoulderRight"),
        ("R_LOWER_ARM", "mElbowRight"), ("R_HAND", "mWristRight"),
        ("L_UPPER_LEG", "mHipLeft"), ("L_LOWER_LEG", "mKneeLeft"),
        ("L_FOOT", "mAnkleLeft"),
        ("R_UPPER_LEG", "mHipRight"), ("R_LOWER_LEG", "mKneeRight"),
        ("R_FOOT", "mAnkleRight"),
    ];

    use std::collections::{HashMap, HashSet};
    let joint_set: HashSet<&str> = skin.joint_names.iter().map(|s| s.as_str()).collect();
    let cv_parents: HashMap<&str, &str> = collision_volumes.iter().copied().collect();

    struct JNode {
        name: String,
        children: Vec<JNode>,
    }

    fn build_tree(name: &str, hierarchy: &[(&str, Option<&str>)], cv_parents: &HashMap<&str, &str>, joint_set: &HashSet<&str>) -> JNode {
        let mut children = Vec::new();
        for &(child, parent) in hierarchy {
            if parent == Some(name) && joint_set.contains(child) {
                children.push(build_tree(child, hierarchy, cv_parents, joint_set));
            }
        }
        for (&cv, &parent) in cv_parents {
            if parent == name && joint_set.contains(cv) {
                children.push(JNode { name: cv.to_string(), children: Vec::new() });
            }
        }
        JNode { name: name.to_string(), children }
    }

    fn write_node(xml: &mut String, node: &JNode, depth: usize) -> Result<()> {
        let indent = "        ".repeat(depth + 1);
        write!(xml, "{}<node id=\"{}\" sid=\"{}\" name=\"{}\" type=\"JOINT\">\n", indent, node.name, node.name, node.name)?;
        write!(xml, "{}  <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n", indent)?;
        for child in &node.children {
            write_node(xml, child, depth + 1)?;
        }
        write!(xml, "{}</node>\n", indent)?;
        Ok(())
    }

    if joint_set.contains("mPelvis") {
        let root = build_tree("mPelvis", hierarchy, &cv_parents, &joint_set);
        write_node(xml, &root, 0)?;
    } else {
        for jname in &skin.joint_names {
            let found_in_hierarchy = hierarchy.iter().any(|(n, _)| *n == jname.as_str());
            let found_in_cv = cv_parents.contains_key(jname.as_str());
            if !found_in_hierarchy && !found_in_cv {
                write!(xml, "        <node id=\"{}\" sid=\"{}\" name=\"{}\" type=\"JOINT\">\n", jname, jname, jname)?;
                xml.push_str("          <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n");
                write!(xml, "        </node>\n")?;
            }
        }
    }

    Ok(())
}

fn write_footer(xml: &mut String) -> Result<()> {
    xml.push_str("  <scene>\n");
    xml.push_str("    <instance_visual_scene url=\"#Scene\"/>\n");
    xml.push_str("  </scene>\n");
    xml.push_str("</COLLADA>\n");
    Ok(())
}

pub fn write_multi_mesh_dae(
    meshes: &[(String, MeshGeometry, Vec<Option<String>>)],
    up_axis: UpAxis,
) -> Result<String> {
    let mut xml = String::with_capacity(256 * 1024);

    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">\n");
    xml.push_str("  <asset>\n");
    xml.push_str("    <contributor>\n");
    xml.push_str("      <author>OpenSim Next Snapshot Statue Pipeline</author>\n");
    xml.push_str("    </contributor>\n");
    match up_axis {
        UpAxis::YUp => xml.push_str("    <up_axis>Y_UP</up_axis>\n"),
        UpAxis::ZUp => xml.push_str("    <up_axis>Z_UP</up_axis>\n"),
    }
    xml.push_str("    <unit name=\"meter\" meter=\"1.0\"/>\n");
    xml.push_str("  </asset>\n");

    xml.push_str("  <library_images>\n");
    for (name, _geo, textures) in meshes {
        for (i, tex) in textures.iter().enumerate() {
            if let Some(ref path) = tex {
                write!(xml, "    <image id=\"image_{name}_face{i}\" name=\"tex_{name}_face{i}\">\n")?;
                write!(xml, "      <init_from>{path}</init_from>\n")?;
                xml.push_str("    </image>\n");
            }
        }
    }
    xml.push_str("  </library_images>\n");

    xml.push_str("  <library_effects>\n");
    for (name, geo, textures) in meshes {
        for i in 0..geo.faces.len() {
            write!(xml, "    <effect id=\"effect_{name}_face{i}\">\n")?;
            xml.push_str("      <profile_COMMON>\n");
            if let Some(Some(_)) = textures.get(i) {
                write!(xml, "        <newparam sid=\"surface_{name}_face{i}\">\n")?;
                xml.push_str("          <surface type=\"2D\">\n");
                write!(xml, "            <init_from>image_{name}_face{i}</init_from>\n")?;
                xml.push_str("          </surface>\n");
                xml.push_str("        </newparam>\n");
                write!(xml, "        <newparam sid=\"sampler_{name}_face{i}\">\n")?;
                xml.push_str("          <sampler2D>\n");
                write!(xml, "            <source>surface_{name}_face{i}</source>\n")?;
                xml.push_str("          </sampler2D>\n");
                xml.push_str("        </newparam>\n");
            }
            xml.push_str("        <technique sid=\"common\">\n");
            xml.push_str("          <phong>\n");
            xml.push_str("            <diffuse>\n");
            if let Some(Some(_)) = textures.get(i) {
                write!(xml, "              <texture texture=\"sampler_{name}_face{i}\" texcoord=\"UVMap\"/>\n")?;
            } else {
                xml.push_str("              <color>0.8 0.8 0.8 1.0</color>\n");
            }
            xml.push_str("            </diffuse>\n");
            xml.push_str("          </phong>\n");
            xml.push_str("        </technique>\n");
            xml.push_str("      </profile_COMMON>\n");
            xml.push_str("    </effect>\n");
        }
    }
    xml.push_str("  </library_effects>\n");

    xml.push_str("  <library_materials>\n");
    for (name, geo, _) in meshes {
        for i in 0..geo.faces.len() {
            write!(xml, "    <material id=\"material_{name}_face{i}\" name=\"{name}_Face{i}\">\n")?;
            write!(xml, "      <instance_effect url=\"#effect_{name}_face{i}\"/>\n")?;
            xml.push_str("    </material>\n");
        }
    }
    xml.push_str("  </library_materials>\n");

    xml.push_str("  <library_geometries>\n");
    for (name, geo, _) in meshes {
        write!(xml, "    <geometry id=\"{name}-mesh\" name=\"{name}\">\n")?;
        xml.push_str("      <mesh>\n");
        for (fi, face) in geo.faces.iter().enumerate() {
            write_face_sources(&mut xml, face, name, fi)?;
        }
        xml.push_str("      </mesh>\n");
        xml.push_str("    </geometry>\n");
    }
    xml.push_str("  </library_geometries>\n");

    let has_skin = meshes.iter().any(|(_, g, _)| g.skin_info.is_some());
    if has_skin {
        xml.push_str("  <library_controllers>\n");
        for (name, geo, _) in meshes {
            if let Some(ref skin) = geo.skin_info {
                write_skin_controller(&mut xml, name, geo, skin)?;
            }
        }
        xml.push_str("  </library_controllers>\n");
    }

    xml.push_str("  <library_visual_scenes>\n");
    xml.push_str("    <visual_scene id=\"Scene\" name=\"Scene\">\n");
    for (name, geo, _) in meshes {
        if geo.skin_info.is_some() {
            write!(xml, "      <node id=\"{name}_Armature\" name=\"{name}_Armature\" type=\"NODE\">\n")?;
            xml.push_str("        <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n");
            write!(xml, "        <node id=\"{name}\" name=\"{name}\" type=\"NODE\">\n")?;
            write!(xml, "          <instance_controller url=\"#{name}-skin\">\n")?;
            write!(xml, "            <skeleton>#{name}_Armature</skeleton>\n")?;
            xml.push_str("            <bind_material>\n");
            xml.push_str("              <technique_common>\n");
            for i in 0..geo.faces.len() {
                write!(xml, "                <instance_material symbol=\"material_{name}_face{i}\" target=\"#material_{name}_face{i}\"/>\n")?;
            }
            xml.push_str("              </technique_common>\n");
            xml.push_str("            </bind_material>\n");
            xml.push_str("          </instance_controller>\n");
            xml.push_str("        </node>\n");
            xml.push_str("      </node>\n");
        } else {
            write!(xml, "      <node id=\"{name}\" name=\"{name}\" type=\"NODE\">\n")?;
            xml.push_str("        <matrix sid=\"transform\">1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</matrix>\n");
            write!(xml, "        <instance_geometry url=\"#{name}-mesh\">\n")?;
            xml.push_str("          <bind_material>\n");
            xml.push_str("            <technique_common>\n");
            for i in 0..geo.faces.len() {
                write!(xml, "              <instance_material symbol=\"material_{name}_face{i}\" target=\"#material_{name}_face{i}\"/>\n")?;
            }
            xml.push_str("            </technique_common>\n");
            xml.push_str("          </bind_material>\n");
            xml.push_str("        </instance_geometry>\n");
            xml.push_str("      </node>\n");
        }
    }
    xml.push_str("    </visual_scene>\n");
    xml.push_str("  </library_visual_scenes>\n");
    xml.push_str("  <scene>\n");
    xml.push_str("    <instance_visual_scene url=\"#Scene\"/>\n");
    xml.push_str("  </scene>\n");
    xml.push_str("</COLLADA>\n");

    Ok(xml)
}

fn write_face_sources(xml: &mut String, face: &super::encoder::MeshFace, name: &str, fi: usize) -> Result<()> {
    let pos_id = format!("{name}-face{fi}-positions");
    let norm_id = format!("{name}-face{fi}-normals");
    let uv_id = format!("{name}-face{fi}-map0");
    let vert_id = format!("{name}-face{fi}-vertices");

    write!(xml, "        <source id=\"{pos_id}\">\n")?;
    let pos_count = face.positions.len() * 3;
    write!(xml, "          <float_array id=\"{pos_id}-array\" count=\"{pos_count}\">")?;
    for (vi, p) in face.positions.iter().enumerate() {
        if vi > 0 { xml.push(' '); }
        write!(xml, "{} {} {}", p[0], p[1], p[2])?;
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{pos_id}-array\" count=\"{}\" stride=\"3\">\n", face.positions.len())?;
    xml.push_str("              <param name=\"X\" type=\"float\"/>\n");
    xml.push_str("              <param name=\"Y\" type=\"float\"/>\n");
    xml.push_str("              <param name=\"Z\" type=\"float\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    if !face.normals.is_empty() {
        write!(xml, "        <source id=\"{norm_id}\">\n")?;
        let norm_count = face.normals.len() * 3;
        write!(xml, "          <float_array id=\"{norm_id}-array\" count=\"{norm_count}\">")?;
        for (vi, n) in face.normals.iter().enumerate() {
            if vi > 0 { xml.push(' '); }
            write!(xml, "{} {} {}", n[0], n[1], n[2])?;
        }
        xml.push_str("</float_array>\n");
        xml.push_str("          <technique_common>\n");
        write!(xml, "            <accessor source=\"#{norm_id}-array\" count=\"{}\" stride=\"3\">\n", face.normals.len())?;
        xml.push_str("              <param name=\"X\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"Y\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"Z\" type=\"float\"/>\n");
        xml.push_str("            </accessor>\n");
        xml.push_str("          </technique_common>\n");
        xml.push_str("        </source>\n");
    }

    if !face.tex_coords.is_empty() {
        write!(xml, "        <source id=\"{uv_id}\">\n")?;
        let uv_count = face.tex_coords.len() * 2;
        write!(xml, "          <float_array id=\"{uv_id}-array\" count=\"{uv_count}\">")?;
        for (vi, uv) in face.tex_coords.iter().enumerate() {
            if vi > 0 { xml.push(' '); }
            write!(xml, "{} {}", uv[0], uv[1])?;
        }
        xml.push_str("</float_array>\n");
        xml.push_str("          <technique_common>\n");
        write!(xml, "            <accessor source=\"#{uv_id}-array\" count=\"{}\" stride=\"2\">\n", face.tex_coords.len())?;
        xml.push_str("              <param name=\"S\" type=\"float\"/>\n");
        xml.push_str("              <param name=\"T\" type=\"float\"/>\n");
        xml.push_str("            </accessor>\n");
        xml.push_str("          </technique_common>\n");
        xml.push_str("        </source>\n");
    }

    write!(xml, "        <vertices id=\"{vert_id}\">\n")?;
    write!(xml, "          <input semantic=\"POSITION\" source=\"#{pos_id}\"/>\n")?;
    xml.push_str("        </vertices>\n");

    let tri_count = face.indices.len() / 3;
    write!(xml, "        <triangles material=\"material_{name}_face{fi}\" count=\"{tri_count}\">\n")?;

    let mut offset = 0;
    write!(xml, "          <input semantic=\"VERTEX\" source=\"#{vert_id}\" offset=\"{offset}\"/>\n")?;
    offset += 1;
    if !face.normals.is_empty() {
        write!(xml, "          <input semantic=\"NORMAL\" source=\"#{norm_id}\" offset=\"{offset}\"/>\n")?;
        offset += 1;
    }
    if !face.tex_coords.is_empty() {
        write!(xml, "          <input semantic=\"TEXCOORD\" source=\"#{uv_id}\" offset=\"{offset}\" set=\"0\"/>\n")?;
        offset += 1;
    }

    let stride = offset;
    xml.push_str("          <p>");
    for (ti, idx) in face.indices.iter().enumerate() {
        if ti > 0 { xml.push(' '); }
        let v = *idx as usize;
        for s in 0..stride {
            if s > 0 { xml.push(' '); }
            write!(xml, "{v}")?;
        }
    }
    xml.push_str("</p>\n");
    xml.push_str("        </triangles>\n");

    Ok(())
}

fn write_skin_controller(
    xml: &mut String,
    name: &str,
    geometry: &MeshGeometry,
    skin: &MeshSkinInfo,
) -> Result<()> {
    write!(xml, "    <controller id=\"{name}-skin\" name=\"{name}-skin\">\n")?;
    write!(xml, "      <skin source=\"#{name}-mesh\">\n")?;

    xml.push_str("        <bind_shape_matrix>");
    for (i, v) in skin.bind_shape_matrix.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{v}")?;
    }
    xml.push_str("</bind_shape_matrix>\n");

    let joint_count = skin.joint_names.len();
    write!(xml, "        <source id=\"{name}-skin-joints\">\n")?;
    write!(xml, "          <Name_array id=\"{name}-skin-joints-array\" count=\"{joint_count}\">")?;
    for (i, jn) in skin.joint_names.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        xml.push_str(jn);
    }
    xml.push_str("</Name_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-joints-array\" count=\"{joint_count}\" stride=\"1\">\n")?;
    xml.push_str("              <param name=\"JOINT\" type=\"name\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    let ibm_count = skin.inverse_bind_matrices.len() * 16;
    write!(xml, "        <source id=\"{name}-skin-bind-poses\">\n")?;
    write!(xml, "          <float_array id=\"{name}-skin-bind-poses-array\" count=\"{ibm_count}\">")?;
    for (mi, mat) in skin.inverse_bind_matrices.iter().enumerate() {
        if mi > 0 { xml.push(' '); }
        for (vi, v) in mat.iter().enumerate() {
            if vi > 0 { xml.push(' '); }
            write!(xml, "{v}")?;
        }
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-bind-poses-array\" count=\"{}\" stride=\"16\">\n", skin.inverse_bind_matrices.len())?;
    xml.push_str("              <param name=\"TRANSFORM\" type=\"float4x4\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    let mut all_weights: Vec<f32> = Vec::new();
    let mut vcounts: Vec<usize> = Vec::new();
    let mut v_pairs: Vec<String> = Vec::new();

    for face in &geometry.faces {
        if let Some(ref jw) = face.joint_weights {
            for vw in jw {
                vcounts.push(vw.influences.len());
                for inf in &vw.influences {
                    let weight_idx = all_weights.len();
                    all_weights.push(inf.weight);
                    v_pairs.push(format!("{} {weight_idx}", inf.joint_index));
                }
            }
        }
    }

    let weight_count = all_weights.len();
    write!(xml, "        <source id=\"{name}-skin-weights\">\n")?;
    write!(xml, "          <float_array id=\"{name}-skin-weights-array\" count=\"{weight_count}\">")?;
    for (i, w) in all_weights.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{w}")?;
    }
    xml.push_str("</float_array>\n");
    xml.push_str("          <technique_common>\n");
    write!(xml, "            <accessor source=\"#{name}-skin-weights-array\" count=\"{weight_count}\" stride=\"1\">\n")?;
    xml.push_str("              <param name=\"WEIGHT\" type=\"float\"/>\n");
    xml.push_str("            </accessor>\n");
    xml.push_str("          </technique_common>\n");
    xml.push_str("        </source>\n");

    xml.push_str("        <joints>\n");
    write!(xml, "          <input semantic=\"JOINT\" source=\"#{name}-skin-joints\"/>\n")?;
    write!(xml, "          <input semantic=\"INV_BIND_MATRIX\" source=\"#{name}-skin-bind-poses\"/>\n")?;
    xml.push_str("        </joints>\n");

    let total_verts = vcounts.len();
    write!(xml, "        <vertex_weights count=\"{total_verts}\">\n")?;
    write!(xml, "          <input semantic=\"JOINT\" source=\"#{name}-skin-joints\" offset=\"0\"/>\n")?;
    write!(xml, "          <input semantic=\"WEIGHT\" source=\"#{name}-skin-weights\" offset=\"1\"/>\n")?;
    xml.push_str("          <vcount>");
    for (i, vc) in vcounts.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        write!(xml, "{vc}")?;
    }
    xml.push_str("</vcount>\n");
    xml.push_str("          <v>");
    for (i, pair) in v_pairs.iter().enumerate() {
        if i > 0 { xml.push(' '); }
        xml.push_str(pair);
    }
    xml.push_str("</v>\n");
    xml.push_str("        </vertex_weights>\n");

    xml.push_str("      </skin>\n");
    xml.push_str("    </controller>\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::encoder::{MeshFace, MeshGeometry, MeshSkinInfo, VertexWeights, JointInfluence};

    fn make_triangle_face() -> MeshFace {
        MeshFace {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        }
    }

    #[test]
    fn test_write_simple_dae() {
        let geo = MeshGeometry {
            faces: vec![make_triangle_face()],
            skin_info: None,
        };
        let opts = DaeWriterOptions::default();
        let dae = write_dae(&geo, &opts).unwrap();
        assert!(dae.contains("<COLLADA"));
        assert!(dae.contains("</COLLADA>"));
        assert!(dae.contains("<library_geometries>"));
        assert!(dae.contains("mesh-positions"));
        assert!(dae.contains("<triangles"));
        assert!(!dae.contains("<library_controllers>"));
    }

    #[test]
    fn test_write_dae_with_textures() {
        let geo = MeshGeometry {
            faces: vec![make_triangle_face()],
            skin_info: None,
        };
        let opts = DaeWriterOptions {
            mesh_name: "shirt".to_string(),
            texture_files: vec![Some("textures/shirt_face0.png".to_string())],
            up_axis: UpAxis::ZUp,
        };
        let dae = write_dae(&geo, &opts).unwrap();
        assert!(dae.contains("<library_images>"));
        assert!(dae.contains("shirt_face0.png"));
        assert!(dae.contains("sampler_face0"));
    }

    #[test]
    fn test_write_dae_with_skin() {
        let face = MeshFace {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: Some(vec![
                VertexWeights { influences: vec![JointInfluence { joint_index: 0, weight: 1.0 }] },
                VertexWeights { influences: vec![JointInfluence { joint_index: 0, weight: 0.5 }, JointInfluence { joint_index: 1, weight: 0.5 }] },
                VertexWeights { influences: vec![JointInfluence { joint_index: 1, weight: 1.0 }] },
            ]),
            original_position_indices: None,
        };
        let mut identity = [0.0f32; 16];
        identity[0] = 1.0; identity[5] = 1.0; identity[10] = 1.0; identity[15] = 1.0;
        let geo = MeshGeometry {
            faces: vec![face],
            skin_info: Some(MeshSkinInfo {
                joint_names: vec!["mChest".to_string(), "mNeck".to_string()],
                inverse_bind_matrices: vec![identity, identity],
                bind_shape_matrix: identity,
                pelvis_offset: 0.0,
            }),
        };
        let opts = DaeWriterOptions::default();
        let dae = write_dae(&geo, &opts).unwrap();
        assert!(dae.contains("<library_controllers>"));
        assert!(dae.contains("mesh-skin"));
        assert!(dae.contains("mChest mNeck"));
        assert!(dae.contains("<vcount>1 2 1</vcount>"));
        assert!(dae.contains("<instance_controller"));
    }

    #[test]
    fn test_write_multi_mesh_dae() {
        let face1 = make_triangle_face();
        let face2 = MeshFace {
            positions: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            joint_weights: None,
            original_position_indices: None,
        };
        let geo1 = MeshGeometry { faces: vec![face1], skin_info: None };
        let geo2 = MeshGeometry { faces: vec![face2], skin_info: None };

        let meshes = vec![
            ("body".to_string(), geo1, vec![Some("body_tex.png".to_string())]),
            ("shirt".to_string(), geo2, vec![Some("shirt_tex.png".to_string())]),
        ];

        let dae = write_multi_mesh_dae(&meshes, UpAxis::ZUp).unwrap();
        assert!(dae.contains("body-mesh"));
        assert!(dae.contains("shirt-mesh"));
        assert!(dae.contains("body_tex.png"));
        assert!(dae.contains("shirt_tex.png"));
        assert!(dae.contains("material_body_face0"));
        assert!(dae.contains("material_shirt_face0"));
    }

    #[test]
    fn test_round_trip_dae_parse() {
        let geo = MeshGeometry {
            faces: vec![make_triangle_face()],
            skin_info: None,
        };
        let opts = DaeWriterOptions {
            mesh_name: "test".to_string(),
            texture_files: vec![],
            up_axis: UpAxis::ZUp,
        };
        let dae = write_dae(&geo, &opts).unwrap();

        let parsed = crate::mesh::collada_geometry::parse_collada_geometry(&dae).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].positions.len(), 3);
        assert_eq!(parsed[0].indices.len(), 3);

        let epsilon = 0.001;
        for i in 0..3 {
            for j in 0..3 {
                assert!((parsed[0].positions[i][j] - geo.faces[0].positions[i][j]).abs() < epsilon,
                    "Position mismatch at [{i}][{j}]: {} vs {}", parsed[0].positions[i][j], geo.faces[0].positions[i][j]);
            }
        }
    }
}
