use anyhow::{bail, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use tracing::info;

use super::encoder::MeshFace;
use super::glc_bridge::generate_flat_normals;

#[derive(Debug, Clone)]
struct SourceData {
    floats: Vec<f32>,
    stride: usize,
}

pub fn is_collada_y_up(dae_xml: &str) -> bool {
    let mut reader = Reader::from_str(dae_xml);
    reader.config_mut().trim_text(true);
    let mut in_up_axis = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                let name = e.local_name();
                let ln = std::str::from_utf8(name.as_ref()).unwrap_or("");
                if ln == "up_axis" { in_up_axis = true; }
            }
            Ok(Event::End(_)) => { in_up_axis = false; }
            Ok(Event::Text(ref e)) if in_up_axis => {
                let text = e.unescape().unwrap_or_default().to_string();
                return text.trim() == "Y_UP";
            }
            _ => {}
        }
        buf.clear();
    }
    true
}

pub fn parse_collada_geometry(dae_xml: &str) -> Result<Vec<MeshFace>> {
    let y_up = is_collada_y_up(dae_xml);
    parse_collada_geometry_with_axis(dae_xml, y_up)
}

pub fn parse_collada_geometry_with_axis(dae_xml: &str, y_up: bool) -> Result<Vec<MeshFace>> {
    let mut reader = Reader::from_str(dae_xml);
    reader.config_mut().trim_text(true);

    let mut sources: HashMap<String, SourceData> = HashMap::new();
    let mut vertices_map: HashMap<String, String> = HashMap::new();

    let mut current_source_id = String::new();
    let mut current_vertices_id = String::new();
    let mut in_mesh = false;
    let mut in_source = false;
    let mut in_float_array = false;
    let mut in_triangles = false;
    let mut in_polylist = false;
    let mut in_p = false;
    let mut in_vcount = false;
    let mut in_vertices = false;
    let mut current_stride = 1usize;

    let mut tri_inputs: Vec<InputSemantic> = Vec::new();
    let mut p_data: Vec<usize> = Vec::new();
    let mut vcount_data: Vec<usize> = Vec::new();
    let mut input_stride = 1usize;

    let mut all_faces: Vec<MeshFace> = Vec::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let ln = e.local_name();
                let local_name = std::str::from_utf8(ln.as_ref()).unwrap_or("");

                match local_name {
                    "mesh" => {
                        in_mesh = true;
                    }
                    "source" if in_mesh && !in_triangles && !in_polylist => {
                        in_source = true;
                        current_stride = 1;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_source_id = format!("#{}", String::from_utf8_lossy(&attr.value));
                            }
                        }
                    }
                    "float_array" if in_source => {
                        in_float_array = true;
                    }
                    "accessor" if in_source => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"stride" {
                                current_stride = String::from_utf8_lossy(&attr.value).parse().unwrap_or(1);
                            }
                        }
                    }
                    "vertices" if in_mesh => {
                        in_vertices = true;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_vertices_id = format!("#{}", String::from_utf8_lossy(&attr.value));
                            }
                        }
                    }
                    "input" if in_vertices => {
                        let mut semantic = String::new();
                        let mut source_ref = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"semantic" => semantic = String::from_utf8_lossy(&attr.value).to_string(),
                                b"source" => source_ref = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        if semantic == "POSITION" && !current_vertices_id.is_empty() {
                            vertices_map.insert(current_vertices_id.clone(), source_ref);
                        }
                    }
                    "triangles" if in_mesh => {
                        in_triangles = true;
                        tri_inputs.clear();
                        p_data.clear();
                        input_stride = 1;
                    }
                    "polylist" if in_mesh => {
                        in_polylist = true;
                        tri_inputs.clear();
                        p_data.clear();
                        vcount_data.clear();
                        input_stride = 1;
                    }
                    "input" if in_triangles || in_polylist => {
                        let mut semantic = String::new();
                        let mut source_ref = String::new();
                        let mut offset = 0usize;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"semantic" => semantic = String::from_utf8_lossy(&attr.value).to_string(),
                                b"source" => source_ref = String::from_utf8_lossy(&attr.value).to_string(),
                                b"offset" => offset = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                        input_stride = input_stride.max(offset + 1);
                        tri_inputs.push(InputSemantic { semantic, source: source_ref, offset });
                    }
                    "vcount" if in_polylist => { in_vcount = true; }
                    "p" if in_triangles || in_polylist => { in_p = true; }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let ln = e.local_name();
                let local_name = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                match local_name {
                    "mesh" => { in_mesh = false; }
                    "source" => {
                        if in_source {
                            if let Some(src) = sources.get_mut(&current_source_id) {
                                src.stride = current_stride;
                            }
                        }
                        in_source = false;
                        current_source_id.clear();
                    }
                    "float_array" => { in_float_array = false; }
                    "vertices" => {
                        in_vertices = false;
                        current_vertices_id.clear();
                    }
                    "triangles" => {
                        if in_triangles && !p_data.is_empty() {
                            if let Some(f) = build_face_from_indices(
                                &tri_inputs, &p_data, &[], input_stride, &sources, &vertices_map, true, y_up,
                            ) {
                                all_faces.push(f);
                            }
                        }
                        in_triangles = false;
                    }
                    "polylist" => {
                        if in_polylist && !p_data.is_empty() {
                            if let Some(f) = build_face_from_indices(
                                &tri_inputs, &p_data, &vcount_data, input_stride, &sources, &vertices_map, false, y_up,
                            ) {
                                all_faces.push(f);
                            }
                        }
                        in_polylist = false;
                    }
                    "vcount" => { in_vcount = false; }
                    "p" => { in_p = false; }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_float_array && in_source && !current_source_id.is_empty() {
                    let vals: Vec<f32> = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    sources.insert(current_source_id.clone(), SourceData { floats: vals, stride: current_stride });
                } else if in_p {
                    p_data.extend(
                        text.split_whitespace().filter_map(|s| s.parse::<usize>().ok())
                    );
                } else if in_vcount {
                    vcount_data.extend(
                        text.split_whitespace().filter_map(|s| s.parse::<usize>().ok())
                    );
                }
            }
            Err(e) => bail!("Collada XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    if all_faces.is_empty() {
        bail!("No geometry found in Collada DAE");
    }

    info!("[COLLADA_GEOM] Parsed: {} face group(s), {} total vertices",
        all_faces.len(),
        all_faces.iter().map(|f| f.positions.len()).sum::<usize>());

    Ok(all_faces)
}

