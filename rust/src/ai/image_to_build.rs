use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::ai::ml_integration::llm_client::LocalLLMClient;
use crate::ai::npc_avatar::NPCAction;
use crate::ai::AIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorPlanData {
    pub walls: Vec<WallSegment>,
    pub doors: Vec<Opening>,
    pub windows: Vec<Opening>,
    pub rooms: Vec<Room>,
    pub overall_width: f32,
    pub overall_depth: f32,
    pub scale_meters_per_unit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallSegment {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub thickness: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opening {
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default = "default_sill")]
    pub sill_height: f32,
    pub wall_index: usize,
}

fn default_sill() -> f32 { 0.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: String,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationData {
    pub floors: Vec<FloorLevel>,
    pub roof_type: String,
    pub roof_height: f32,
    pub total_height: f32,
    pub facade_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorLevel {
    pub level: usize,
    pub height: f32,
    pub floor_height: f32,
    pub windows: Vec<Opening>,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub origin: [f32; 3],
    pub wall_height: f32,
    pub wall_thickness: f32,
    pub floor_thickness: f32,
    pub default_scale: f32,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            origin: [128.0, 128.0, 25.0],
            wall_height: 3.0,
            wall_thickness: 0.3,
            floor_thickness: 0.15,
            default_scale: 1.0,
        }
    }
}

const FLOORPLAN_VISION_PROMPT: &str = r#"You are an architectural floor plan analyzer. Analyze this floor plan image and extract the geometry as structured JSON.

Return ONLY a JSON object with this exact structure (no markdown, no explanation):
{
  "walls": [
    {"start_x": 0.0, "start_y": 0.0, "end_x": 10.0, "end_y": 0.0, "thickness": 0.3, "height": 3.0}
  ],
  "doors": [
    {"center_x": 5.0, "center_y": 0.0, "width": 1.0, "height": 2.1, "sill_height": 0.0, "wall_index": 0}
  ],
  "windows": [
    {"center_x": 2.5, "center_y": 0.0, "width": 1.2, "height": 1.4, "sill_height": 0.9, "wall_index": 0}
  ],
  "rooms": [
    {"name": "Living Room", "min_x": 0.0, "min_y": 0.0, "max_x": 5.0, "max_y": 4.0}
  ],
  "overall_width": 10.0,
  "overall_depth": 8.0,
  "scale_meters_per_unit": 1.0
}

Rules:
- All measurements in meters
- Coordinate origin is bottom-left corner of the plan
- X axis is left-right, Y axis is top-bottom (depth)
- Wall thickness defaults to 0.3m if not visible
- Wall height defaults to 3.0m unless annotated
- Door height defaults to 2.1m, width defaults to 0.9m
- Window sill_height is distance from floor to bottom of window
- wall_index refers to which wall (0-indexed) the opening is in
- If scale is shown (e.g., "1cm = 1m"), use it; otherwise estimate from standard room sizes (bedrooms ~3x4m, bathrooms ~2x2.5m, kitchens ~3x3m)
- Include ALL walls, even interior partition walls
- Return ONLY the JSON, no other text"#;

const ELEVATION_VISION_PROMPT: &str = r#"You are an architectural elevation analyzer. Analyze this building elevation/facade image and extract the geometry as structured JSON.

Return ONLY a JSON object with this exact structure (no markdown, no explanation):
{
  "floors": [
    {"level": 0, "height": 0.0, "floor_height": 3.0, "windows": [{"center_x": 2.0, "center_y": 1.5, "width": 1.2, "height": 1.4, "sill_height": 0.9, "wall_index": 0}]}
  ],
  "roof_type": "gabled",
  "roof_height": 2.0,
  "total_height": 8.0,
  "facade_width": 10.0
}

Rules:
- All measurements in meters
- level 0 = ground floor
- height = distance from ground to bottom of this floor
- floor_height = height of this floor (floor to ceiling)
- roof_type: "flat", "gabled", "hip", "shed", "mansard"
- roof_height = height of roof above top floor
- Window positions are relative to the floor they're on
- Return ONLY the JSON, no other text"#;

