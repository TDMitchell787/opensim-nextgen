use crate::error::{IoError, Result};
use glam::Vec3;
use glc_core::camera::Camera;
use glc_core::material::Material;
use glc_core::types::{Color4f, EntityId};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Current album file format version.
pub const ALBUM_VERSION: &str = "2.1";
pub const APP_NAME: &str = "GLC-Player";

/// A single model entry in an album, matching the original FileEntry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumEntry {
    pub id: EntityId,
    /// Absolute file path.
    pub absolute_path: String,
    /// Relative file path (relative to album location).
    pub relative_path: String,
    /// Camera state (None if not set).
    pub camera: Option<AlbumCamera>,
    /// Modified material overrides.
    pub materials: Vec<AlbumMaterial>,
    /// Names of invisible instances.
    pub invisible_instances: Vec<String>,
    /// Shader group assignments: shader_name → list of instance names.
    pub shader_groups: Vec<ShaderGroup>,
}

/// Camera state stored in an album entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumCamera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub default_up: Vec3,
    pub fov_degrees: f64,
}

impl From<&Camera> for AlbumCamera {
    fn from(cam: &Camera) -> Self {
        Self {
            eye: cam.eye,
            target: cam.target,
            up: cam.up,
            default_up: cam.default_up,
            fov_degrees: cam.fov_degrees,
        }
    }
}

impl From<&AlbumCamera> for Camera {
    fn from(ac: &AlbumCamera) -> Self {
        Camera {
            eye: ac.eye,
            target: ac.target,
            up: ac.up,
            default_up: ac.default_up,
            fov_degrees: ac.fov_degrees,
        }
    }
}

/// Material override stored in an album (colors in 0-255 range matching original).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumMaterial {
    pub name: String,
    pub ambient: [u8; 4],
    pub diffuse: [u8; 4],
    pub specular: [u8; 4],
    pub emissive: [u8; 4],
    pub shininess: f32,
    pub texture_path: String,
}

impl AlbumMaterial {
    pub fn from_material(mat: &Material) -> Self {
        Self {
            name: mat.name.clone(),
            ambient: color_to_u8(mat.ambient),
            diffuse: color_to_u8(mat.diffuse),
            specular: color_to_u8(mat.specular),
            emissive: color_to_u8(mat.emissive),
            shininess: mat.shininess,
            texture_path: mat.texture_path.clone().unwrap_or_default(),
        }
    }

    pub fn to_material(&self) -> Material {
        Material {
            id: EntityId::new(),
            name: self.name.clone(),
            ambient: color_from_u8(self.ambient),
            diffuse: color_from_u8(self.diffuse),
            specular: color_from_u8(self.specular),
            emissive: color_from_u8(self.emissive),
            shininess: self.shininess,
            opacity: self.diffuse[3] as f32 / 255.0,
            texture_path: if self.texture_path.is_empty() {
                None
            } else {
                Some(self.texture_path.clone())
            },
            texture_data: None,
            is_modified: true,
        }
    }
}

/// A shader group assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderGroup {
    pub shader_name: String,
    pub instance_names: Vec<String>,
}

/// Full album file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub version: String,
    pub entries: Vec<AlbumEntry>,
    /// Path to the album file itself (for relative path resolution).
    pub file_path: Option<PathBuf>,
}

impl Album {
    pub fn new() -> Self {
        Self {
            version: ALBUM_VERSION.to_string(),
            entries: Vec::new(),
            file_path: None,
        }
    }
}

impl Default for Album {
    fn default() -> Self {
        Self::new()
    }
}

// --- Color conversion helpers ---