#[derive(Debug)]
struct InputSemantic {
    semantic: String,
    source: String,
    offset: usize,
}

fn resolve_source<'a>(
    source_ref: &str,
    sources: &'a HashMap<String, SourceData>,
    vertices_map: &HashMap<String, String>,
) -> Option<&'a SourceData> {
    if let Some(src) = sources.get(source_ref) {
        return Some(src);
    }
    if let Some(pos_ref) = vertices_map.get(source_ref) {
        return sources.get(pos_ref);
    }
    None
}

fn build_face_from_indices(
    inputs: &[InputSemantic],
    p_data: &[usize],
    vcount_data: &[usize],
    stride: usize,
    sources: &HashMap<String, SourceData>,
    vertices_map: &HashMap<String, String>,
    is_triangles: bool,
    y_up: bool,
) -> Option<MeshFace> {
    let mut pos_input: Option<&InputSemantic> = None;
    let mut norm_input: Option<&InputSemantic> = None;
    let mut tc_input: Option<&InputSemantic> = None;

    for input in inputs {
        match input.semantic.as_str() {
            "VERTEX" | "POSITION" => pos_input = Some(input),
            "NORMAL" => norm_input = Some(input),
            "TEXCOORD" => tc_input = Some(input),
            _ => {}
        }
    }

    let pos_in = pos_input?;
    let pos_source = resolve_source(&pos_in.source, sources, vertices_map)?;

    let norm_source = norm_input.and_then(|n| resolve_source(&n.source, sources, vertices_map));
    let tc_source = tc_input.and_then(|t| resolve_source(&t.source, sources, vertices_map));

    let mut unique_verts: HashMap<Vec<usize>, u32> = HashMap::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let mut out_indices: Vec<u32> = Vec::new();
    let mut position_indices: Vec<usize> = Vec::new();

    let triangle_indices: Vec<usize> = if is_triangles {
        (0..p_data.len() / stride).collect()
    } else {
        let mut tri_verts = Vec::new();
        let mut p_offset = 0usize;
        for &vc in vcount_data {
            if vc < 3 {
                p_offset += vc;
                continue;
            }
            for i in 1..vc - 1 {
                tri_verts.push(p_offset);
                tri_verts.push(p_offset + i);
                tri_verts.push(p_offset + i + 1);
            }
            p_offset += vc;
        }
        tri_verts
    };

    for &vert_idx in &triangle_indices {
        let base = vert_idx * stride;
        if base + stride > p_data.len() {
            continue;
        }

        let key: Vec<usize> = (0..stride).map(|i| p_data[base + i]).collect();

        let unified_idx = if let Some(&idx) = unique_verts.get(&key) {
            idx
        } else {
            let idx = positions.len() as u32;

            let pi = p_data[base + pos_in.offset];
            let ps = &pos_source.floats;
            let ps_stride = pos_source.stride.max(3);
            let p_base = pi * ps_stride;
            if p_base + 2 < ps.len() {
                let cx = ps[p_base];
                let cy = ps[p_base + 1];
                let cz = ps[p_base + 2];
                if y_up {
                    positions.push([cx, -cz, cy]);
                } else {
                    positions.push([cx, cy, cz]);
                }
            } else {
                positions.push([0.0, 0.0, 0.0]);
            }
            position_indices.push(pi);

            if let (Some(ni), Some(ns)) = (norm_input, norm_source) {
                let n_idx = p_data[base + ni.offset];
                let ns_stride = ns.stride.max(3);
                let n_base = n_idx * ns_stride;
                if n_base + 2 < ns.floats.len() {
                    let nx = ns.floats[n_base];
                    let ny = ns.floats[n_base + 1];
                    let nz = ns.floats[n_base + 2];
                    if y_up {
                        normals.push([nx, -nz, ny]);
                    } else {
                        normals.push([nx, ny, nz]);
                    }
                } else {
                    normals.push([0.0, 0.0, 1.0]);
                }
            }

            if let (Some(ti), Some(ts)) = (tc_input, tc_source) {
                let t_idx = p_data[base + ti.offset];
                let ts_stride = ts.stride.max(2);
                let t_base = t_idx * ts_stride;
                if t_base + 1 < ts.floats.len() {
                    tex_coords.push([ts.floats[t_base], ts.floats[t_base + 1]]);
                } else {
                    tex_coords.push([0.0, 0.0]);
                }
            }

            unique_verts.insert(key, idx);
            idx
        };

        out_indices.push(unified_idx);
    }

    if positions.is_empty() {
        return None;
    }

    if normals.len() != positions.len() {
        tracing::warn!("[COLLADA_GEOM] Normals count {} != positions {} — generating flat normals",
            normals.len(), positions.len());
        normals = generate_flat_normals(&positions, &out_indices);
    }

    if tex_coords.len() != positions.len() {
        tex_coords = vec![[0.0, 0.0]; positions.len()];
    }

    Some(MeshFace {
        positions,
        normals,
        tex_coords,
        indices: out_indices,
        joint_weights: None,
        original_position_indices: Some(position_indices),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_triangles() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_geometries>
    <geometry id="Cube-mesh" name="Cube">
      <mesh>
        <source id="Cube-mesh-positions">
          <float_array id="Cube-mesh-positions-array" count="9">0 0 0 1 0 0 0 1 0</float_array>
          <technique_common>
            <accessor source="#Cube-mesh-positions-array" count="3" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Cube-mesh-normals">
          <float_array id="Cube-mesh-normals-array" count="3">0 0 1</float_array>
          <technique_common>
            <accessor source="#Cube-mesh-normals-array" count="1" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Cube-mesh-map-0">
          <float_array id="Cube-mesh-map-0-array" count="6">0 0 1 0 0 1</float_array>
          <technique_common>
            <accessor source="#Cube-mesh-map-0-array" count="3" stride="2">
              <param name="S" type="float"/>
              <param name="T" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <vertices id="Cube-mesh-vertices">
          <input semantic="POSITION" source="#Cube-mesh-positions"/>
        </vertices>
        <triangles count="1">
          <input semantic="VERTEX" source="#Cube-mesh-vertices" offset="0"/>
          <input semantic="NORMAL" source="#Cube-mesh-normals" offset="1"/>
          <input semantic="TEXCOORD" source="#Cube-mesh-map-0" offset="2" set="0"/>
          <p>0 0 0 1 0 1 2 0 2</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
</COLLADA>"##;

        let faces = parse_collada_geometry(dae).expect("parse failed");
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].positions.len(), 3);
        assert_eq!(faces[0].normals.len(), 3);
        assert_eq!(faces[0].tex_coords.len(), 3);
        assert_eq!(faces[0].indices.len(), 3);
        assert!((faces[0].positions[0][0]).abs() < 0.01);
        assert!((faces[0].positions[1][0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_polylist_quad() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_geometries>
    <geometry id="Plane-mesh">
      <mesh>
        <source id="Plane-mesh-positions">
          <float_array id="pos" count="12">0 0 0 1 0 0 1 1 0 0 1 0</float_array>
          <technique_common>
            <accessor source="#pos" count="4" stride="3"/>
          </technique_common>
        </source>
        <vertices id="Plane-mesh-vertices">
          <input semantic="POSITION" source="#Plane-mesh-positions"/>
        </vertices>
        <polylist count="1">
          <input semantic="VERTEX" source="#Plane-mesh-vertices" offset="0"/>
          <vcount>4</vcount>
          <p>0 1 2 3</p>
        </polylist>
      </mesh>
    </geometry>
  </library_geometries>
</COLLADA>"##;

        let faces = parse_collada_geometry(dae).expect("parse failed");
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].indices.len(), 6, "Quad should triangulate to 6 indices");
        assert_eq!(faces[0].positions.len(), 4);
    }

    #[test]
    fn test_parse_no_geometry() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_geometries/>
</COLLADA>"##;
        assert!(parse_collada_geometry(dae).is_err());
    }

    #[test]
    fn test_parse_shared_normal_index() {
        let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
  <library_geometries>
    <geometry id="Tri-mesh">
      <mesh>
        <source id="Tri-pos">
          <float_array count="18">0 0 0 1 0 0 0 1 0 0 0 1 1 0 1 0 1 1</float_array>
          <technique_common><accessor source="#x" count="6" stride="3"/></technique_common>
        </source>
        <source id="Tri-norm">
          <float_array count="6">0 0 1 0 0 -1</float_array>
          <technique_common><accessor source="#x" count="2" stride="3"/></technique_common>
        </source>
        <vertices id="Tri-verts">
          <input semantic="POSITION" source="#Tri-pos"/>
        </vertices>
        <triangles count="2">
          <input semantic="VERTEX" source="#Tri-verts" offset="0"/>
          <input semantic="NORMAL" source="#Tri-norm" offset="1"/>
          <p>0 0 1 0 2 0 3 1 4 1 5 1</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
</COLLADA>"##;

        let faces = parse_collada_geometry(dae).expect("parse failed");
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].indices.len(), 6);
    }
}