pub async fn analyze_floorplan(
    llm: &LocalLLMClient,
    image_data: &[u8],
    media_type: &str,
) -> Result<FloorPlanData, AIError> {
    info!("[IMAGE_TO_BUILD] Analyzing floor plan image ({} bytes, {})", image_data.len(), media_type);

    let response = llm.chat_with_image(
        FLOORPLAN_VISION_PROMPT,
        "Analyze this floor plan and extract all walls, doors, windows, and rooms as JSON.",
        image_data,
        media_type,
    ).await?;

    let json_text = extract_json(&response.text);
    info!("[IMAGE_TO_BUILD] Vision response ({}): {}", response.tokens_used, &json_text[..json_text.len().min(500)]);

    serde_json::from_str::<FloorPlanData>(&json_text)
        .map_err(|e| AIError::InferenceFailed(format!("Failed to parse floor plan JSON: {} — raw: {}", e, &json_text[..json_text.len().min(300)])))
}

pub async fn analyze_elevation(
    llm: &LocalLLMClient,
    image_data: &[u8],
    media_type: &str,
) -> Result<ElevationData, AIError> {
    info!("[IMAGE_TO_BUILD] Analyzing elevation image ({} bytes, {})", image_data.len(), media_type);

    let response = llm.chat_with_image(
        ELEVATION_VISION_PROMPT,
        "Analyze this building elevation and extract floor levels, windows, and roof geometry as JSON.",
        image_data,
        media_type,
    ).await?;

    let json_text = extract_json(&response.text);
    info!("[IMAGE_TO_BUILD] Elevation response ({}): {}", response.tokens_used, &json_text[..json_text.len().min(500)]);

    serde_json::from_str::<ElevationData>(&json_text)
        .map_err(|e| AIError::InferenceFailed(format!("Failed to parse elevation JSON: {} — raw: {}", e, &json_text[..json_text.len().min(300)])))
}

fn extract_json(text: &str) -> String {
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            let inner = after[..end].trim();
            if inner.starts_with('{') {
                return inner.to_string();
            }
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }
    text.trim().to_string()
}

pub fn floorplan_to_actions(plan: &FloorPlanData, config: &BuildConfig) -> Vec<NPCAction> {
    let mut actions: Vec<NPCAction> = Vec::new();
    let scale = config.default_scale * plan.scale_meters_per_unit;
    let ox = config.origin[0] - (plan.overall_width * scale / 2.0);
    let oy = config.origin[1] - (plan.overall_depth * scale / 2.0);
    let oz = config.origin[2];

    let floor_action = NPCAction::RezBox {
        position: [
            config.origin[0],
            config.origin[1],
            oz,
        ],
        scale: [
            plan.overall_width * scale,
            plan.overall_depth * scale,
            config.floor_thickness,
        ],
        name: "Floor".to_string(),
    };
    actions.push(floor_action);

    for (i, wall) in plan.walls.iter().enumerate() {
        let sx = ox + wall.start_x * scale;
        let sy = oy + wall.start_y * scale;
        let ex = ox + wall.end_x * scale;
        let ey = oy + wall.end_y * scale;

        let cx = (sx + ex) / 2.0;
        let cy = (sy + ey) / 2.0;
        let cz = oz + config.floor_thickness / 2.0 + config.wall_height / 2.0;

        let dx = ex - sx;
        let dy = ey - sy;
        let length = (dx * dx + dy * dy).sqrt();

        if length < 0.1 {
            continue;
        }

        let openings_in_wall: Vec<&Opening> = plan.doors.iter()
            .chain(plan.windows.iter())
            .filter(|o| o.wall_index == i)
            .collect();

        if openings_in_wall.is_empty() {
            let (wall_scale, rot) = wall_dimensions(length, wall.thickness * scale, config.wall_height, dx, dy);
            actions.push(NPCAction::RezBox {
                position: [cx, cy, cz],
                scale: wall_scale,
                name: format!("Wall_{}", i + 1),
            });
            if rot != [0.0, 0.0, 0.0, 1.0] {
                actions.push(NPCAction::SetRotation {
                    local_id: 0,
                    rotation: rot,
                });
            }
        } else {
            let mut sorted_openings: Vec<(&Opening, bool)> = plan.doors.iter()
                .filter(|o| o.wall_index == i)
                .map(|o| (o, true))
                .chain(plan.windows.iter().filter(|o| o.wall_index == i).map(|o| (o, false)))
                .collect();
            sorted_openings.sort_by(|a, b| {
                let a_pos = project_onto_wall(a.0.center_x, a.0.center_y, wall, scale, ox, oy);
                let b_pos = project_onto_wall(b.0.center_x, b.0.center_y, wall, scale, ox, oy);
                a_pos.partial_cmp(&b_pos).unwrap_or(std::cmp::Ordering::Equal)
            });

            let segments = split_wall_around_openings(wall, &sorted_openings, config, scale, ox, oy, oz, i);
            actions.extend(segments);
        }
    }

    actions
}

