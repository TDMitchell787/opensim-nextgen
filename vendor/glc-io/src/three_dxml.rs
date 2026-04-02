use crate::error::{IoError, Result};
use glc_core::material::Material;
use glc_core::mesh::{MaterialRange, Mesh};
use glc_core::scene::{SceneNode, World};
use glc_core::transform::Transform;
use glc_core::types::{Color4f, EntityId};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

/// ZIP magic bytes: PK\x03\x04
const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

/// Geometry extracted from a PolygonalLOD element: (positions, normals, tex_coords, indices, line_indices).
type LodGeometry = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>, Vec<u32>);

/// Load a 3DXML file from disk (ZIP archive or raw XML), producing a `World`.
pub fn load_3dxml(path: &Path) -> Result<World> {
    let path_str = path.to_string_lossy();
    let bytes = std::fs::read(path)
        .map_err(|e| IoError::FileNotFound(format!("{path_str}: {e}")))?;
    load_3dxml_from_bytes(&bytes, &path_str)
}

/// Load a 3DXML file from in-memory bytes (auto-detects ZIP vs raw XML).
pub fn load_3dxml_from_bytes(bytes: &[u8], name: &str) -> Result<World> {
    if bytes.len() >= 4 && bytes[0..4] == ZIP_MAGIC {
        let cursor = std::io::Cursor::new(bytes);
        load_3dxml_from_zip(cursor, name)
    } else {
        let xml = std::str::from_utf8(bytes)
            .map_err(|e| IoError::ThreeDxmlError(format!("{name}: invalid UTF-8: {e}")))?;
        load_3dxml_from_raw_xml(xml, name)
    }
}

/// Load from a ZIP archive containing Manifest.xml + root XML + optional .3DRep files.
fn load_3dxml_from_zip<R: Read + Seek>(reader: R, name: &str) -> Result<World> {
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| IoError::ThreeDxmlError(format!("{name}: failed to open ZIP: {e}")))?;

    let mut parser = Parser::new(name);
    parser.load_manifest(&mut archive)?;

    // Read root XML from ZIP and parse it (handles ProductStructure + inline reps)
    let root_xml = read_zip_entry_insensitive(&mut archive, &parser.root_name, name)?;
    parser.parse_root_xml(&root_xml)?;

    // Also try loading external .3DRep files from ZIP (v4 style)
    parser.load_representations_from_zip(&mut archive)?;

    // Load referenced images from ZIP and decode them
    parser.load_images_from_zip(&mut archive);

    let world = parser.build_world()?;
    Ok(world)
}

/// Load from raw XML (no ZIP, no manifest).
fn load_3dxml_from_raw_xml(xml: &str, name: &str) -> Result<World> {
    let mut parser = Parser::new(name);
    parser.parse_root_xml(xml)?;
    let world = parser.build_world()?;
    Ok(world)
}

// ── Internal data structures ────────────────────────────────────────────

/// Assembly link: an Instance3D connecting a parent Reference3D to a child.
struct AssyLink {
    parent_ref_id: u32,
    instance_id: u32,
    instance_name: String,
    instance_of_ref: InstanceTarget,
    matrix: Transform,
}

#[allow(dead_code)]
enum InstanceTarget {
    Local(u32),
    External(String),
}

/// Representation link: an InstanceRep connecting a Reference3D to a ReferenceRep.
struct RepLink {
    reference_id: u32,
    rep_id: u32,
}