fn color_to_u8(c: Color4f) -> [u8; 4] {
    [
        (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
        (c.a * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn color_from_u8(c: [u8; 4]) -> Color4f {
    Color4f::new(
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    )
}

// --- XML Writing ---

/// Write an album to an XML writer.
pub fn write_album<W: Write>(album: &Album, writer: W) -> Result<()> {
    let mut xml = Writer::new_with_indent(writer, b' ', 2);

    xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    xml.write_event(Event::Start(BytesStart::new("Album")))?;

    // Header
    xml.write_event(Event::Start(BytesStart::new("Header")))?;
    let mut app = BytesStart::new("Application");
    app.push_attribute(("Name", APP_NAME));
    app.push_attribute(("Version", ALBUM_VERSION));
    xml.write_event(Event::Empty(app))?;
    xml.write_event(Event::End(BytesEnd::new("Header")))?;

    // Root with models
    xml.write_event(Event::Start(BytesStart::new("Root")))?;
    for entry in &album.entries {
        write_model_entry(&mut xml, entry)?;
    }
    xml.write_event(Event::End(BytesEnd::new("Root")))?;

    xml.write_event(Event::End(BytesEnd::new("Album")))?;
    Ok(())
}

fn write_model_entry<W: Write>(xml: &mut Writer<W>, entry: &AlbumEntry) -> Result<()> {
    let mut model = BytesStart::new("Model");
    model.push_attribute(("AFileName", entry.absolute_path.as_str()));
    model.push_attribute(("RFileName", entry.relative_path.as_str()));
    xml.write_event(Event::Start(model))?;

    // Camera
    if let Some(ref cam) = entry.camera {
        write_camera(xml, cam)?;
    }

    // Materials
    write_materials(xml, &entry.materials)?;

    // Invisible instances
    write_invisible_instances(xml, &entry.invisible_instances)?;

    // Shaders
    write_shaders(xml, &entry.shader_groups)?;

    xml.write_event(Event::End(BytesEnd::new("Model")))?;
    Ok(())
}

fn write_camera<W: Write>(xml: &mut Writer<W>, cam: &AlbumCamera) -> Result<()> {
    let mut camera = BytesStart::new("Camera");
    camera.push_attribute(("Angle", cam.fov_degrees.to_string().as_str()));
    xml.write_event(Event::Start(camera))?;

    write_vec3_element(xml, "Eye", cam.eye)?;
    write_vec3_element(xml, "Target", cam.target)?;
    write_vec3_element(xml, "Up", cam.up)?;
    write_vec3_element(xml, "DefaultUp", cam.default_up)?;

    xml.write_event(Event::End(BytesEnd::new("Camera")))?;
    Ok(())
}

fn write_vec3_element<W: Write>(xml: &mut Writer<W>, name: &str, v: Vec3) -> Result<()> {
    let mut elem = BytesStart::new(name);
    elem.push_attribute(("x", v.x.to_string().as_str()));
    elem.push_attribute(("y", v.y.to_string().as_str()));
    elem.push_attribute(("z", v.z.to_string().as_str()));
    xml.write_event(Event::Empty(elem))?;
    Ok(())
}

fn write_materials<W: Write>(xml: &mut Writer<W>, materials: &[AlbumMaterial]) -> Result<()> {
    let mut elem = BytesStart::new("Materials");
    elem.push_attribute(("size", materials.len().to_string().as_str()));
    xml.write_event(Event::Start(elem))?;

    for mat in materials {
        let mut mat_elem = BytesStart::new("Material");
        mat_elem.push_attribute(("name", mat.name.as_str()));
        xml.write_event(Event::Start(mat_elem))?;

        // Note: original uses "Ambiant" (misspelled)
        write_color_element(xml, "Ambiant", mat.ambient)?;
        write_color_element(xml, "Diffuse", mat.diffuse)?;
        write_color_element(xml, "Specular", mat.specular)?;
        write_color_element(xml, "LightEmission", mat.emissive)?;

        let mut shin = BytesStart::new("Shininess");
        shin.push_attribute(("value", mat.shininess.to_string().as_str()));
        xml.write_event(Event::Empty(shin))?;

        let mut tex = BytesStart::new("Texture");
        tex.push_attribute(("fileName", mat.texture_path.as_str()));
        xml.write_event(Event::Empty(tex))?;

        xml.write_event(Event::End(BytesEnd::new("Material")))?;
    }

    xml.write_event(Event::End(BytesEnd::new("Materials")))?;
    Ok(())
}

fn write_color_element<W: Write>(xml: &mut Writer<W>, name: &str, rgba: [u8; 4]) -> Result<()> {
    let mut elem = BytesStart::new(name);
    elem.push_attribute(("r", rgba[0].to_string().as_str()));
    elem.push_attribute(("g", rgba[1].to_string().as_str()));
    elem.push_attribute(("b", rgba[2].to_string().as_str()));
    elem.push_attribute(("a", rgba[3].to_string().as_str()));
    xml.write_event(Event::Empty(elem))?;
    Ok(())
}

fn write_invisible_instances<W: Write>(
    xml: &mut Writer<W>,
    instances: &[String],
) -> Result<()> {
    let mut elem = BytesStart::new("InvisibleInstances");
    elem.push_attribute(("size", instances.len().to_string().as_str()));
    xml.write_event(Event::Start(elem))?;

    for name in instances {
        let mut inst = BytesStart::new("Instance");
        inst.push_attribute(("name", name.as_str()));
        xml.write_event(Event::Empty(inst))?;
    }

    xml.write_event(Event::End(BytesEnd::new("InvisibleInstances")))?;
    Ok(())
}

fn write_shaders<W: Write>(xml: &mut Writer<W>, groups: &[ShaderGroup]) -> Result<()> {
    let mut elem = BytesStart::new("Shaders");
    elem.push_attribute(("size", groups.len().to_string().as_str()));
    xml.write_event(Event::Start(elem))?;

    for group in groups {
        let mut shader = BytesStart::new("Shader");
        shader.push_attribute(("name", group.shader_name.as_str()));
        shader.push_attribute(("size", group.instance_names.len().to_string().as_str()));
        xml.write_event(Event::Start(shader))?;

        for inst_name in &group.instance_names {
            let mut inst = BytesStart::new("Instance");
            inst.push_attribute(("name", inst_name.as_str()));
            xml.write_event(Event::Empty(inst))?;
        }

        xml.write_event(Event::End(BytesEnd::new("Shader")))?;
    }

    xml.write_event(Event::End(BytesEnd::new("Shaders")))?;
    Ok(())
}

/// Write an album to a file.
pub fn save_album(album: &Album, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    write_album(album, writer)
}

// --- XML Reading ---

/// Read an album from an XML reader.
pub fn read_album<R: BufRead>(reader: R) -> Result<Album> {
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut album = Album::new();

    // Find <Album>
    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Album" => break,
            Ok(Event::Eof) => return Err(IoError::AlbumError("no <Album> element found".into())),
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    // Read Header
    let mut version = ALBUM_VERSION.to_string();
    buf.clear();
    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"Application" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"Version" {
                        version = String::from_utf8_lossy(&attr.value).into_owned();
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Root" => break,
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Album" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }
    album.version = version.clone();

    // Read Models inside Root
    buf.clear();
    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Model" => {
                let entry = read_model_entry(&mut xml, e, &version)?;
                album.entries.push(entry);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Root" => break,
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Album" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(album)
}

fn read_model_entry<R: BufRead>(
    xml: &mut Reader<R>,
    start: &BytesStart,
    version: &str,
) -> Result<AlbumEntry> {
    let mut absolute_path = String::new();
    let mut relative_path = String::new();

    for attr in start.attributes().flatten() {
        match attr.key.as_ref() {
            b"AFileName" => absolute_path = String::from_utf8_lossy(&attr.value).into_owned(),
            b"RFileName" => relative_path = String::from_utf8_lossy(&attr.value).into_owned(),
            _ => {}
        }
    }

    let mut camera = None;
    let mut materials = Vec::new();
    let mut invisible_instances = Vec::new();
    let mut shader_groups = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"Camera" => camera = Some(read_camera(xml, e, version)?),
                b"Materials" => materials = read_materials(xml)?,
                b"InvisibleInstances" => invisible_instances = read_instance_list(xml)?,
                b"Shaders" => shader_groups = read_shaders(xml)?,
                _ => {}
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Model" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(AlbumEntry {
        id: EntityId::new(),
        absolute_path,
        relative_path,
        camera,
        materials,
        invisible_instances,
        shader_groups,
    })
}

fn read_camera<R: BufRead>(
    xml: &mut Reader<R>,
    start: &BytesStart,
    version: &str,
) -> Result<AlbumCamera> {
    let mut fov_degrees = 35.0;
    for attr in start.attributes().flatten() {
        if attr.key.as_ref() == b"Angle" {
            fov_degrees = String::from_utf8_lossy(&attr.value)
                .parse()
                .unwrap_or(35.0);
        }
    }

    let mut eye = Vec3::ZERO;
    let mut target = Vec3::ZERO;
    let mut up = Vec3::Z;
    let mut default_up = Vec3::Z;
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let v = read_vec3_attrs(e);
                match e.name().as_ref() {
                    b"Eye" => eye = v,
                    b"Target" => target = v,
                    b"Up" => up = v,
                    b"DefaultUp" => default_up = v,
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Camera" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    // v2.0 backward compatibility: no DefaultUp element → use Z axis
    if version == "2.0" || version == "1.5" {
        // default_up stays at Z if not read
    }

    Ok(AlbumCamera {
        eye,
        target,
        up,
        default_up,
        fov_degrees,
    })
}

fn read_vec3_attrs(elem: &BytesStart) -> Vec3 {
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut z = 0.0f32;
    for attr in elem.attributes().flatten() {
        let val: f32 = String::from_utf8_lossy(&attr.value)
            .parse()
            .unwrap_or(0.0);
        match attr.key.as_ref() {
            b"x" => x = val,
            b"y" => y = val,
            b"z" => z = val,
            _ => {}
        }
    }
    Vec3::new(x, y, z)
}

fn read_materials<R: BufRead>(xml: &mut Reader<R>) -> Result<Vec<AlbumMaterial>> {
    let mut materials = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Material" => {
                let mat = read_single_material(xml, e)?;
                materials.push(mat);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Materials" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(materials)
}

fn read_single_material<R: BufRead>(
    xml: &mut Reader<R>,
    start: &BytesStart,
) -> Result<AlbumMaterial> {
    let mut name = String::new();
    for attr in start.attributes().flatten() {
        if attr.key.as_ref() == b"name" {
            name = String::from_utf8_lossy(&attr.value).into_owned();
        }
    }

    let mut ambient = [51, 51, 51, 255]; // 0.2*255
    let mut diffuse = [204, 204, 204, 255]; // 0.8*255
    let mut specular = [255, 255, 255, 255];
    let mut emissive = [0, 0, 0, 255];
    let mut shininess = 50.0f32;
    let mut texture_path = String::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"Ambiant" => ambient = read_color_attrs(e),
                    b"Diffuse" => diffuse = read_color_attrs(e),
                    b"Specular" => specular = read_color_attrs(e),
                    b"LightEmission" => emissive = read_color_attrs(e),
                    b"Shininess" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"value" {
                                shininess = String::from_utf8_lossy(&attr.value)
                                    .parse()
                                    .unwrap_or(50.0);
                            }
                        }
                    }
                    b"Texture" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"fileName" {
                                texture_path =
                                    String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Material" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(AlbumMaterial {
        name,
        ambient,
        diffuse,
        specular,
        emissive,
        shininess,
        texture_path,
    })
}

fn read_color_attrs(elem: &BytesStart) -> [u8; 4] {
    let mut r = 0u8;
    let mut g = 0u8;
    let mut b = 0u8;
    let mut a = 255u8;
    for attr in elem.attributes().flatten() {
        let val: u8 = String::from_utf8_lossy(&attr.value)
            .parse()
            .unwrap_or(0);
        match attr.key.as_ref() {
            b"r" => r = val,
            b"g" => g = val,
            b"b" => b = val,
            b"a" => a = val,
            _ => {}
        }
    }
    [r, g, b, a]
}

fn read_instance_list<R: BufRead>(xml: &mut Reader<R>) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"Instance" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"name" {
                        names.push(String::from_utf8_lossy(&attr.value).into_owned());
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Instance" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"name" {
                        names.push(String::from_utf8_lossy(&attr.value).into_owned());
                    }
                }
            }
            Ok(Event::End(ref e))
                if e.name().as_ref() == b"InvisibleInstances" =>
            {
                break
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(names)
}