fn wall_dimensions(length: f32, thickness: f32, height: f32, dx: f32, dy: f32) -> ([f32; 3], [f32; 4]) {
    let angle = dy.atan2(dx);
    let is_ns = angle.abs() > std::f32::consts::FRAC_PI_4 && angle.abs() < 3.0 * std::f32::consts::FRAC_PI_4;

    if is_ns {
        ([thickness, length, height], [0.0, 0.0, 0.0, 1.0])
    } else {
        ([length, thickness, height], [0.0, 0.0, 0.0, 1.0])
    }
}

fn project_onto_wall(px: f32, py: f32, wall: &WallSegment, scale: f32, ox: f32, oy: f32) -> f32 {
    let wx = ox + wall.start_x * scale;
    let wy = oy + wall.start_y * scale;
    let dx = (wall.end_x - wall.start_x) * scale;
    let dy = (wall.end_y - wall.start_y) * scale;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.01 { return 0.0; }
    let px_w = px * scale + ox - wx;
    let py_w = py * scale + oy - wy;
    (px_w * dx + py_w * dy) / len
}

fn split_wall_around_openings(
    wall: &WallSegment,
    openings: &[(&Opening, bool)],
    config: &BuildConfig,
    scale: f32,
    ox: f32,
    oy: f32,
    oz: f32,
    wall_idx: usize,
) -> Vec<NPCAction> {
    let mut actions = Vec::new();
    let sx = ox + wall.start_x * scale;
    let sy = oy + wall.start_y * scale;
    let ex = ox + wall.end_x * scale;
    let ey = oy + wall.end_y * scale;
    let dx = ex - sx;
    let dy = ey - sy;
    let wall_len = (dx * dx + dy * dy).sqrt();
    if wall_len < 0.1 { return actions; }

    let dir_x = dx / wall_len;
    let dir_y = dy / wall_len;
    let thick = wall.thickness * scale;
    let cz_full = oz + config.floor_thickness / 2.0 + config.wall_height / 2.0;
    let seg_count = &mut 0usize;

    let mut cursor = 0.0f32;
    for (opening, is_door) in openings {
        let op_center = project_onto_wall(opening.center_x, opening.center_y, wall, scale, ox, oy);
        let op_half_w = opening.width * scale / 2.0;
        let op_start = (op_center - op_half_w).max(0.0);
        let op_end = (op_center + op_half_w).min(wall_len);

        if op_start > cursor + 0.05 {
            let seg_len = op_start - cursor;
            let seg_center = cursor + seg_len / 2.0;
            let cx = sx + dir_x * seg_center;
            let cy = sy + dir_y * seg_center;
            *seg_count += 1;
            let (wall_scale, _) = wall_dimensions(seg_len, thick, config.wall_height, dx, dy);
            actions.push(NPCAction::RezBox {
                position: [cx, cy, cz_full],
                scale: wall_scale,
                name: format!("Wall_{}_{}", wall_idx + 1, seg_count),
            });
        }

        if !is_door {
            let sill = opening.sill_height * scale;
            let win_h = opening.height * scale;
            let above_h = config.wall_height - sill - win_h;

            if sill > 0.05 {
                let cz_sill = oz + config.floor_thickness / 2.0 + sill / 2.0;
                let seg_cx = sx + dir_x * op_center;
                let seg_cy = sy + dir_y * op_center;
                *seg_count += 1;
                let (wall_scale, _) = wall_dimensions(opening.width * scale, thick, sill, dx, dy);
                actions.push(NPCAction::RezBox {
                    position: [seg_cx, seg_cy, cz_sill],
                    scale: wall_scale,
                    name: format!("Wall_{}_{}_sill", wall_idx + 1, seg_count),
                });
            }
            if above_h > 0.05 {
                let cz_above = oz + config.floor_thickness / 2.0 + sill + win_h + above_h / 2.0;
                let seg_cx = sx + dir_x * op_center;
                let seg_cy = sy + dir_y * op_center;
                *seg_count += 1;
                let (wall_scale, _) = wall_dimensions(opening.width * scale, thick, above_h, dx, dy);
                actions.push(NPCAction::RezBox {
                    position: [seg_cx, seg_cy, cz_above],
                    scale: wall_scale,
                    name: format!("Wall_{}_{}_above", wall_idx + 1, seg_count),
                });
            }
        } else {
            let door_h = opening.height * scale;
            let above_h = config.wall_height - door_h;
            if above_h > 0.05 {
                let cz_above = oz + config.floor_thickness / 2.0 + door_h + above_h / 2.0;
                let seg_cx = sx + dir_x * op_center;
                let seg_cy = sy + dir_y * op_center;
                *seg_count += 1;
                let (wall_scale, _) = wall_dimensions(opening.width * scale, thick, above_h, dx, dy);
                actions.push(NPCAction::RezBox {
                    position: [seg_cx, seg_cy, cz_above],
                    scale: wall_scale,
                    name: format!("Wall_{}_{}_transom", wall_idx + 1, seg_count),
                });
            }
        }

        cursor = op_end;
    }

    if cursor < wall_len - 0.05 {
        let seg_len = wall_len - cursor;
        let seg_center = cursor + seg_len / 2.0;
        let cx = sx + dir_x * seg_center;
        let cy = sy + dir_y * seg_center;
        *seg_count += 1;
        let (wall_scale, _) = wall_dimensions(seg_len, thick, config.wall_height, dx, dy);
        actions.push(NPCAction::RezBox {
            position: [cx, cy, cz_full],
            scale: wall_scale,
            name: format!("Wall_{}_{}", wall_idx + 1, seg_count),
        });
    }

    actions
}