/// Parsed geometry for a single representation.
struct RepGeometry {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

// ── Parser state ────────────────────────────────────────────────────────

struct Parser {
    name: String,
    root_name: String,
    /// Schema version from Header (e.g., "3.0", "4.0")
    schema_version: String,
    /// Header title
    header_title: String,
    /// Header generator
    header_generator: String,
    /// Reference3D: id → name
    references: HashMap<u32, String>,
    /// Assembly links (Instance3D)
    assy_links: Vec<AssyLink>,
    /// ReferenceRep: id → associated file/repId string
    reference_reps: HashMap<u32, String>,
    /// Local rep links (InstanceRep with local ref)
    local_rep_links: Vec<RepLink>,
    /// Extern rep links (InstanceRep with external ref)
    extern_rep_links: Vec<RepLink>,
    /// Loaded geometries from external .3DRep files (keyed by rep id string)
    geometries: HashMap<String, RepGeometry>,
    /// Loaded geometries from inline Representation elements (keyed by id attr)
    inline_geometries: HashMap<String, RepGeometry>,
    /// Material hash for deduplication (key = color hash string)
    material_cache: HashMap<String, Material>,
    /// Default camera eye from DefaultView/Viewpoint
    default_camera_eye: Option<[f32; 3]>,
    /// Default camera target from DefaultView/Viewpoint
    default_camera_target: Option<[f32; 3]>,
    /// CATRepImage: id → associated filename in ZIP (e.g., "urn:3DXML:image.jpg" → "image.jpg")
    image_files: HashMap<String, String>,
}

impl Parser {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            root_name: String::new(),
            schema_version: String::new(),
            header_title: String::new(),
            header_generator: String::new(),
            references: HashMap::new(),
            assy_links: Vec::new(),
            reference_reps: HashMap::new(),
            local_rep_links: Vec::new(),
            extern_rep_links: Vec::new(),
            geometries: HashMap::new(),
            inline_geometries: HashMap::new(),
            material_cache: HashMap::new(),
            default_camera_eye: None,
            default_camera_target: None,
            image_files: HashMap::new(),
        }
    }

    // ── Manifest ────────────────────────────────────────────────────────

    fn load_manifest<R: Read + Seek>(&mut self, archive: &mut zip::ZipArchive<R>) -> Result<()> {
        let xml = read_zip_entry_insensitive(archive, "Manifest.xml", &self.name)?;
        let mut reader = Reader::from_str(&xml);
        let mut in_root = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Root" => {
                    in_root = true;
                }
                Ok(Event::Text(ref t)) if in_root => {
                    self.root_name = t.unescape().unwrap_or_default().trim().to_string();
                    in_root = false;
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Root" => {
                    in_root = false;
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IoError::ThreeDxmlError(format!(
                        "{}: Manifest.xml parse error: {e}",
                        self.name
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        if self.root_name.is_empty() {
            return Err(IoError::ThreeDxmlError(format!(
                "{}: Manifest.xml missing Root element",
                self.name
            )));
        }
        Ok(())
    }

    // ── Unified Root XML Parser ─────────────────────────────────────────

    /// Parse the root XML document in a single pass, dispatching to subsections.
    /// Handles both v3 (inline reps) and v4 (separate .3DRep) formats.
    fn parse_root_xml(&mut self, xml: &str) -> Result<()> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"Header" => self.parse_header(&mut reader)?,
                        b"ProductStructure" => {
                            self.parse_product_structure_inner(&mut reader)?;
                        }
                        b"GeometricRepresentationSet" => {
                            self.parse_geometric_representation_set(&mut reader)?;
                        }
                        b"DefaultView" => self.parse_default_view(&mut reader)?,
                        b"ImageSet" => skip_element(&mut reader),
                        b"CATMaterialRef" => skip_element(&mut reader),
                        b"CATMaterial" => skip_element(&mut reader),
                        b"CATRepImage" => {
                            self.parse_cat_rep_image(e);
                            skip_element(&mut reader);
                        }
                        b"PROCESS" => skip_element(&mut reader),
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IoError::ThreeDxmlError(format!(
                        "{}: root XML parse error: {e}",
                        self.name
                    )));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    // ── Header ──────────────────────────────────────────────────────────

    fn parse_header(&mut self, reader: &mut Reader<&[u8]>) -> Result<()> {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                    b"SchemaVersion" => {
                        self.schema_version = read_text_content(reader);
                    }
                    b"Title" => {
                        self.header_title = read_text_content(reader);
                    }
                    b"Generator" => {
                        self.header_generator = read_text_content(reader);
                    }
                    _ => {}
                },
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Header" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(())
    }

    // ── DefaultView ───────────────────────────────────────────────────

    /// Parse <CATRepImage> attributes to extract image filename from ZIP.
    /// Format: <CATRepImage id="nn" name="..." format="jpg" associatedFile="urn:3DXML:path/image.jpg"/>
    fn parse_cat_rep_image(&mut self, e: &quick_xml::events::BytesStart) {
        let id = get_attr_str(e, b"id").unwrap_or_default();
        if let Some(assoc) = get_attr_str(e, b"associatedFile") {
            // Strip "urn:3DXML:" prefix to get the filename within the ZIP
            let filename = assoc
                .strip_prefix("urn:3DXML:")
                .unwrap_or(&assoc)
                .to_string();
            if !filename.is_empty() {
                self.image_files.insert(id, filename);
            }
        }
    }

    /// Parse <DefaultView> → <DefaultViewProperty> → <Viewpoint>
    /// Viewpoint contains 9 space-separated floats: eye(3) target(3) up(3)
    fn parse_default_view(&mut self, reader: &mut Reader<&[u8]>) -> Result<()> {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Viewpoint" => {
                    let text = read_text_content(reader);
                    let floats: Vec<f32> = text
                        .split_whitespace()
                        .filter_map(|v| v.parse().ok())
                        .collect();
                    if floats.len() >= 6 {
                        self.default_camera_eye = Some([floats[0], floats[1], floats[2]]);
                        self.default_camera_target = Some([floats[3], floats[4], floats[5]]);
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"DefaultView" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(())
    }

    // ── Product Structure ───────────────────────────────────────────────

    /// Parse ProductStructure content from an already-positioned reader.
    fn parse_product_structure_inner(&mut self, reader: &mut Reader<&[u8]>) -> Result<()> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    match local.as_ref() {
                        b"Reference3D" => self.parse_reference3d(e)?,
                        b"Instance3D" => self.parse_instance3d(reader, e)?,
                        b"ReferenceRep" => self.parse_reference_rep(e)?,
                        b"InstanceRep" => self.parse_instance_rep(reader)?,
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let local = e.local_name();
                    match local.as_ref() {
                        b"Reference3D" => self.parse_reference3d(e)?,
                        b"ReferenceRep" => self.parse_reference_rep(e)?,
                        _ => {}
                    }
                }
                Ok(Event::End(ref e))
                    if e.local_name().as_ref() == b"ProductStructure" =>
                {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(IoError::ThreeDxmlError(format!(
                        "{}: ProductStructure parse error: {e}",
                        self.name
                    )));
                }
                _ => {}
            }
            buf.clear();
        }
        Ok(())
    }

    fn parse_reference3d(&mut self, e: &quick_xml::events::BytesStart) -> Result<()> {
        let id = get_attr_u32(e, b"id")?;
        let name = get_attr_str(e, b"name").unwrap_or_default();
        self.references.insert(id, name);
        Ok(())
    }

    fn parse_instance3d(
        &mut self,
        reader: &mut Reader<&[u8]>,
        start: &quick_xml::events::BytesStart,
    ) -> Result<()> {
        let instance_id = get_attr_u32(start, b"id")?;
        let inst_name = get_attr_str(start, b"name").unwrap_or_default();

        let mut aggregated_by: u32 = 0;
        let mut instance_of_str = String::new();
        let mut matrix_str = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    match local.as_ref() {
                        b"IsAggregatedBy" => {
                            aggregated_by = read_text_content(reader)
                                .parse::<u32>()
                                .unwrap_or(0);
                        }
                        b"IsInstanceOf" => {
                            instance_of_str = read_text_content(reader);
                        }
                        b"RelativeMatrix" => {
                            matrix_str = read_text_content(reader);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Instance3D" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let matrix = parse_matrix(&matrix_str);

        let target = parse_instance_target(&instance_of_str);

        self.assy_links.push(AssyLink {
            parent_ref_id: aggregated_by,
            instance_id,
            instance_name: inst_name,
            instance_of_ref: target,
            matrix,
        });

        Ok(())
    }

    fn parse_reference_rep(&mut self, e: &quick_xml::events::BytesStart) -> Result<()> {
        let id = get_attr_u32(e, b"id")?;
        let format = get_attr_str(e, b"format").unwrap_or_default();
        let associated_file = get_attr_str(e, b"associatedFile").unwrap_or_default();

        if format.eq_ignore_ascii_case("TESSELLATED") || format.is_empty() {
            let rep_id = strip_urn_prefix(&associated_file);
            self.reference_reps.insert(id, rep_id);
        }
        Ok(())
    }

    fn parse_instance_rep(&mut self, reader: &mut Reader<&[u8]>) -> Result<()> {
        let mut aggregated_by: u32 = 0;
        let mut instance_of_str = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    match local.as_ref() {
                        b"IsAggregatedBy" => {
                            aggregated_by = read_text_content(reader)
                                .parse::<u32>()
                                .unwrap_or(0);
                        }
                        b"IsInstanceOf" => {
                            instance_of_str = read_text_content(reader);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"InstanceRep" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let rep_id_str = strip_urn_prefix(&instance_of_str);
        if let Ok(rep_id) = rep_id_str.parse::<u32>() {
            if instance_of_str.contains("urn:3DXML:Reference:loc:") {
                self.local_rep_links.push(RepLink {
                    reference_id: aggregated_by,
                    rep_id,
                });
            } else {
                self.extern_rep_links.push(RepLink {
                    reference_id: aggregated_by,
                    rep_id,
                });
            }
        }
        Ok(())
    }

    // ── Inline Representation loading (v3 format) ───────────────────────

    /// Parse <GeometricRepresentationSet> containing inline <Representation> elements.
    fn parse_geometric_representation_set(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Result<()> {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Representation" => {
                    let rep_id = get_attr_str(e, b"id").unwrap_or_default();
                    let geom = self.parse_representation_element(reader)?;
                    if !geom.meshes.is_empty() {
                        self.inline_geometries.insert(rep_id, geom);
                    }
                }
                Ok(Event::End(ref e))
                    if e.local_name().as_ref() == b"GeometricRepresentationSet" =>
                {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(())
    }

    /// Parse a single <Representation> element which may contain:
    /// - Direct <Rep xsi:type="PolygonalRepType"> children
    /// - <AssociatedXML xsi:type="BagRepType"> wrapping <Rep> children
    fn parse_representation_element(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Result<RepGeometry> {
        let mut geom = RepGeometry {
            meshes: Vec::new(),
            materials: Vec::new(),
        };
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"Rep" | b"Root" => {
                            let xsi_type =
                                get_attr_str(e, b"xsi:type").unwrap_or_default();
                            if xsi_type == "PolygonalRepType" {
                                self.parse_polygonal_rep(reader, &mut geom)?;
                            }
                        }
                        // AssociatedXML / BagRepType is a container — descend into it
                        b"AssociatedXML" => {
                            // Continue — Rep elements will appear inside
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Representation" => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(geom)
    }

    // ── External .3DRep loading (v4 format) ─────────────────────────────

    fn load_representations_from_zip<R: Read + Seek>(
        &mut self,
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<()> {
        let rep_files: Vec<(String, String)> = self
            .reference_reps
            .iter()
            .map(|(id, rep_id)| (id.to_string(), rep_id.clone()))
            .collect();

        for (_ref_id, rep_id) in &rep_files {
            // Skip if already loaded (from inline parsing or earlier)
            if self.geometries.contains_key(rep_id)
                || self.inline_geometries.contains_key(rep_id)
            {
                continue;
            }

            // Try loading from archive — rep_id may be a filename or an inline id
            let xml = if let Ok(data) = read_zip_entry_insensitive(archive, rep_id, &self.name) {
                data
            } else {
                continue;
            };

            let geom = self.parse_representation_xml(&xml)?;
            if !geom.meshes.is_empty() {
                self.geometries.insert(rep_id.clone(), geom);
            }
        }
        Ok(())
    }

    /// Load images referenced by CATRepImage from the ZIP archive.
    /// Decoded textures are stored as TextureData and will be associated with materials
    /// that have matching texture_path references.
    fn load_images_from_zip<R: Read + Seek>(&mut self, archive: &mut zip::ZipArchive<R>) {
        use glc_core::material::TextureData;

        if self.image_files.is_empty() {
            return;
        }

        // Load and decode each referenced image
        let mut decoded: HashMap<String, TextureData> = HashMap::new();
        for (id, filename) in &self.image_files {
            if let Ok(bytes) = read_zip_entry_bytes_insensitive(archive, filename) {
                if let Some(tex_data) = crate::texture::decode_texture_bytes(&bytes) {
                    log::debug!("Decoded texture '{}' ({}x{}) from ZIP", filename, tex_data.width, tex_data.height);
                    decoded.insert(id.clone(), tex_data);
                } else {
                    log::warn!("Failed to decode image '{}' from 3DXML ZIP", filename);
                }
            }
        }

        if decoded.is_empty() {
            return;
        }

        // Associate decoded images with materials that have matching texture_path
        for geom in self.geometries.values_mut().chain(self.inline_geometries.values_mut()) {
            for mat in &mut geom.materials {
                if let Some(ref path) = mat.texture_path {
                    // Check if any decoded image filename matches this texture path
                    for (id, tex_data) in &decoded {
                        let img_filename = self.image_files.get(id).map(|s| s.as_str()).unwrap_or("");
                        if path.contains(img_filename) || img_filename.contains(path.as_str()) {
                            mat.texture_data = Some(tex_data.clone());
                            break;
                        }
                    }
                }
            }
        }

        // If there's exactly one image and materials don't have texture_path but have texture references,
        // try to assign the single image to all materials (common case for simple textured models)
        if decoded.len() == 1 {
            let single_tex = decoded.into_values().next().unwrap();
            for geom in self.geometries.values_mut().chain(self.inline_geometries.values_mut()) {
                for mat in &mut geom.materials {
                    if mat.texture_data.is_none() && mat.texture_path.is_some() {
                        mat.texture_data = Some(single_tex.clone());
                    }
                }
            }
        }
    }

    fn parse_representation_xml(&mut self, xml: &str) -> Result<RepGeometry> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut geom = RepGeometry {
            meshes: Vec::new(),
            materials: Vec::new(),
        };

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let is_rep = e.local_name().as_ref() == b"Rep"
                        || e.local_name().as_ref() == b"Root";
                    if is_rep {
                        let xsi_type = get_attr_str(e, b"xsi:type").unwrap_or_default();
                        if xsi_type == "PolygonalRepType" {
                            self.parse_polygonal_rep(&mut reader, &mut geom)?;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(geom)
    }

    // ── Polygonal rep parsing ───────────────────────────────────────────

    fn parse_polygonal_rep(
        &mut self,
        reader: &mut Reader<&[u8]>,
        geom: &mut RepGeometry,
    ) -> Result<()> {
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut tex_coords: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut line_indices: Vec<u32> = Vec::new();
        let mut face_materials: Vec<(u32, Option<Material>)> = Vec::new();
        let mut current_surface_material: Option<Material> = None;
        // LOD tracking: keep the most detailed (lowest accuracy) LOD
        let mut best_lod: Option<LodGeometry> = None;
        let mut best_lod_accuracy: f32 = f32::MAX;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local = e.local_name();
                    match local.as_ref() {
                        b"SurfaceAttributes" => {
                            current_surface_material =
                                self.parse_surface_attributes(reader);
                        }
                        b"Face" => {
                            let start_idx = indices.len() as u32;
                            let face_mat = self.parse_face(reader, e, &mut indices);
                            let mat = face_mat.or(current_surface_material.clone());
                            let count = indices.len() as u32 - start_idx;
                            if count > 0 {
                                face_materials.push((start_idx, mat));
                            }
                        }
                        b"Positions" => {
                            let text = read_text_content(reader);
                            positions = parse_float_list(&text);
                        }
                        b"Normals" => {
                            let text = read_text_content(reader);
                            normals = parse_float_list(&text);
                        }
                        b"TextureCoordinates" => {
                            let text = read_text_content(reader);
                            tex_coords = parse_float_list(&text);
                        }
                        b"PolygonalLOD" => {
                            let accuracy = get_attr_f32(e, b"accuracy").unwrap_or(f32::MAX);
                            let lod_geom = self.parse_lod_geometry(reader)?;
                            if let Some(ref lg) = lod_geom {
                                if !lg.0.is_empty() && accuracy < best_lod_accuracy {
                                    best_lod_accuracy = accuracy;
                                    best_lod = lod_geom;
                                }
                            }
                        }
                        b"Edges" => {
                            parse_edges(reader, &mut line_indices);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"Face" => {
                    let start_idx = indices.len() as u32;
                    let face_mat = self.parse_face_empty(e, &mut indices);
                    let mat = face_mat.or(current_surface_material.clone());
                    let count = indices.len() as u32 - start_idx;
                    if count > 0 {
                        face_materials.push((start_idx, mat));
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"Rep" || local.as_ref() == b"Root" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        // If no direct geometry found, use the best LOD data
        if positions.is_empty() || indices.is_empty() {
            if let Some((lod_pos, lod_norm, lod_tex, lod_idx, lod_lines)) = best_lod {
                positions = lod_pos;
                normals = lod_norm;
                tex_coords = lod_tex;
                indices = lod_idx;
                line_indices = lod_lines;
            }
        }

        if positions.is_empty() || indices.is_empty() {
            return Ok(());
        }

        // If normals are missing, compute them
        if normals.len() != positions.len() {
            normals = compute_normals(&positions, &indices);
        }

        // Build material ranges
        let mut materials_for_mesh: Vec<Material> = Vec::new();
        let mut material_ranges: Vec<MaterialRange> = Vec::new();

        let base_mat_idx = geom.materials.len();

        if face_materials.is_empty() || face_materials.iter().all(|(_, m)| m.is_none()) {
            // Single default material
            let mat = Material::default();
            materials_for_mesh.push(mat);
            material_ranges.push(MaterialRange {
                material_index: base_mat_idx,
                start: 0,
                count: indices.len() as u32,
            });
        } else {
            // Group consecutive faces with same material
            let mut mat_map: HashMap<String, usize> = HashMap::new();
            for (start, mat_opt) in &face_materials {
                let mat = mat_opt.clone().unwrap_or_default();
                let key = material_key(&mat);
                let idx = if let Some(&existing) = mat_map.get(&key) {
                    existing
                } else {
                    let idx = base_mat_idx + materials_for_mesh.len();
                    mat_map.insert(key, idx);
                    materials_for_mesh.push(mat);
                    idx
                };

                // Determine count: distance to next face_materials entry or end
                let end = face_materials
                    .iter()
                    .find(|(s, _)| *s > *start)
                    .map(|(s, _)| *s)
                    .unwrap_or(indices.len() as u32);

                let count = end - start;
                // Merge with previous if same material
                if let Some(last) = material_ranges.last_mut() {
                    if last.material_index == idx && last.start + last.count == *start {
                        last.count += count;
                        continue;
                    }
                }
                material_ranges.push(MaterialRange {
                    material_index: idx,
                    start: *start,
                    count,
                });
            }
        }

        let mesh = Mesh {
            id: EntityId::new(),
            name: self.name.clone(),
            positions,
            normals,
            tex_coords,
            indices,
            line_indices,
            material_ranges,
            lod: 0,
        };
        geom.meshes.push(mesh);
        geom.materials.extend(materials_for_mesh);
        Ok(())
    }

    /// Parse geometry inside a `<PolygonalLOD>` element.
    /// Returns (positions, normals, tex_coords, indices) or None if empty.
    fn parse_lod_geometry(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Result<Option<LodGeometry>> {
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut tex_coords: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut line_indices: Vec<u32> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"Face" => {
                            self.parse_face(reader, e, &mut indices);
                        }
                        b"Positions" => {
                            let text = read_text_content(reader);
                            positions = parse_float_list(&text);
                        }
                        b"Normals" => {
                            let text = read_text_content(reader);
                            normals = parse_float_list(&text);
                        }
                        b"TextureCoordinates" => {
                            let text = read_text_content(reader);
                            tex_coords = parse_float_list(&text);
                        }
                        b"Edges" => {
                            parse_edges(reader, &mut line_indices);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"Face" => {
                    parse_face_indices(e, &mut indices);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"PolygonalLOD" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        if positions.is_empty() || indices.is_empty() {
            Ok(None)
        } else {
            Ok(Some((positions, normals, tex_coords, indices, line_indices)))
        }
    }

    /// Parse a <Face> start element and its children for material, collecting indices.
    fn parse_face(
        &mut self,
        reader: &mut Reader<&[u8]>,
        start: &quick_xml::events::BytesStart,
        indices: &mut Vec<u32>,
    ) -> Option<Material> {
        parse_face_indices(start, indices);

        let mut face_mat: Option<Material> = None;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"SurfaceAttributes" => {
                    face_mat = self.parse_surface_attributes(reader);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Face" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        face_mat
    }

    /// Parse an empty <Face .../> element.
    fn parse_face_empty(
        &mut self,
        e: &quick_xml::events::BytesStart,
        indices: &mut Vec<u32>,
    ) -> Option<Material> {
        parse_face_indices(e, indices);
        None
    }

    // ── Material parsing ────────────────────────────────────────────────

    fn parse_surface_attributes(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Option<Material> {
        let mut result: Option<Material> = None;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                    b"Color" => {
                        result = Some(self.parse_color(e));
                    }
                    b"MaterialApplication" => {
                        result = self.parse_material_application(reader);
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => {
                    if e.local_name().as_ref() == b"Color" {
                        result = Some(self.parse_color(e));
                    }
                }
                Ok(Event::End(ref e))
                    if e.local_name().as_ref() == b"SurfaceAttributes" =>
                {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        result
    }

    /// Parse <MaterialApplication> containing <Material xsi:type="BasicMaterialType|TextureMaterialType">.
    fn parse_material_application(
        &mut self,
        reader: &mut Reader<&[u8]>,
    ) -> Option<Material> {
        let mut result: Option<Material> = None;
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Material" => {
                    let xsi_type = get_attr_str(e, b"xsi:type").unwrap_or_default();
                    match xsi_type.as_str() {
                        "BasicMaterialType" | "TextureMaterialType" => {
                            result = Some(self.parse_basic_material(e, reader));
                        }
                        _ => {
                            // Unknown material type — try parsing as basic anyway
                            result = Some(self.parse_basic_material(e, reader));
                        }
                    }
                }
                Ok(Event::End(ref e))
                    if e.local_name().as_ref() == b"MaterialApplication" =>
                {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        result
    }

    /// Parse <Material xsi:type="BasicMaterialType"> with coefficient attributes
    /// and <Ambient>, <Diffuse>, <Specular> RGBAColorType children.
    fn parse_basic_material(
        &mut self,
        start: &quick_xml::events::BytesStart,
        reader: &mut Reader<&[u8]>,
    ) -> Material {
        let ambient_coef = get_attr_f32(start, b"ambientCoef").unwrap_or(1.0);
        let diffuse_coef = get_attr_f32(start, b"diffuseCoef").unwrap_or(1.0);
        let specular_coef = get_attr_f32(start, b"specularCoef").unwrap_or(1.0);
        let specular_exp = get_attr_f32(start, b"specularExponent").unwrap_or(35.0);
        let transparency = get_attr_f32(start, b"transparencyCoef").unwrap_or(0.0);

        let mut ambient_color: Option<Color4f> = None;
        let mut diffuse_color: Option<Color4f> = None;
        let mut specular_color: Option<Color4f> = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    match e.local_name().as_ref() {
                        b"Ambient" => ambient_color = Some(parse_rgba_color(e)),
                        b"Diffuse" => diffuse_color = Some(parse_rgba_color(e)),
                        b"Specular" => specular_color = Some(parse_rgba_color(e)),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Material" => break,
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        // Apply coefficients to colors
        let ambient = ambient_color
            .map(|c| {
                Color4f::new(
                    c.r * ambient_coef,
                    c.g * ambient_coef,
                    c.b * ambient_coef,
                    c.a,
                )
            })
            .unwrap_or(Color4f::new(0.2, 0.2, 0.2, 1.0));
        let diffuse = diffuse_color
            .map(|c| {
                Color4f::new(
                    c.r * diffuse_coef,
                    c.g * diffuse_coef,
                    c.b * diffuse_coef,
                    c.a,
                )
            })
            .unwrap_or(Color4f::new(0.8, 0.8, 0.8, 1.0));
        let specular = specular_color
            .map(|c| {
                Color4f::new(
                    c.r * specular_coef,
                    c.g * specular_coef,
                    c.b * specular_coef,
                    c.a,
                )
            })
            .unwrap_or(Color4f::new(0.27, 0.27, 0.27, 1.0));

        let opacity = 1.0 - transparency;

        // Deduplicate via cache
        let key = format!(
            "BM:{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            diffuse.r, diffuse.g, diffuse.b, ambient.r, ambient.g, ambient.b, specular_exp, opacity
        );
        if let Some(cached) = self.material_cache.get(&key) {
            return cached.clone();
        }

        let mat = Material {
            name: format!("Material_{}", self.material_cache.len()),
            ambient,
            diffuse,
            specular,
            shininess: specular_exp,
            opacity,
            ..Material::default()
        };
        self.material_cache.insert(key, mat.clone());
        mat
    }

    fn parse_color(&mut self, e: &quick_xml::events::BytesStart) -> Material {
        let r = get_attr_f32(e, b"red").unwrap_or(0.8);
        let g = get_attr_f32(e, b"green").unwrap_or(0.8);
        let b = get_attr_f32(e, b"blue").unwrap_or(0.8);
        let a = get_attr_f32(e, b"alpha").unwrap_or(1.0);

        let key = format!("{r:.4},{g:.4},{b:.4},{a:.4}");
        if let Some(cached) = self.material_cache.get(&key) {
            return cached.clone();
        }

        let mat = Material {
            name: format!("Material_{}", self.material_cache.len()),
            ambient: Color4f::new(0.2, 0.2, 0.2, 1.0),
            diffuse: Color4f::new(r, g, b, 1.0),
            specular: Color4f::new(0.27, 0.27, 0.27, 1.0),
            shininess: 35.0,
            opacity: a,
            ..Material::default()
        };
        self.material_cache.insert(key, mat.clone());
        mat
    }

    // ── Build World ─────────────────────────────────────────────────────

    fn build_world(&self) -> Result<World> {
        let mut world = World::new();
        world.source_path = Some(self.name.clone());
        world.schema_version = (!self.schema_version.is_empty()).then(|| self.schema_version.clone());
        world.header_title = (!self.header_title.is_empty()).then(|| self.header_title.clone());
        world.header_generator = (!self.header_generator.is_empty()).then(|| self.header_generator.clone());
        world.default_camera_eye = self.default_camera_eye;
        world.default_camera_target = self.default_camera_target;

        // Create a map from reference_id → rep geometry key
        let mut ref_to_rep: HashMap<u32, String> = HashMap::new();
        // Process local rep links
        for link in &self.local_rep_links {
            if let Some(rep_id_str) = self.reference_reps.get(&link.rep_id) {
                ref_to_rep.insert(link.reference_id, rep_id_str.clone());
            }
        }
        // Process extern rep links
        for link in &self.extern_rep_links {
            if let Some(rep_id_str) = self.reference_reps.get(&link.rep_id) {
                ref_to_rep.insert(link.reference_id, rep_id_str.clone());
            }
        }

        // Upload geometry: for each reference that has geometry, store mesh indices
        let mut ref_mesh_indices: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut ref_mat_offsets: HashMap<u32, usize> = HashMap::new();

        for (ref_id, rep_key) in &ref_to_rep {
            // Look in both geometry stores (external .3DRep and inline)
            let geom = self
                .geometries
                .get(rep_key)
                .or_else(|| self.inline_geometries.get(rep_key));

            if let Some(geom) = geom {
                let mat_offset = world.materials.len();
                ref_mat_offsets.insert(*ref_id, mat_offset);

                for mat in &geom.materials {
                    world.add_material(mat.clone());
                }

                let mut mesh_idxs = Vec::new();
                for mesh in &geom.meshes {
                    let mut m = mesh.clone();
                    // Adjust material indices by offset
                    for range in &mut m.material_ranges {
                        range.material_index += mat_offset;
                    }
                    let idx = world.add_mesh(m);
                    mesh_idxs.push(idx);
                }
                ref_mesh_indices.insert(*ref_id, mesh_idxs);
            }
        }

        // Build scene tree from assy_links
        // First, find root reference (id=1 or the one not referenced as a child)
        let root_ref_id = if self.references.contains_key(&1) {
            1
        } else {
            *self.references.keys().min().unwrap_or(&1)
        };

        let root_name = self.references.get(&root_ref_id).cloned().unwrap_or_default();

        // Create root node
        let root_node_idx = if let Some(mesh_idxs) = ref_mesh_indices.get(&root_ref_id) {
            if let Some(&first_mesh) = mesh_idxs.first() {
                world.add_node(SceneNode::with_mesh(&root_name, first_mesh))
            } else {
                world.add_node(SceneNode::new(&root_name))
            }
        } else {
            world.add_node(SceneNode::new(&root_name))
        };
        world.root = Some(root_node_idx);

        // Build children map: parent_ref_id → list of assy_links
        let mut children_map: HashMap<u32, Vec<&AssyLink>> = HashMap::new();
        for link in &self.assy_links {
            children_map
                .entry(link.parent_ref_id)
                .or_default()
                .push(link);
        }

        // Recursively build tree
        self.build_subtree(
            &mut world,
            root_ref_id,
            root_node_idx,
            &children_map,
            &ref_mesh_indices,
        );

        // If world has no meshes at all, return error
        if world.meshes.is_empty() {
            return Err(IoError::ThreeDxmlError(format!(
                "{}: no geometry found in 3DXML",
                self.name
            )));
        }

        // If root has no mesh and no children, add a default material
        if world.materials.is_empty() {
            world.add_material(Material::default());
        }

        Ok(world)
    }

    fn build_subtree(
        &self,
        world: &mut World,
        parent_ref_id: u32,
        parent_node_idx: glc_core::scene::NodeIndex,
        children_map: &HashMap<u32, Vec<&AssyLink>>,
        ref_mesh_indices: &HashMap<u32, Vec<usize>>,
    ) {
        let Some(children) = children_map.get(&parent_ref_id) else {
            return;
        };

        for link in children {
            let child_ref_id = match &link.instance_of_ref {
                InstanceTarget::Local(id) => *id,
                InstanceTarget::External(_) => continue, // Skip external refs for now
            };

            let child_name = if !link.instance_name.is_empty() {
                link.instance_name.clone()
            } else {
                self.references
                    .get(&child_ref_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Node_{}", link.instance_id))
            };

            // Create node — attach first mesh if geometry exists
            let mut node = if let Some(mesh_idxs) = ref_mesh_indices.get(&child_ref_id) {
                if let Some(&first_mesh) = mesh_idxs.first() {
                    SceneNode::with_mesh(&child_name, first_mesh)
                } else {
                    SceneNode::new(&child_name)
                }
            } else {
                SceneNode::new(&child_name)
            };

            node.transform = link.matrix;

            let child_node_idx = world.add_node(node);
            world.set_parent(child_node_idx, parent_node_idx);

            // If this reference has multiple meshes, add additional children
            if let Some(mesh_idxs) = ref_mesh_indices.get(&child_ref_id) {
                for &mesh_idx in mesh_idxs.iter().skip(1) {
                    let sub_name = format!("{child_name}_lod{mesh_idx}");
                    let sub_node = SceneNode::with_mesh(&sub_name, mesh_idx);
                    let sub_idx = world.add_node(sub_node);
                    world.set_parent(sub_idx, child_node_idx);
                }
            }

            // Recurse into children of this reference
            self.build_subtree(
                world,
                child_ref_id,
                child_node_idx,
                children_map,
                ref_mesh_indices,
            );
        }
    }
}

// ── Helper functions ────────────────────────────────────────────────────

/// Skip an entire XML element by tracking start/end depth.
fn skip_element(reader: &mut Reader<&[u8]>) {
    let mut buf = Vec::new();
    let mut depth = 1u32;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
}

/// Parse an RGBAColorType element's attributes into a Color4f.
fn parse_rgba_color(e: &quick_xml::events::BytesStart) -> Color4f {
    let r = get_attr_f32(e, b"red").unwrap_or(0.8);
    let g = get_attr_f32(e, b"green").unwrap_or(0.8);
    let b = get_attr_f32(e, b"blue").unwrap_or(0.8);
    let a = get_attr_f32(e, b"alpha").unwrap_or(1.0);
    Color4f::new(r, g, b, a)
}

fn read_zip_entry_insensitive<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
    file_name: &str,
) -> Result<String> {
    // Try exact name first, then case-insensitive search
    let idx = find_zip_entry(archive, name).ok_or_else(|| {
        IoError::ThreeDxmlError(format!("{file_name}: entry '{name}' not found in archive"))
    })?;

    let mut entry = archive.by_index(idx).map_err(|e| {
        IoError::ThreeDxmlError(format!("{file_name}: failed to read '{name}': {e}"))
    })?;

    let mut contents = String::new();
    entry.read_to_string(&mut contents).map_err(|e| {
        IoError::ThreeDxmlError(format!("{file_name}: failed to read '{name}': {e}"))
    })?;
    Ok(contents)
}

/// Read raw bytes from a ZIP entry (case-insensitive lookup).
fn read_zip_entry_bytes_insensitive<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> std::result::Result<Vec<u8>, String> {
    let idx = find_zip_entry(archive, name)
        .ok_or_else(|| format!("entry '{name}' not found in archive"))?;
    let mut entry = archive
        .by_index(idx)
        .map_err(|e| format!("failed to read '{name}': {e}"))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| format!("failed to read '{name}': {e}"))?;
    Ok(bytes)
}

fn find_zip_entry<R: Read + Seek>(archive: &mut zip::ZipArchive<R>, name: &str) -> Option<usize> {
    // Try exact match
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            if entry.name() == name {
                return Some(i);
            }
        }
    }
    // Case-insensitive
    let lower = name.to_lowercase();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            if entry.name().to_lowercase() == lower {
                return Some(i);
            }
        }
    }
    None
}

fn get_attr_str(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == key {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

fn get_attr_u32(e: &quick_xml::events::BytesStart, key: &[u8]) -> Result<u32> {
    get_attr_str(e, key)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| IoError::ThreeDxmlError(format!("missing attribute '{}'", String::from_utf8_lossy(key))))
}

fn get_attr_f32(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<f32> {
    get_attr_str(e, key).and_then(|s| s.parse().ok())
}

/// Read text content until the current element ends.
fn read_text_content(reader: &mut Reader<&[u8]>) -> String {
    let mut text = String::new();
    let mut buf = Vec::new();
    let mut depth = 1u32;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                if let Ok(s) = t.unescape() {
                    text.push_str(&s);
                }
            }
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    text.trim().to_string()
}

/// Parse 12-float matrix string into a Transform.
/// Format: "a11 a21 a31 a12 a22 a32 a13 a23 a33 tx ty tz"
fn parse_matrix(s: &str) -> Transform {
    let vals: Vec<f32> = s.split_whitespace().filter_map(|v| v.parse().ok()).collect();
    if vals.len() != 12 {
        return Transform::IDENTITY;
    }
    // Column-major Mat4 matching C++ GLC_Matrix4x4 layout:
    // col0=[a11,a21,a31,0], col1=[a12,a22,a32,0], col2=[a13,a23,a33,0], col3=[tx,ty,tz,1]
    let mat = glam::Mat4::from_cols(
        glam::Vec4::new(vals[0], vals[1], vals[2], 0.0),
        glam::Vec4::new(vals[3], vals[4], vals[5], 0.0),
        glam::Vec4::new(vals[6], vals[7], vals[8], 0.0),
        glam::Vec4::new(vals[9], vals[10], vals[11], 1.0),
    );
    Transform::new(mat)
}

fn parse_instance_target(s: &str) -> InstanceTarget {
    let local_prefix = "urn:3DXML:Reference:loc:";
    let ext_prefix = "urn:3DXML:Reference:ext:";

    if let Some(rest) = s.strip_prefix(ext_prefix) {
        let ext_ref = rest.replace("#1", "");
        InstanceTarget::External(ext_ref)
    } else if let Some(rest) = s.strip_prefix(local_prefix) {
        let id = rest.parse::<u32>().unwrap_or(0);
        InstanceTarget::Local(id)
    } else {
        // 3dvia style: plain number
        let id = s.trim().parse::<u32>().unwrap_or(0);
        InstanceTarget::Local(id)
    }
}

fn strip_urn_prefix(s: &str) -> String {
    let prefixes = [
        "urn:3DXML:Representation:loc:",
        "urn:3DXML:Representation:ext:",
        "urn:3DXML:Reference:loc:",
        "urn:3DXML:",
    ];
    for prefix in &prefixes {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    s.to_string()
}

/// Parse comma-or-space-separated float list.
fn parse_float_list(s: &str) -> Vec<f32> {
    s.replace(',', " ")
        .split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect()
}

/// Parse space-separated u32 index list.
fn parse_index_list(s: &str) -> Vec<u32> {
    s.replace(',', " ")
        .split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect()
}

/// Parse `<Edges>` element containing `<Polyline vertices="..."/>` children.
/// Converts polyline vertex strips into line segment pairs and appends to line_indices.
fn parse_edges(reader: &mut Reader<&[u8]>, line_indices: &mut Vec<u32>) {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"Polyline" => {
                if let Some(verts_str) = get_attr_str(e, b"vertices") {
                    let verts = parse_index_list(&verts_str);
                    // Convert strip to line segment pairs: (v0,v1), (v1,v2), ...
                    for pair in verts.windows(2) {
                        line_indices.push(pair[0]);
                        line_indices.push(pair[1]);
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"Polyline" => {
                if let Some(verts_str) = get_attr_str(e, b"vertices") {
                    let verts = parse_index_list(&verts_str);
                    for pair in verts.windows(2) {
                        line_indices.push(pair[0]);
                        line_indices.push(pair[1]);
                    }
                }
                // Skip to end of Polyline
                skip_element(reader);
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Edges" => break,
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
}

/// Parse face index attributes (triangles, strips, fans) and append to indices.
fn parse_face_indices(e: &quick_xml::events::BytesStart, indices: &mut Vec<u32>) {
    // Triangles
    if let Some(tri_str) = get_attr_str(e, b"triangles") {
        let tri_indices = parse_index_list(&tri_str);
        indices.extend_from_slice(&tri_indices);
    }
    // Strips: comma-separated strips, each strip is space-separated indices
    if let Some(strips_str) = get_attr_str(e, b"strips") {
        for strip in strips_str.split(',') {
            let strip_indices: Vec<u32> = strip
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            // Convert strip to triangles
            if strip_indices.len() >= 3 {
                for i in 0..strip_indices.len() - 2 {
                    if i % 2 == 0 {
                        indices.push(strip_indices[i]);
                        indices.push(strip_indices[i + 1]);
                        indices.push(strip_indices[i + 2]);
                    } else {
                        // Flip winding for odd triangles in strip
                        indices.push(strip_indices[i + 1]);
                        indices.push(strip_indices[i]);
                        indices.push(strip_indices[i + 2]);
                    }
                }
            }
        }
    }
    // Fans: comma-separated fans, each fan is space-separated indices
    if let Some(fans_str) = get_attr_str(e, b"fans") {
        for fan in fans_str.split(',') {
            let fan_indices: Vec<u32> = fan
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            // Convert fan to triangles
            if fan_indices.len() >= 3 {
                for i in 1..fan_indices.len() - 1 {
                    indices.push(fan_indices[0]);
                    indices.push(fan_indices[i]);
                    indices.push(fan_indices[i + 1]);
                }
            }
        }
    }
}

/// Compute face normals and accumulate per-vertex (smooth normals).
fn compute_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let num_verts = positions.len() / 3;
    let mut normals = vec![0.0f32; num_verts * 3];

    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 * 3 + 2 >= positions.len()
            || i1 * 3 + 2 >= positions.len()
            || i2 * 3 + 2 >= positions.len()
        {
            continue;
        }
        let p0 = glam::Vec3::new(
            positions[i0 * 3],
            positions[i0 * 3 + 1],
            positions[i0 * 3 + 2],
        );
        let p1 = glam::Vec3::new(
            positions[i1 * 3],
            positions[i1 * 3 + 1],
            positions[i1 * 3 + 2],
        );
        let p2 = glam::Vec3::new(
            positions[i2 * 3],
            positions[i2 * 3 + 1],
            positions[i2 * 3 + 2],
        );
        let n = (p1 - p0).cross(p2 - p0);
        for &idx in tri {
            let base = idx as usize * 3;
            normals[base] += n.x;
            normals[base + 1] += n.y;
            normals[base + 2] += n.z;
        }
    }

    // Normalize
    for chunk in normals.chunks_exact_mut(3) {
        let len = (chunk[0] * chunk[0] + chunk[1] * chunk[1] + chunk[2] * chunk[2]).sqrt();
        if len > 1e-8 {
            chunk[0] /= len;
            chunk[1] /= len;
            chunk[2] /= len;
        }
    }
    normals
}

fn material_key(mat: &Material) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4}",
        mat.diffuse.r, mat.diffuse.g, mat.diffuse.b, mat.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_3dxml() -> Vec<u8> {
        use std::io::Write;
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Manifest.xml
        zip.start_file("Manifest.xml", options).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Manifest>
  <Root>Structure.xml</Root>
</Manifest>"#
        )
        .unwrap();

        // Structure.xml (ProductStructure with a triangle)
        zip.start_file("Structure.xml", options).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml>
  <ProductStructure>
    <Reference3D id="1" name="Root"/>
    <Reference3D id="2" name="Triangle"/>
    <Instance3D id="10" name="TriInst">
      <IsAggregatedBy>1</IsAggregatedBy>
      <IsInstanceOf>2</IsInstanceOf>
      <RelativeMatrix>1 0 0 0 1 0 0 0 1 0 0 0</RelativeMatrix>
    </Instance3D>
    <ReferenceRep id="100" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:triangle.3DRep"/>
    <InstanceRep id="200">
      <IsAggregatedBy>2</IsAggregatedBy>
      <IsInstanceOf>urn:3DXML:Reference:loc:100</IsInstanceOf>
    </InstanceRep>
  </ProductStructure>
</Model_3dxml>"#
        )
        .unwrap();

        // triangle.3DRep (PolygonalRepType with 3 vertices)
        zip.start_file("triangle.3DRep", options).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<XMLRepresentation>
  <Root xsi:type="PolygonalRepType" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <Faces>
      <Face triangles="0 1 2">
        <SurfaceAttributes>
          <Color red="1.0" green="0.0" blue="0.0" alpha="1.0"/>
        </SurfaceAttributes>
      </Face>
    </Faces>
    <VertexBuffer>
      <Positions>0.0 0.0 0.0, 1.0 0.0 0.0, 0.0 1.0 0.0</Positions>
      <Normals>0.0 0.0 1.0, 0.0 0.0 1.0, 0.0 0.0 1.0</Normals>
    </VertexBuffer>
  </Root>
</XMLRepresentation>"#
        )
        .unwrap();

        let cursor = zip.finish().unwrap();
        cursor.into_inner()
    }

    #[test]
    fn test_load_3dxml_triangle() {
        let data = make_test_3dxml();
        let world = load_3dxml_from_bytes(&data, "test.3dxml").unwrap();

        assert_eq!(world.meshes.len(), 1);
        let mesh = &world.meshes[0];
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert!(!world.materials.is_empty());
    }

    #[test]
    fn test_load_3dxml_scene_tree() {
        let data = make_test_3dxml();
        let world = load_3dxml_from_bytes(&data, "test.3dxml").unwrap();

        assert!(world.root.is_some());
        let root_idx = world.root.unwrap();
        let root = &world.nodes[root_idx.0];
        assert_eq!(root.name, "Root");
        // Root should have at least one child
        assert!(!root.children.is_empty());
    }

    #[test]
    fn test_parse_matrix_identity() {
        let m = parse_matrix("1 0 0 0 1 0 0 0 1 0 0 0");
        assert_eq!(m, Transform::IDENTITY);
    }

    #[test]
    fn test_parse_matrix_translation() {
        let m = parse_matrix("1 0 0 0 1 0 0 0 1 10 20 30");
        let p = m.transform_point(glam::Vec3::ZERO);
        assert!((p.x - 10.0).abs() < 1e-6);
        assert!((p.y - 20.0).abs() < 1e-6);
        assert!((p.z - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_face_indices_triangles() {
        let xml = r#"<Face triangles="0 1 2 3 4 5"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut indices = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            parse_face_indices(e, &mut indices);
        }
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_face_indices_strips() {
        let xml = r#"<Face strips="0 1 2 3"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut indices = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            parse_face_indices(e, &mut indices);
        }
        // Strip 0,1,2,3 → triangles: (0,1,2), (2,1,3)
        assert_eq!(indices, vec![0, 1, 2, 2, 1, 3]);
    }

    #[test]
    fn test_parse_face_indices_fans() {
        let xml = r#"<Face fans="0 1 2 3"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut indices = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            parse_face_indices(e, &mut indices);
        }
        // Fan 0,1,2,3 → triangles: (0,1,2), (0,2,3)
        assert_eq!(indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn test_parse_float_list() {
        let vals = parse_float_list("1.0 2.0 3.0, 4.0 5.0 6.0");
        assert_eq!(vals, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_strip_urn_prefix() {
        assert_eq!(
            strip_urn_prefix("urn:3DXML:Representation:loc:triangle.3DRep"),
            "triangle.3DRep"
        );
        assert_eq!(strip_urn_prefix("plain_name"), "plain_name");
    }

    // ── New tests for v3/raw XML/BasicMaterialType ──────────────────────

    #[test]
    fn test_zip_detection() {
        // ZIP bytes should route to ZIP path
        let zip_data = make_test_3dxml();
        assert!(zip_data.len() >= 4);
        assert_eq!(zip_data[0..4], ZIP_MAGIC);

        // Raw XML bytes should route to raw XML path
        let xml_bytes = b"<?xml version=\"1.0\"?><Model_3dxml></Model_3dxml>";
        assert_ne!(xml_bytes[0..4], ZIP_MAGIC);
    }

    #[test]
    fn test_load_raw_xml_3dxml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<Header>
<SchemaVersion>3.0</SchemaVersion>
<Title>Test V3</Title>
</Header>
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<Reference3D id="2" name="Part1"/>
<Instance3D id="3" name="Inst1">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>2</IsInstanceOf>
  <RelativeMatrix>1 0 0 0 1 0 0 0 1 0 0 0</RelativeMatrix>
</Instance3D>
<ReferenceRep id="4" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:6"/>
<InstanceRep id="5">
  <IsAggregatedBy>2</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:4</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="6" format="TESSELLATED" version="1.1">
<AssociatedXML xsi:type="BagRepType" id="607">
<Rep xsi:type="PolygonalRepType" id="608">
  <Faces>
    <Face triangles="0 1 2"/>
  </Faces>
  <VertexBuffer>
    <Positions>0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 0.0</Positions>
    <Normals>0.0 0.0 1.0 0.0 0.0 1.0 0.0 0.0 1.0</Normals>
  </VertexBuffer>
</Rep>
</AssociatedXML>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_v3.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert_eq!(world.meshes[0].vertex_count(), 3);
        assert_eq!(world.meshes[0].face_count(), 1);
        assert!(!world.materials.is_empty());
    }

    #[test]
    fn test_load_v3_zip_with_inline_reps() {
        use std::io::Write;
        let buf = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("Manifest.xml", options).unwrap();
        write!(zip, r#"<?xml version="1.0"?><Manifest><Root>root.3dxml</Root></Manifest>"#).unwrap();

        zip.start_file("root.3dxml", options).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<Header><SchemaVersion>3.0</SchemaVersion></Header>
<ProductStructure root="1">
<Reference3D id="1" name="Assembly"/>
<Reference3D id="2" name="Cube"/>
<Instance3D id="3" name="CubeInst">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>2</IsInstanceOf>
  <RelativeMatrix>1 0 0 0 1 0 0 0 1 5 0 0</RelativeMatrix>
</Instance3D>
<ReferenceRep id="10" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:20"/>
<InstanceRep id="11">
  <IsAggregatedBy>2</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:10</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="20" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <Faces>
    <Face triangles="0 1 2 2 3 0"/>
  </Faces>
  <VertexBuffer>
    <Positions>0 0 0 1 0 0 1 1 0 0 1 0</Positions>
    <Normals>0 0 1 0 0 1 0 0 1 0 0 1</Normals>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#
        )
        .unwrap();

        let cursor = zip.finish().unwrap();
        let data = cursor.into_inner();

        let world = load_3dxml_from_bytes(&data, "v3_zip.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert_eq!(world.meshes[0].vertex_count(), 4);
        assert_eq!(world.meshes[0].face_count(), 2);
        assert!(world.root.is_some());
    }

    #[test]
    fn test_parse_basic_material_type() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <SurfaceAttributes>
    <MaterialApplication xsi:type="MaterialApplicationType">
      <Material xsi:type="BasicMaterialType" ambientCoef="0.5" diffuseCoef="1.0" specularCoef="0.8" specularExponent="50.0" transparencyCoef="0.1">
        <Ambient xsi:type="RGBAColorType" red="0.4" green="0.4" blue="0.4" alpha="1"/>
        <Diffuse xsi:type="RGBAColorType" red="0.8" green="0.2" blue="0.1" alpha="1"/>
        <Specular xsi:type="RGBAColorType" red="1.0" green="1.0" blue="1.0" alpha="1"/>
      </Material>
    </MaterialApplication>
  </SurfaceAttributes>
  <Faces>
    <Face triangles="0 1 2"/>
  </Faces>
  <VertexBuffer>
    <Positions>0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 0.0</Positions>
    <Normals>0.0 0.0 1.0 0.0 0.0 1.0 0.0 0.0 1.0</Normals>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_mat.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
        assert!(!world.materials.is_empty());

        let mat = &world.materials[0];
        // Diffuse = (0.8, 0.2, 0.1) * diffuseCoef 1.0
        assert!((mat.diffuse.r - 0.8).abs() < 0.01);
        assert!((mat.diffuse.g - 0.2).abs() < 0.01);
        assert!((mat.diffuse.b - 0.1).abs() < 0.01);
        // Ambient = (0.4, 0.4, 0.4) * ambientCoef 0.5
        assert!((mat.ambient.r - 0.2).abs() < 0.01);
        assert!((mat.ambient.g - 0.2).abs() < 0.01);
        // Specular = (1.0, 1.0, 1.0) * specularCoef 0.8
        assert!((mat.specular.r - 0.8).abs() < 0.01);
        // Shininess = specularExponent
        assert!((mat.shininess - 50.0).abs() < 0.01);
        // Opacity = 1.0 - transparencyCoef 0.1 = 0.9
        assert!((mat.opacity - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_parse_header() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml>
<Header>
<SchemaVersion>3.0</SchemaVersion>
<Title>My Model</Title>
<Generator>SolidWorks 2024</Generator>
</Header>
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Faces><Face triangles="0 1 2"/></Faces>
  <VertexBuffer>
    <Positions>0 0 0 1 0 0 0 1 0</Positions>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        // Verify it loads without error (header is parsed but stored internally)
        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_header.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
    }

    #[test]
    fn test_representation_bag_rep_type() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:10"/>
<InstanceRep id="3">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="10" format="TESSELLATED">
<AssociatedXML xsi:type="BagRepType" id="100">
  <Rep xsi:type="PolygonalRepType" id="101">
    <Faces><Face triangles="0 1 2"/></Faces>
    <VertexBuffer>
      <Positions>0 0 0 2 0 0 0 2 0</Positions>
      <Normals>0 0 1 0 0 1 0 0 1</Normals>
    </VertexBuffer>
  </Rep>
  <Rep xsi:type="PolygonalRepType" id="102">
    <Faces><Face triangles="0 1 2"/></Faces>
    <VertexBuffer>
      <Positions>1 1 0 3 1 0 1 3 0</Positions>
      <Normals>0 0 1 0 0 1 0 0 1</Normals>
    </VertexBuffer>
  </Rep>
</AssociatedXML>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_bag.3dxml").unwrap();
        // BagRepType contains 2 PolygonalRep → 2 meshes
        assert_eq!(world.meshes.len(), 2);
    }

    #[test]
    #[ignore] // Requires external test file
    fn test_load_secondlife_3dxml() {
        let path = Path::new("/Users/venturahome/GLC Player Mac/3dxml/models/SecondLife.exe_Mon_Apr_13_16-35-51_2009.3dxml");
        if !path.exists() {
            eprintln!("Skipping: test file not found at {}", path.display());
            return;
        }
        let world = load_3dxml(path).unwrap();
        assert!(!world.meshes.is_empty(), "SecondLife model should have meshes");
        assert!(!world.materials.is_empty(), "SecondLife model should have materials");
        assert!(world.root.is_some(), "SecondLife model should have a scene root");
        eprintln!(
            "SecondLife: {} meshes, {} materials, {} nodes",
            world.meshes.len(),
            world.materials.len(),
            world.nodes.len()
        );
    }

    #[test]
    #[ignore] // Requires external test file
    fn test_load_model_zip_3dxml() {
        let path = Path::new("/Users/venturahome/GLC Player Mac/3dxml/models/Model.zip.3dxml");
        if !path.exists() {
            eprintln!("Skipping: test file not found at {}", path.display());
            return;
        }
        let world = load_3dxml(path).unwrap();
        assert!(!world.meshes.is_empty(), "Model.zip should have meshes");
        assert!(!world.materials.is_empty(), "Model.zip should have materials");
        eprintln!(
            "Model.zip: {} meshes, {} materials, {} nodes",
            world.meshes.len(),
            world.materials.len(),
            world.nodes.len()
        );
    }

    #[test]
    fn test_parse_default_view() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<Header><SchemaVersion>4.0</SchemaVersion></Header>
<DefaultView>
  <DefaultViewProperty>
    <Viewpoint>10.0 20.0 30.0 1.0 2.0 3.0 0.0 0.0 1.0</Viewpoint>
  </DefaultViewProperty>
</DefaultView>
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <Faces><Face triangles="0 1 2"/></Faces>
  <VertexBuffer>
    <Positions>0 0 0 1 0 0 0 1 0</Positions>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_view.3dxml").unwrap();
        let eye = world.default_camera_eye.unwrap();
        let target = world.default_camera_target.unwrap();
        assert!((eye[0] - 10.0).abs() < 1e-6);
        assert!((eye[1] - 20.0).abs() < 1e-6);
        assert!((eye[2] - 30.0).abs() < 1e-6);
        assert!((target[0] - 1.0).abs() < 1e-6);
        assert!((target[1] - 2.0).abs() < 1e-6);
        assert!((target[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_header_metadata_in_world() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<Header>
<SchemaVersion>3.0</SchemaVersion>
<Title>Test Assembly</Title>
<Generator>CATIA V5</Generator>
</Header>
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <Faces><Face triangles="0 1 2"/></Faces>
  <VertexBuffer>
    <Positions>0 0 0 1 0 0 0 1 0</Positions>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_meta.3dxml").unwrap();
        assert_eq!(world.schema_version.as_deref(), Some("3.0"));
        assert_eq!(world.header_title.as_deref(), Some("Test Assembly"));
        assert_eq!(world.header_generator.as_deref(), Some("CATIA V5"));
        assert!(world.default_camera_eye.is_none());
        assert!(world.default_camera_target.is_none());
    }

    #[test]
    fn test_polygonal_lod_parsing() {
        // Rep with two PolygonalLOD elements at different accuracies.
        // The parser should select the most detailed (lowest accuracy) LOD.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <PolygonalLOD accuracy="10.0">
    <Faces>
      <Face triangles="0 1 2"/>
    </Faces>
    <VertexBuffer>
      <Positions>0 0 0 1 0 0 0 1 0</Positions>
      <Normals>0 0 1 0 0 1 0 0 1</Normals>
    </VertexBuffer>
  </PolygonalLOD>
  <PolygonalLOD accuracy="0.5">
    <Faces>
      <Face triangles="0 1 2 2 3 0"/>
    </Faces>
    <VertexBuffer>
      <Positions>0 0 0 2 0 0 2 2 0 0 2 0</Positions>
      <Normals>0 0 1 0 0 1 0 0 1 0 0 1</Normals>
    </VertexBuffer>
  </PolygonalLOD>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_lod.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
        let mesh = &world.meshes[0];
        // The accuracy=0.5 LOD has 4 vertices and 2 faces
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
    }

    #[test]
    fn test_parse_edges() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
<ReferenceRep id="2" format="TESSELLATED" associatedFile="urn:3DXML:Representation:loc:3"/>
<InstanceRep id="4">
  <IsAggregatedBy>1</IsAggregatedBy>
  <IsInstanceOf>urn:3DXML:Reference:loc:2</IsInstanceOf>
</InstanceRep>
</ProductStructure>
<GeometricRepresentationSet>
<Representation id="3" format="TESSELLATED">
<Rep xsi:type="PolygonalRepType">
  <Faces><Face triangles="0 1 2 2 3 0"/></Faces>
  <Edges>
    <Polyline vertices="0 1 2 3"/>
  </Edges>
  <VertexBuffer>
    <Positions>0 0 0 1 0 0 1 1 0 0 1 0</Positions>
    <Normals>0 0 1 0 0 1 0 0 1 0 0 1</Normals>
  </VertexBuffer>
</Rep>
</Representation>
</GeometricRepresentationSet>
</Model_3dxml>"#;

        let world = load_3dxml_from_bytes(xml.as_bytes(), "test_edges.3dxml").unwrap();
        assert_eq!(world.meshes.len(), 1);
        let mesh = &world.meshes[0];
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
        // Polyline "0 1 2 3" → line pairs: (0,1), (1,2), (2,3)
        assert_eq!(mesh.line_indices, vec![0, 1, 1, 2, 2, 3]);
    }

    #[test]
    #[test]
    fn test_parse_cat_rep_image() {
        // CATRepImage elements at the root level should be parsed into image_files map
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Model_3dxml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<ProductStructure root="1">
<Reference3D id="1" name="Root"/>
</ProductStructure>
<CATRepImage id="42" name="diffuse_map" format="jpg" associatedFile="urn:3DXML:CATRepImage/texture.jpg">
</CATRepImage>
</Model_3dxml>"#;

        let mut parser = Parser::new("test");
        parser.parse_root_xml(xml).unwrap();
        assert_eq!(parser.image_files.len(), 1);
        assert_eq!(
            parser.image_files.get("42").unwrap(),
            "CATRepImage/texture.jpg"
        );
    }

    #[test]
    fn test_decode_texture_bytes() {
        // Create a minimal 1x1 PNG in memory
        let mut png_data = Vec::new();
        {
            let mut encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            use image::ImageEncoder;
            encoder.write_image(
                &[255u8, 0, 0, 255], // 1x1 red pixel RGBA
                1,
                1,
                image::ExtendedColorType::Rgba8,
            ).unwrap();
        }

        let tex = crate::texture::decode_texture_bytes(&png_data).unwrap();
        assert_eq!(tex.width, 1);
        assert_eq!(tex.height, 1);
        assert_eq!(tex.rgba.len(), 4);
        assert_eq!(tex.rgba[0], 255); // red
        assert_eq!(tex.rgba[3], 255); // alpha
    }

    #[ignore] // Requires external test file
    fn test_load_model2_raw_xml_3dxml() {
        let path = Path::new("/Users/venturahome/GLC Player Mac/3dxml/models/Model2.zip.3dxml");
        if !path.exists() {
            eprintln!("Skipping: test file not found at {}", path.display());
            return;
        }
        let world = load_3dxml(path).unwrap();
        assert!(!world.meshes.is_empty(), "Model2 (raw XML) should have meshes");
        assert!(!world.materials.is_empty(), "Model2 (raw XML) should have materials");
        eprintln!(
            "Model2 (raw XML): {} meshes, {} materials, {} nodes",
            world.meshes.len(),
            world.materials.len(),
            world.nodes.len()
        );
    }
}