fn read_shaders<R: BufRead>(xml: &mut Reader<R>) -> Result<Vec<ShaderGroup>> {
    let mut groups = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Shader" => {
                let group = read_shader_group(xml, e)?;
                groups.push(group);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Shaders" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(groups)
}

fn read_shader_group<R: BufRead>(
    xml: &mut Reader<R>,
    start: &BytesStart,
) -> Result<ShaderGroup> {
    let mut shader_name = String::new();
    for attr in start.attributes().flatten() {
        if attr.key.as_ref() == b"name" {
            shader_name = String::from_utf8_lossy(&attr.value).into_owned();
        }
    }

    let mut instance_names = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.name().as_ref() == b"Instance" =>
            {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"name" {
                        instance_names
                            .push(String::from_utf8_lossy(&attr.value).into_owned());
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Shader" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(IoError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(ShaderGroup {
        shader_name,
        instance_names,
    })
}

/// Load an album from a file.
pub fn load_album(path: &Path) -> Result<Album> {
    let file = std::fs::File::open(path)
        .map_err(|e| IoError::FileNotFound(format!("{}: {e}", path.display())))?;
    let reader = std::io::BufReader::new(file);
    let mut album = read_album(reader)?;
    album.file_path = Some(path.to_path_buf());

    // Resolve relative paths against album directory
    if let Some(album_dir) = path.parent() {
        for entry in &mut album.entries {
            if !Path::new(&entry.absolute_path).exists() && !entry.relative_path.is_empty() {
                let resolved = album_dir.join(&entry.relative_path);
                if resolved.exists() {
                    entry.absolute_path = resolved.to_string_lossy().into_owned();
                }
            }
        }
    }

    Ok(album)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_album() -> Album {
        Album {
            version: ALBUM_VERSION.to_string(),
            entries: vec![
                AlbumEntry {
                    id: EntityId::new(),
                    absolute_path: "/models/cube.obj".into(),
                    relative_path: "./cube.obj".into(),
                    camera: Some(AlbumCamera {
                        eye: Vec3::new(10.0, 20.0, 30.0),
                        target: Vec3::ZERO,
                        up: Vec3::Z,
                        default_up: Vec3::Z,
                        fov_degrees: 35.0,
                    }),
                    materials: vec![AlbumMaterial {
                        name: "Red".into(),
                        ambient: [51, 0, 0, 255],
                        diffuse: [255, 0, 0, 200],
                        specular: [255, 255, 255, 255],
                        emissive: [0, 0, 0, 255],
                        shininess: 80.0,
                        texture_path: String::new(),
                    }],
                    invisible_instances: vec!["hidden_part".into()],
                    shader_groups: vec![ShaderGroup {
                        shader_name: "ToonShader".into(),
                        instance_names: vec!["part_a".into(), "part_b".into()],
                    }],
                },
                AlbumEntry {
                    id: EntityId::new(),
                    absolute_path: "/models/sphere.stl".into(),
                    relative_path: "./sphere.stl".into(),
                    camera: None,
                    materials: Vec::new(),
                    invisible_instances: Vec::new(),
                    shader_groups: Vec::new(),
                },
            ],
            file_path: None,
        }
    }

    #[test]
    fn test_album_round_trip() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let xml_str = String::from_utf8(buf.clone()).unwrap();
        assert!(xml_str.contains("GLC-Player"));
        assert!(xml_str.contains("cube.obj"));

        // Read back
        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.entries[0].absolute_path, "/models/cube.obj");
        assert_eq!(loaded.entries[1].absolute_path, "/models/sphere.stl");
    }

    #[test]
    fn test_camera_round_trip() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        let cam = loaded.entries[0].camera.as_ref().unwrap();
        assert!((cam.eye.x - 10.0).abs() < 0.01);
        assert!((cam.eye.y - 20.0).abs() < 0.01);
        assert!((cam.eye.z - 30.0).abs() < 0.01);
        assert!((cam.fov_degrees - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_material_round_trip() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.entries[0].materials.len(), 1);
        let mat = &loaded.entries[0].materials[0];
        assert_eq!(mat.name, "Red");
        assert_eq!(mat.diffuse, [255, 0, 0, 200]);
        assert!((mat.shininess - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_invisible_instances_round_trip() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.entries[0].invisible_instances, vec!["hidden_part"]);
    }

    #[test]
    fn test_shader_groups_round_trip() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        assert_eq!(loaded.entries[0].shader_groups.len(), 1);
        let group = &loaded.entries[0].shader_groups[0];
        assert_eq!(group.shader_name, "ToonShader");
        assert_eq!(group.instance_names, vec!["part_a", "part_b"]);
    }

    #[test]
    fn test_no_camera_entry() {
        let album = make_test_album();
        let mut buf = Vec::new();
        write_album(&album, &mut buf).unwrap();

        let loaded = read_album(std::io::Cursor::new(&buf)).unwrap();
        assert!(loaded.entries[1].camera.is_none());
    }

    #[test]
    fn test_color_conversion_round_trip() {
        let c = Color4f::new(0.5, 0.25, 0.75, 1.0);
        let u = color_to_u8(c);
        let c2 = color_from_u8(u);
        assert!((c.r - c2.r).abs() < 0.01);
        assert!((c.g - c2.g).abs() < 0.01);
        assert!((c.b - c2.b).abs() < 0.01);
        assert!((c.a - c2.a).abs() < 0.01);
    }
}