pub fn elevation_to_actions(elevation: &ElevationData, plan: Option<&FloorPlanData>, config: &BuildConfig) -> Vec<NPCAction> {
    let mut actions = Vec::new();

    let width = if let Some(p) = plan {
        p.overall_width * config.default_scale * p.scale_meters_per_unit
    } else {
        elevation.facade_width
    };

    let oz = config.origin[2];

    for floor in &elevation.floors {
        let floor_z = oz + floor.height;
        for window in &floor.windows {
            let wx = config.origin[0] - width / 2.0 + window.center_x;
            let wz = floor_z + window.sill_height + window.height / 2.0;

            actions.push(NPCAction::RezBox {
                position: [wx, config.origin[1], wz],
                scale: [window.width, 0.05, window.height],
                name: format!("Window_F{}", floor.level),
            });
        }
    }

    match elevation.roof_type.as_str() {
        "gabled" => {
            let depth = if let Some(p) = plan {
                p.overall_depth * config.default_scale * p.scale_meters_per_unit
            } else {
                width * 0.6
            };
            let roof_base = oz + elevation.total_height - elevation.roof_height;
            actions.push(NPCAction::RezPrism {
                position: [config.origin[0], config.origin[1], roof_base + elevation.roof_height / 2.0],
                scale: [width + 0.4, depth + 0.4, elevation.roof_height],
                name: "Roof".to_string(),
            });
        }
        "flat" => {
            let depth = if let Some(p) = plan {
                p.overall_depth * config.default_scale * p.scale_meters_per_unit
            } else {
                width * 0.6
            };
            let roof_z = oz + elevation.total_height;
            actions.push(NPCAction::RezBox {
                position: [config.origin[0], config.origin[1], roof_z],
                scale: [width + 0.2, depth + 0.2, 0.15],
                name: "Roof".to_string(),
            });
        }
        _ => {}
    }

    actions
}

pub fn detect_media_type(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") { "image/png" }
    else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "image/jpeg" }
    else if lower.ends_with(".gif") { "image/gif" }
    else if lower.ends_with(".webp") { "image/webp" }
    else if lower.ends_with(".bmp") { "image/bmp" }
    else { "image/png" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_backticks() {
        let text = "Here's the analysis:\n```json\n{\"walls\": []}\n```\nDone.";
        assert_eq!(extract_json(text), "{\"walls\": []}");
    }

    #[test]
    fn test_extract_json_raw() {
        let text = "{\"walls\": [{\"start_x\": 0}]}";
        assert_eq!(extract_json(text), text);
    }

    #[test]
    fn test_extract_json_with_prefix() {
        let text = "The floor plan shows: {\"walls\": []}";
        assert_eq!(extract_json(text), "{\"walls\": []}");
    }

    #[test]
    fn test_wall_dimensions_ew() {
        let (scale, rot) = wall_dimensions(5.0, 0.3, 3.0, 5.0, 0.0);
        assert_eq!(scale, [5.0, 0.3, 3.0]);
        assert_eq!(rot, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_wall_dimensions_ns() {
        let (scale, rot) = wall_dimensions(5.0, 0.3, 3.0, 0.0, 5.0);
        assert_eq!(scale, [0.3, 5.0, 3.0]);
        assert_eq!(rot, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_floorplan_to_actions_simple_room() {
        let plan = FloorPlanData {
            walls: vec![
                WallSegment { start_x: 0.0, start_y: 0.0, end_x: 5.0, end_y: 0.0, thickness: 0.3, height: 3.0 },
                WallSegment { start_x: 5.0, start_y: 0.0, end_x: 5.0, end_y: 4.0, thickness: 0.3, height: 3.0 },
                WallSegment { start_x: 5.0, start_y: 4.0, end_x: 0.0, end_y: 4.0, thickness: 0.3, height: 3.0 },
                WallSegment { start_x: 0.0, start_y: 4.0, end_x: 0.0, end_y: 0.0, thickness: 0.3, height: 3.0 },
            ],
            doors: vec![],
            windows: vec![],
            rooms: vec![Room { name: "Room".to_string(), min_x: 0.0, min_y: 0.0, max_x: 5.0, max_y: 4.0 }],
            overall_width: 5.0,
            overall_depth: 4.0,
            scale_meters_per_unit: 1.0,
        };
        let config = BuildConfig::default();
        let actions = floorplan_to_actions(&plan, &config);
        assert!(!actions.is_empty());
        assert!(actions.len() >= 5);

        let rez_count = actions.iter().filter(|a| matches!(a, NPCAction::RezBox { .. })).count();
        assert!(rez_count >= 5);
    }

    #[test]
    fn test_detect_media_type() {
        assert_eq!(detect_media_type("plan.png"), "image/png");
        assert_eq!(detect_media_type("elevation.jpg"), "image/jpeg");
        assert_eq!(detect_media_type("PHOTO.JPEG"), "image/jpeg");
        assert_eq!(detect_media_type("sketch.webp"), "image/webp");
        assert_eq!(detect_media_type("unknown.xyz"), "image/png");
    }
}
