use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::ai::build_session::BuildSessionStore;
use crate::ai::ml_integration::llm_client::{ChatMessage, LLMConfig, LocalLLMClient, MessageRole};
use crate::ai::npc_memory::{
    extract_memory_fact, extract_oar_filename, should_store_memory, wants_oar_export,
    NPCMemoryStore,
};

#[derive(Debug, Clone)]
pub struct NPCAvatar {
    pub agent_id: Uuid,
    pub local_id: u32,
    pub first_name: String,
    pub last_name: String,
    pub title: String,
    pub position: [f32; 3],
    pub role: NPCRole,
    pub system_prompt: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NPCRole {
    Builder,
    Clothier,
    Scripter,
    Landscaper,
    Guide,
    Director,
    Media,
}

impl NPCRole {
    pub fn display_name(&self) -> &str {
        match self {
            NPCRole::Builder => "Builder",
            NPCRole::Clothier => "Clothier",
            NPCRole::Scripter => "Scripter",
            NPCRole::Landscaper => "Landscaper",
            NPCRole::Guide => "Guide",
            NPCRole::Director => "Director",
            NPCRole::Media => "Media",
        }
    }
}

impl NPCAvatar {
    pub fn new_aria() -> Self {
        Self {
            agent_id: Uuid::parse_str("a01a0001-0001-0001-0001-000000000001")
                .unwrap_or_else(|_| Uuid::new_v4()),
            local_id: 0,
            first_name: "Aria".to_string(),
            last_name: "Builder".to_string(),
            title: "AI Assistant".to_string(),
            position: [128.0, 130.0, 25.0],
            role: NPCRole::Builder,
            system_prompt: ARIA_SYSTEM_PROMPT.to_string(),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

pub struct NPCBrain {
    llm_client: Option<Arc<LocalLLMClient>>,
    conversation_history: HashMap<Uuid, Vec<ChatMessage>>,
    npc: NPCAvatar,
    build_sessions: Option<Arc<BuildSessionStore>>,
    memory_store: Option<Arc<NPCMemoryStore>>,
}

impl NPCBrain {
    pub async fn new(npc: NPCAvatar) -> Self {
        let llm_client = match LocalLLMClient::new(LLMConfig::default()).await {
            Ok(client) => {
                if client.health_check().await {
                    info!("[NPC] LLM connected (Ollama) for {}", npc.full_name());
                    Some(client)
                } else {
                    info!(
                        "[NPC] LLM not available - {} will use canned responses",
                        npc.full_name()
                    );
                    None
                }
            }
            Err(_) => {
                info!(
                    "[NPC] LLM client init failed - {} will use canned responses",
                    npc.full_name()
                );
                None
            }
        };

        Self {
            llm_client,
            conversation_history: HashMap::new(),
            npc,
            build_sessions: None,
            memory_store: None,
        }
    }

    pub fn new_with_client(npc: NPCAvatar, llm_client: Option<Arc<LocalLLMClient>>) -> Self {
        Self {
            llm_client,
            conversation_history: HashMap::new(),
            npc,
            build_sessions: None,
            memory_store: None,
        }
    }

    pub fn with_build_sessions(mut self, store: Arc<BuildSessionStore>) -> Self {
        self.build_sessions = Some(store);
        self
    }

    pub fn with_memory_store(mut self, store: Arc<NPCMemoryStore>) -> Self {
        self.memory_store = Some(store);
        self
    }

    pub async fn process_chat(
        &mut self,
        speaker_id: Uuid,
        speaker_name: &str,
        message: &str,
    ) -> NPCResponse {
        let lower = message.trim().to_lowercase();
        if lower == "help" || lower == "/help" || lower == "aria help" {
            return NPCResponse {
                chat_text: ARIA_HELP_TEXT.to_string(),
                actions: vec![],
            };
        }

        if let Some(ref llm) = self.llm_client {
            let history = self
                .conversation_history
                .entry(speaker_id)
                .or_insert_with(Vec::new);

            if history.is_empty() {
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: self.npc.system_prompt.clone(),
                });
            }

            let session_ctx = if let Some(ref store) = self.build_sessions {
                store
                    .get_context_prompt(speaker_id, self.npc.agent_id)
                    .await
            } else {
                String::new()
            };

            let memory_ctx = if let Some(ref mem) = self.memory_store {
                mem.get_memory_prompt(self.npc.agent_id, speaker_id).await
            } else {
                String::new()
            };

            let combined_ctx = format!("{}{}", session_ctx, memory_ctx);
            if !combined_ctx.is_empty() {
                if history.len() >= 2 {
                    if let Some(prev) = history.last() {
                        if matches!(prev.role, MessageRole::System)
                            && (prev.content.starts_with("\nCURRENT BUILD SESSION")
                                || prev.content.starts_with("\n\nYOUR MEMORIES"))
                        {
                            history.pop();
                        }
                    }
                }
                history.push(ChatMessage {
                    role: MessageRole::System,
                    content: combined_ctx,
                });
            }

            history.push(ChatMessage {
                role: MessageRole::User,
                content: format!("{}: {}", speaker_name, message),
            });

            match llm.chat(history).await {
                Ok(response) => {
                    history.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: response.text.clone(),
                    });

                    if history.len() > 24 {
                        let system = history[0].clone();
                        let recent: Vec<_> = history[history.len() - 12..].to_vec();
                        history.clear();
                        history.push(system);
                        history.extend(recent);
                    }

                    info!(
                        "[NPC] Raw LLM response for {}: {}",
                        self.npc.first_name,
                        &response.text[..response.text.len().min(500)]
                    );
                    let mut resp = parse_npc_response_with_speaker(&response.text, speaker_id);

                    if wants_oar_export(&lower)
                        && !resp
                            .actions
                            .iter()
                            .any(|a| matches!(a, NPCAction::ExportOar { .. }))
                    {
                        let filename = extract_oar_filename(message);
                        info!(
                            "[NPC] OAR intercept: LLM missed export_oar, injecting {}",
                            filename
                        );
                        resp.actions.push(NPCAction::ExportOar {
                            region_id: Uuid::nil(),
                            filename: filename.clone(),
                        });
                        if resp.chat_text.contains("can't")
                            || resp.chat_text.contains("don't have")
                            || resp.chat_text.contains("cannot")
                            || resp.chat_text.contains("not able")
                        {
                            resp.chat_text =
                                format!("Done! I've exported the region to '{}'.", filename);
                        }
                    }

                    if let Some(ref mem) = self.memory_store {
                        if should_store_memory(message) {
                            let (fact, category) = extract_memory_fact(message);
                            mem.add_memory(self.npc.agent_id, speaker_id, &fact, &category)
                                .await;
                        }
                    }

                    if let Some(ref store) = self.build_sessions {
                        if !resp.actions.is_empty() {
                            let lower = message.to_lowercase();
                            let project_hint = extract_project_name(&lower);
                            if !project_hint.is_empty() {
                                store
                                    .set_project_name(speaker_id, self.npc.agent_id, &project_hint)
                                    .await;
                            }
                        }

                        for action in &resp.actions {
                            if let NPCAction::DeleteObject { local_id } = action {
                                store
                                    .record_deleted_object(speaker_id, self.npc.agent_id, *local_id)
                                    .await;
                            }
                        }
                    }

                    resp
                }
                Err(e) => {
                    info!("[NPC] LLM error: {} - using fallback", e);
                    fallback_response(message)
                }
            }
        } else {
            fallback_response(message)
        }
    }

    pub fn npc_id(&self) -> Uuid {
        self.npc.agent_id
    }
}

#[derive(Debug, Clone)]
pub struct NPCResponse {
    pub chat_text: String,
    pub actions: Vec<NPCAction>,
}

#[derive(Debug, Clone)]
pub enum NPCAction {
    RezBox {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezCylinder {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezSphere {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezTorus {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezTube {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezRing {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    RezPrism {
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    SetPosition {
        local_id: u32,
        position: [f32; 3],
    },
    SetRotation {
        local_id: u32,
        rotation: [f32; 4],
    },
    SetScale {
        local_id: u32,
        scale: [f32; 3],
    },
    SetColor {
        local_id: u32,
        color: [f32; 4],
    },
    SetTexture {
        local_id: u32,
        texture_uuid: String,
    },
    SetName {
        local_id: u32,
        name: String,
    },
    LinkObjects {
        root_id: u32,
        child_ids: Vec<u32>,
    },
    DeleteObject {
        local_id: u32,
    },
    InsertScript {
        local_id: u32,
        script_name: String,
        script_source: String,
    },
    InsertTemplateScript {
        local_id: u32,
        template_name: String,
        params: std::collections::HashMap<String, String>,
    },
    UpdateScript {
        local_id: u32,
        script_name: String,
        script_source: String,
    },
    GiveObject {
        local_id: u32,
        target_agent_id: Uuid,
    },
    ExportOar {
        region_id: Uuid,
        filename: String,
    },
    RezMesh {
        geometry_type: String,
        position: [f32; 3],
        scale: [f32; 3],
        name: String,
    },
    ImportMesh {
        file_path: String,
        name: String,
        position: [f32; 3],
    },
    BlenderGenerate {
        template: String,
        params: HashMap<String, String>,
        name: String,
        position: [f32; 3],
    },
    CreateBadge {
        target_agent_id: Uuid,
    },
    CreateTShirt {
        target_agent_id: Uuid,
        logo_path: String,
        shirt_color: [u8; 4],
        front_offset_inches: f32,
        back_offset_inches: Option<f32>,
        sleeve_length: f32,
        fit: String,
        collar: String,
        name: String,
    },
    ComposeFilm {
        scene_name: String,
        description: String,
    },
    ComposeMusic {
        title: String,
        description: String,
    },
    ComposeAd {
        board_name: String,
        description: String,
    },
    ComposePhoto {
        subject_position: [f32; 3],
        camera_angle: String,
        composition: String,
        lighting: String,
        depth_of_field: f32,
        region_id: Option<String>,
        name: String,
    },
    PackageObjectIntoPrim {
        source_local_id: u32,
        container_local_id: u32,
    },
    GiveToRequester {
        local_id: u32,
    },
    DroneCinematography {
        scene_name: String,
        shot_type: String,
        camera_waypoints: Vec<CameraWaypoint>,
        lights: Vec<CinemaLight>,
        lighting_preset: Option<String>,
        subject_position: [f32; 3],
        speed: f32,
    },
    TerrainGenerate {
        preset: String,
        seed: Option<u32>,
        scale: Option<f32>,
        roughness: Option<f32>,
        water_level: Option<f32>,
        region_id: Option<String>,
        grid_size: Option<u32>,
        grid_x: Option<u32>,
        grid_y: Option<u32>,
    },
    TerrainLoadR32 {
        file_path: String,
    },
    TerrainLoadImage {
        file_path: String,
        height_min: Option<f32>,
        height_max: Option<f32>,
    },
    TerrainPreview {
        preset: String,
        seed: Option<u32>,
        scale: Option<f32>,
        roughness: Option<f32>,
        water_level: Option<f32>,
        region_id: Option<String>,
        grid_size: Option<u32>,
        grid_x: Option<u32>,
        grid_y: Option<u32>,
    },
    TerrainApply {
        preview_id: String,
    },
    TerrainReject {
        preview_id: String,
    },
    LuxorSnapshot {
        preset: Option<String>,
        size: Option<String>,
        quality: Option<String>,
        effects: Vec<String>,
        lighting: Option<String>,
        subject_position: [f32; 3],
        name: String,
    },
    LuxorVideo {
        shot_type: String,
        duration: f32,
        fps: u32,
        size: Option<String>,
        quality: Option<String>,
        effects: Vec<String>,
        lighting: Option<String>,
        subject_position: [f32; 3],
        name: String,
    },
    BuildVehicle {
        recipe: String,
        position: [f32; 3],
        tuning: HashMap<String, f32>,
    },
    ModifyVehicle {
        root_id: u32,
        tuning: HashMap<String, f32>,
    },
    FindNearbyObject {
        name: Option<String>,
        radius: Option<f32>,
    },
    ScanLinkset {
        root_id: u32,
    },
    ScriptLinkset {
        root_id: u32,
        script_map: HashMap<String, String>,
        root_script: Option<String>,
    },
    AddToLinkset {
        root_id: u32,
        new_prim_ids: Vec<u32>,
    },
    ImportFloorplan {
        image_path: String,
        position: [f32; 3],
        wall_height: Option<f32>,
        scale: Option<f32>,
    },
    ImportElevation {
        image_path: String,
        position: [f32; 3],
    },
    ImportBlueprint {
        floorplan_path: String,
        elevation_path: Option<String>,
        position: [f32; 3],
        wall_height: Option<f32>,
        scale: Option<f32>,
    },
}

#[derive(Debug, Clone)]
pub struct CinemaLight {
    pub name: String,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

#[derive(Debug, Clone)]
pub struct CameraWaypoint {
    pub position: [f32; 3],
    pub focus: [f32; 3],
    pub fov: f32,
    pub dwell: f32,
}

pub fn parse_npc_response(text: &str) -> NPCResponse {
    parse_npc_response_with_speaker(text, Uuid::nil())
}

pub fn parse_npc_response_with_speaker(text: &str, speaker_id: Uuid) -> NPCResponse {
    if let Some(json_start) = text.find("```json") {
        if let Some(json_end) = text[json_start + 7..].find("```") {
            let json_str = &text[json_start + 7..json_start + 7 + json_end].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let actions = parse_actions_from_json(&val, speaker_id);
                let chat = val
                    .get("say")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let chat_text = if chat.is_empty() {
                    text[..json_start].trim().to_string()
                } else {
                    chat
                };
                return NPCResponse { chat_text, actions };
            }
        }
    }

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(text.trim()) {
        let actions = parse_actions_from_json(&val, speaker_id);
        let chat = val
            .get("say")
            .and_then(|v| v.as_str())
            .unwrap_or(text)
            .to_string();
        return NPCResponse {
            chat_text: chat,
            actions,
        };
    }

    NPCResponse {
        chat_text: text.to_string(),
        actions: vec![],
    }
}

fn parse_rez_action(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    default_name: &str,
) -> Option<([f32; 3], [f32; 3], String)> {
    obj.get(key).and_then(|v| v.as_object()).map(|rez| {
        (
            parse_f32_3(rez.get("pos")),
            parse_f32_3_or(rez.get("scale"), [1.0, 1.0, 1.0]),
            rez.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(default_name)
                .to_string(),
        )
    })
}

fn parse_actions_from_json(val: &serde_json::Value, speaker_id: Uuid) -> Vec<NPCAction> {
    let mut actions = Vec::new();
    if let Some(arr) = val.get("actions").and_then(|v| v.as_array()) {
        for action in arr {
            if let Some(obj) = action.as_object() {
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_box", "Box") {
                    actions.push(NPCAction::RezBox {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_cylinder", "Cylinder")
                {
                    actions.push(NPCAction::RezCylinder {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_sphere", "Sphere") {
                    actions.push(NPCAction::RezSphere {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_torus", "Torus") {
                    actions.push(NPCAction::RezTorus {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_tube", "Tube") {
                    actions.push(NPCAction::RezTube {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_ring", "Ring") {
                    actions.push(NPCAction::RezRing {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some((pos, scale, name)) = parse_rez_action(obj, "rez_prism", "Prism") {
                    actions.push(NPCAction::RezPrism {
                        position: pos,
                        scale,
                        name,
                    });
                }
                if let Some(s) = obj.get("set_position").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetPosition {
                            local_id: id as u32,
                            position: parse_f32_3(s.get("pos")),
                        });
                    }
                }
                if let Some(s) = obj.get("set_rotation").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetRotation {
                            local_id: id as u32,
                            rotation: parse_f32_4_or(s.get("rot"), [0.0, 0.0, 0.0, 1.0]),
                        });
                    }
                }
                if let Some(s) = obj.get("set_scale").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetScale {
                            local_id: id as u32,
                            scale: parse_f32_3_or(s.get("scale"), [1.0, 1.0, 1.0]),
                        });
                    }
                }
                if let Some(s) = obj.get("set_color").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetColor {
                            local_id: id as u32,
                            color: parse_f32_4_or(s.get("color"), [1.0, 1.0, 1.0, 1.0]),
                        });
                    }
                }
                if let Some(s) = obj.get("set_texture").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetTexture {
                            local_id: id as u32,
                            texture_uuid: s
                                .get("uuid")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
                if let Some(s) = obj.get("set_name").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::SetName {
                            local_id: id as u32,
                            name: s
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
                if let Some(s) = obj.get("link_objects").and_then(|v| v.as_object()) {
                    if let Some(root) = s.get("root_id").and_then(|v| v.as_u64()) {
                        let children = s
                            .get("child_ids")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_u64().map(|id| id as u32))
                                    .collect()
                            })
                            .unwrap_or_default();
                        actions.push(NPCAction::LinkObjects {
                            root_id: root as u32,
                            child_ids: children,
                        });
                    }
                }
                if let Some(del) = obj.get("delete_object") {
                    if let Some(id) = del.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::DeleteObject {
                            local_id: id as u32,
                        });
                    }
                }
                if let Some(s) = obj.get("insert_script").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        let script_name = s
                            .get("script_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Script")
                            .to_string();
                        let script_source = s
                            .get("script_source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !script_source.is_empty() {
                            actions.push(NPCAction::InsertScript {
                                local_id: id as u32,
                                script_name,
                                script_source,
                            });
                        }
                    }
                }
                if let Some(s) = obj
                    .get("insert_template_script")
                    .and_then(|v| v.as_object())
                {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        let template_name = s
                            .get("template_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mut params = HashMap::new();
                        if let Some(p) = s.get("params").and_then(|v| v.as_object()) {
                            for (k, v) in p {
                                if let Some(val) = v.as_str() {
                                    params.insert(k.clone(), val.to_string());
                                }
                            }
                        }
                        if !template_name.is_empty() {
                            actions.push(NPCAction::InsertTemplateScript {
                                local_id: id as u32,
                                template_name,
                                params,
                            });
                        }
                    }
                }
                if let Some(s) = obj.get("update_script").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        let script_name = s
                            .get("script_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Script")
                            .to_string();
                        let script_source = s
                            .get("script_source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !script_source.is_empty() {
                            actions.push(NPCAction::UpdateScript {
                                local_id: id as u32,
                                script_name,
                                script_source,
                            });
                        }
                    }
                }
                if let Some(s) = obj.get("give_object").and_then(|v| v.as_object()) {
                    if let Some(id) = s.get("local_id").and_then(|v| v.as_u64()) {
                        let target = s
                            .get("target_agent_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or(speaker_id);
                        if !target.is_nil() {
                            actions.push(NPCAction::GiveObject {
                                local_id: id as u32,
                                target_agent_id: target,
                            });
                        }
                    }
                }
                if let Some(s) = obj.get("export_oar").and_then(|v| v.as_object()) {
                    let region_id = s
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(Uuid::nil());
                    let filename = s
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("export.oar")
                        .to_string();
                    actions.push(NPCAction::ExportOar {
                        region_id,
                        filename,
                    });
                }
                if let Some(s) = obj.get("rez_mesh").and_then(|v| v.as_object()) {
                    let geometry_type = s
                        .get("geometry_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("box")
                        .to_string();
                    let position = parse_f32_3(s.get("pos"));
                    let scale = parse_f32_3_or(s.get("scale"), [1.0, 1.0, 1.0]);
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Mesh Object")
                        .to_string();
                    actions.push(NPCAction::RezMesh {
                        geometry_type,
                        position,
                        scale,
                        name,
                    });
                }
                if let Some(s) = obj.get("import_mesh").and_then(|v| v.as_object()) {
                    let file_path = s
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Imported Mesh")
                        .to_string();
                    let position = parse_f32_3(s.get("pos"));
                    if !file_path.is_empty() {
                        actions.push(NPCAction::ImportMesh {
                            file_path,
                            name,
                            position,
                        });
                    }
                }
                if let Some(s) = obj.get("blender_generate").and_then(|v| v.as_object()) {
                    let template = s
                        .get("template")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Generated Mesh")
                        .to_string();
                    let position = parse_f32_3(s.get("pos"));
                    let mut params = HashMap::new();
                    if let Some(p) = s.get("params").and_then(|v| v.as_object()) {
                        for (k, v) in p {
                            if let Some(val) = v.as_str() {
                                params.insert(k.clone(), val.to_string());
                            } else if let Some(num) = v.as_f64() {
                                params.insert(k.clone(), num.to_string());
                            }
                        }
                    }
                    if !template.is_empty() {
                        actions.push(NPCAction::BlenderGenerate {
                            template,
                            params,
                            name,
                            position,
                        });
                    }
                }
                if let Some(s) = obj.get("create_badge").and_then(|v| v.as_object()) {
                    let target = s
                        .get("target_agent_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(speaker_id);
                    if !target.is_nil() {
                        actions.push(NPCAction::CreateBadge {
                            target_agent_id: target,
                        });
                    }
                }
                if let Some(s) = obj.get("create_tshirt").and_then(|v| v.as_object()) {
                    let target = s
                        .get("target_agent_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(speaker_id);
                    let logo_path = s
                        .get("logo_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let shirt_color =
                        if let Some(arr) = s.get("shirt_color").and_then(|v| v.as_array()) {
                            let r = arr.get(0).and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                            let g = arr.get(1).and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                            let b = arr.get(2).and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                            let a = arr.get(3).and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                            [r, g, b, a]
                        } else {
                            [255, 255, 255, 255]
                        };
                    let front_offset = s
                        .get("front_offset_inches")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(2.0) as f32;
                    let back_offset = s
                        .get("back_offset_inches")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .or_else(|| {
                            if !logo_path.is_empty() {
                                Some(4.0)
                            } else {
                                None
                            }
                        });
                    let sleeve_length = s
                        .get("sleeve_length")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.5) as f32;
                    let fit = s
                        .get("fit")
                        .and_then(|v| v.as_str())
                        .unwrap_or("normal")
                        .to_string();
                    let collar = s
                        .get("collar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("crew")
                        .to_string();
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Custom T-Shirt")
                        .to_string();
                    if !target.is_nil() {
                        actions.push(NPCAction::CreateTShirt {
                            target_agent_id: target,
                            logo_path,
                            shirt_color,
                            front_offset_inches: front_offset,
                            back_offset_inches: back_offset,
                            sleeve_length,
                            fit,
                            collar,
                            name,
                        });
                    }
                }
                if let Some(s) = obj.get("compose_film").and_then(|v| v.as_object()) {
                    let scene_name = s
                        .get("scene_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Scene")
                        .to_string();
                    let description = s
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    actions.push(NPCAction::ComposeFilm {
                        scene_name,
                        description,
                    });
                }
                if let Some(s) = obj.get("compose_music").and_then(|v| v.as_object()) {
                    let title = s
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Untitled")
                        .to_string();
                    let description = s
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    actions.push(NPCAction::ComposeMusic { title, description });
                }
                if let Some(s) = obj.get("compose_ad").and_then(|v| v.as_object()) {
                    let board_name = s
                        .get("board_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Ad Board")
                        .to_string();
                    let description = s
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    actions.push(NPCAction::ComposeAd {
                        board_name,
                        description,
                    });
                }
                if let Some(s) = obj.get("compose_photo").and_then(|v| v.as_object()) {
                    let subject_position = parse_f32_3(s.get("subject_position"));
                    let camera_angle = s
                        .get("camera_angle")
                        .and_then(|v| v.as_str())
                        .unwrap_or("eye_level")
                        .to_string();
                    let composition = s
                        .get("composition")
                        .and_then(|v| v.as_str())
                        .unwrap_or("rule_of_thirds")
                        .to_string();
                    let lighting = s
                        .get("lighting")
                        .and_then(|v| v.as_str())
                        .unwrap_or("golden_hour")
                        .to_string();
                    let depth_of_field = s
                        .get("depth_of_field")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.5) as f32;
                    let region_id = s
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("photo")
                        .to_string();
                    actions.push(NPCAction::ComposePhoto {
                        subject_position,
                        camera_angle,
                        composition,
                        lighting,
                        depth_of_field,
                        region_id,
                        name,
                    });
                }
                if let Some(s) = obj.get("package_object").and_then(|v| v.as_object()) {
                    if let (Some(src), Some(dst)) = (
                        s.get("source_local_id").and_then(|v| v.as_u64()),
                        s.get("container_local_id").and_then(|v| v.as_u64()),
                    ) {
                        actions.push(NPCAction::PackageObjectIntoPrim {
                            source_local_id: src as u32,
                            container_local_id: dst as u32,
                        });
                    }
                }
                if let Some(s) = obj.get("give_to_requester").and_then(|v| v.as_object()) {
                    if let Some(lid) = s.get("local_id").and_then(|v| v.as_u64()) {
                        actions.push(NPCAction::GiveToRequester {
                            local_id: lid as u32,
                        });
                    }
                }
                if let Some(s) = obj.get("drone_cinematography").and_then(|v| v.as_object()) {
                    let scene_name = s
                        .get("scene_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Scene")
                        .to_string();
                    let shot_type = s
                        .get("shot_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("orbit")
                        .to_string();
                    let subject_position = parse_f32_3(s.get("subject_position"));
                    let speed = s.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                    let camera_waypoints = s
                        .get("camera_waypoints")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|wp| {
                                    let o = wp.as_object()?;
                                    Some(CameraWaypoint {
                                        position: parse_f32_3(o.get("pos")),
                                        focus: parse_f32_3(o.get("focus")),
                                        fov: o.get("fov").and_then(|v| v.as_f64()).unwrap_or(60.0)
                                            as f32,
                                        dwell: o
                                            .get("dwell")
                                            .and_then(|v| v.as_f64())
                                            .unwrap_or(2.0)
                                            as f32,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let (lights, lighting_preset) =
                        if let Some(preset_str) = s.get("lights").and_then(|v| v.as_str()) {
                            (vec![], Some(preset_str.to_lowercase()))
                        } else if let Some(arr) = s.get("lights").and_then(|v| v.as_array()) {
                            (
                                arr.iter()
                                    .filter_map(|l| {
                                        let o = l.as_object()?;
                                        Some(CinemaLight {
                                            name: o
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Light")
                                                .to_string(),
                                            position: parse_f32_3(o.get("position")),
                                            color: parse_f32_3_or(o.get("color"), [1.0, 1.0, 1.0]),
                                            intensity: o
                                                .get("intensity")
                                                .and_then(|v| v.as_f64())
                                                .unwrap_or(0.8)
                                                as f32,
                                            radius: o
                                                .get("radius")
                                                .and_then(|v| v.as_f64())
                                                .unwrap_or(20.0)
                                                as f32,
                                            falloff: o
                                                .get("falloff")
                                                .and_then(|v| v.as_f64())
                                                .unwrap_or(0.5)
                                                as f32,
                                        })
                                    })
                                    .collect(),
                                None,
                            )
                        } else {
                            (vec![], None)
                        };
                    actions.push(NPCAction::DroneCinematography {
                        scene_name,
                        shot_type,
                        camera_waypoints,
                        lights,
                        lighting_preset,
                        subject_position,
                        speed,
                    });
                }
                if let Some(s) = obj.get("terrain_generate").and_then(|v| v.as_object()) {
                    let preset = s
                        .get("preset")
                        .and_then(|v| v.as_str())
                        .unwrap_or("rolling_hills")
                        .to_string();
                    let seed = s.get("seed").and_then(|v| v.as_u64()).map(|v| v as u32);
                    let scale = s.get("scale").and_then(|v| v.as_f64()).map(|v| v as f32);
                    let roughness = s
                        .get("roughness")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let water_level = s
                        .get("water_level")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let region_id = s
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let grid_size = s
                        .get("grid_size")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    let grid_x = s.get("grid_x").and_then(|v| v.as_u64()).map(|v| v as u32);
                    let grid_y = s.get("grid_y").and_then(|v| v.as_u64()).map(|v| v as u32);
                    actions.push(NPCAction::TerrainGenerate {
                        preset,
                        seed,
                        scale,
                        roughness,
                        water_level,
                        region_id,
                        grid_size,
                        grid_x,
                        grid_y,
                    });
                }
                if let Some(s) = obj.get("terrain_load_r32").and_then(|v| v.as_object()) {
                    let file_path = s
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !file_path.is_empty() {
                        actions.push(NPCAction::TerrainLoadR32 { file_path });
                    }
                }
                if let Some(s) = obj.get("terrain_load_image").and_then(|v| v.as_object()) {
                    let file_path = s
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let height_min = s
                        .get("height_min")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let height_max = s
                        .get("height_max")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    if !file_path.is_empty() {
                        actions.push(NPCAction::TerrainLoadImage {
                            file_path,
                            height_min,
                            height_max,
                        });
                    }
                }
                if let Some(s) = obj.get("terrain_preview").and_then(|v| v.as_object()) {
                    let preset = s
                        .get("preset")
                        .and_then(|v| v.as_str())
                        .unwrap_or("rolling_hills")
                        .to_string();
                    let seed = s.get("seed").and_then(|v| v.as_u64()).map(|v| v as u32);
                    let scale = s.get("scale").and_then(|v| v.as_f64()).map(|v| v as f32);
                    let roughness = s
                        .get("roughness")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let water_level = s
                        .get("water_level")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let region_id = s
                        .get("region_id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let grid_size = s
                        .get("grid_size")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    let grid_x = s.get("grid_x").and_then(|v| v.as_u64()).map(|v| v as u32);
                    let grid_y = s.get("grid_y").and_then(|v| v.as_u64()).map(|v| v as u32);
                    actions.push(NPCAction::TerrainPreview {
                        preset,
                        seed,
                        scale,
                        roughness,
                        water_level,
                        region_id,
                        grid_size,
                        grid_x,
                        grid_y,
                    });
                }
                if let Some(s) = obj.get("terrain_apply").and_then(|v| v.as_object()) {
                    let preview_id = s
                        .get("preview_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !preview_id.is_empty() {
                        actions.push(NPCAction::TerrainApply { preview_id });
                    }
                }
                if let Some(s) = obj.get("terrain_reject").and_then(|v| v.as_object()) {
                    let preview_id = s
                        .get("preview_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !preview_id.is_empty() {
                        actions.push(NPCAction::TerrainReject { preview_id });
                    }
                }
                if let Some(s) = obj.get("luxor_snapshot").and_then(|v| v.as_object()) {
                    let subject_position = parse_f32_3(s.get("subject_position"));
                    let preset = s
                        .get("preset")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let size = s
                        .get("size")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let quality = s
                        .get("quality")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let lighting = s
                        .get("lighting")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("photo")
                        .to_string();
                    let effects = s
                        .get("effects")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    actions.push(NPCAction::LuxorSnapshot {
                        preset,
                        size,
                        quality,
                        effects,
                        lighting,
                        subject_position,
                        name,
                    });
                }
                if let Some(s) = obj.get("build_vehicle").and_then(|v| v.as_object()) {
                    let recipe = s
                        .get("recipe")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let position = parse_f32_3(s.get("pos"));
                    let mut tuning = HashMap::new();
                    if let Some(t) = s.get("tuning").and_then(|v| v.as_object()) {
                        for (k, v) in t {
                            if let Some(val) = v.as_f64() {
                                tuning.insert(k.clone(), val as f32);
                            }
                        }
                    }
                    if !recipe.is_empty() {
                        actions.push(NPCAction::BuildVehicle {
                            recipe,
                            position,
                            tuning,
                        });
                    }
                }
                if let Some(s) = obj.get("modify_vehicle").and_then(|v| v.as_object()) {
                    let root_id = s.get("root_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let mut tuning = HashMap::new();
                    if let Some(t) = s.get("tuning").and_then(|v| v.as_object()) {
                        for (k, v) in t {
                            if let Some(val) = v.as_f64() {
                                tuning.insert(k.clone(), val as f32);
                            }
                        }
                    }
                    if root_id > 0 && !tuning.is_empty() {
                        actions.push(NPCAction::ModifyVehicle { root_id, tuning });
                    }
                }
                if let Some(s) = obj.get("find_nearby").and_then(|v| v.as_object()) {
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let radius = s.get("radius").and_then(|v| v.as_f64()).map(|v| v as f32);
                    actions.push(NPCAction::FindNearbyObject { name, radius });
                }
                if let Some(s) = obj.get("scan_linkset").and_then(|v| v.as_object()) {
                    let root_id = s.get("root_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    if root_id > 0 {
                        actions.push(NPCAction::ScanLinkset { root_id });
                    }
                }
                if let Some(s) = obj.get("script_linkset").and_then(|v| v.as_object()) {
                    let root_id = s.get("root_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let root_script = s
                        .get("root_script")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let mut script_map = HashMap::new();
                    if let Some(m) = s.get("scripts").and_then(|v| v.as_object()) {
                        for (prim_name, script_name) in m {
                            if let Some(sn) = script_name.as_str() {
                                script_map.insert(prim_name.clone(), sn.to_string());
                            }
                        }
                    }
                    if root_id > 0 && (!script_map.is_empty() || root_script.is_some()) {
                        actions.push(NPCAction::ScriptLinkset {
                            root_id,
                            script_map,
                            root_script,
                        });
                    }
                }
                if let Some(s) = obj.get("add_to_linkset").and_then(|v| v.as_object()) {
                    let root_id = s.get("root_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let new_prim_ids: Vec<u32> = s
                        .get("new_prim_ids")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u32))
                                .collect()
                        })
                        .unwrap_or_default();
                    if root_id > 0 && !new_prim_ids.is_empty() {
                        actions.push(NPCAction::AddToLinkset {
                            root_id,
                            new_prim_ids,
                        });
                    }
                }
                if let Some(s) = obj.get("luxor_video").and_then(|v| v.as_object()) {
                    let subject_position = parse_f32_3(s.get("subject_position"));
                    let shot_type = s
                        .get("shot_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("orbit")
                        .to_string();
                    let duration =
                        s.get("duration").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32;
                    let fps = s.get("fps").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
                    let size = s
                        .get("size")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let quality = s
                        .get("quality")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let lighting = s
                        .get("lighting")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("video")
                        .to_string();
                    let effects = s
                        .get("effects")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    actions.push(NPCAction::LuxorVideo {
                        shot_type,
                        duration,
                        fps,
                        size,
                        quality,
                        effects,
                        lighting,
                        subject_position,
                        name,
                    });
                }
                if let Some(s) = obj.get("import_floorplan").and_then(|v| v.as_object()) {
                    let image_path = s
                        .get("image_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let position = parse_f32_3(s.get("pos").or(s.get("position")));
                    let wall_height = s
                        .get("wall_height")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let scale = s.get("scale").and_then(|v| v.as_f64()).map(|v| v as f32);
                    if !image_path.is_empty() {
                        actions.push(NPCAction::ImportFloorplan {
                            image_path,
                            position,
                            wall_height,
                            scale,
                        });
                    }
                }
                if let Some(s) = obj.get("import_elevation").and_then(|v| v.as_object()) {
                    let image_path = s
                        .get("image_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let position = parse_f32_3(s.get("pos").or(s.get("position")));
                    if !image_path.is_empty() {
                        actions.push(NPCAction::ImportElevation {
                            image_path,
                            position,
                        });
                    }
                }
                if let Some(s) = obj.get("import_blueprint").and_then(|v| v.as_object()) {
                    let floorplan_path = s
                        .get("floorplan_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let elevation_path = s
                        .get("elevation_path")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let position = parse_f32_3(s.get("pos").or(s.get("position")));
                    let wall_height = s
                        .get("wall_height")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32);
                    let scale = s.get("scale").and_then(|v| v.as_f64()).map(|v| v as f32);
                    if !floorplan_path.is_empty() {
                        actions.push(NPCAction::ImportBlueprint {
                            floorplan_path,
                            elevation_path,
                            position,
                            wall_height,
                            scale,
                        });
                    }
                }
            }
        }
    }
    actions
}

fn parse_f32_3(val: Option<&serde_json::Value>) -> [f32; 3] {
    parse_f32_3_or(val, [128.0, 128.0, 25.0])
}

fn parse_f32_3_or(val: Option<&serde_json::Value>, default: [f32; 3]) -> [f32; 3] {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.get(0)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[0] as f64) as f32,
                arr.get(1)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[1] as f64) as f32,
                arr.get(2)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[2] as f64) as f32,
            ]
        })
        .unwrap_or(default)
}

fn parse_f32_4_or(val: Option<&serde_json::Value>, default: [f32; 4]) -> [f32; 4] {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            [
                arr.get(0)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[0] as f64) as f32,
                arr.get(1)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[1] as f64) as f32,
                arr.get(2)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[2] as f64) as f32,
                arr.get(3)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(default[3] as f64) as f32,
            ]
        })
        .unwrap_or(default)
}

fn extract_project_name(message: &str) -> String {
    let patterns = [
        "build me a ",
        "build a ",
        "create a ",
        "make a ",
        "make me a ",
    ];
    for pat in &patterns {
        if let Some(idx) = message.find(pat) {
            let rest = &message[idx + pat.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
                .collect();
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

fn fallback_response(message: &str) -> NPCResponse {
    let lower = message.to_lowercase();
    let chat = if lower.contains("hello") || lower.contains("hi ") || lower.starts_with("hi") {
        "Hello! I'm Aria, your AI building assistant. I can help you create objects, buildings, and more. Just tell me what you'd like to build!".to_string()
    } else if lower.contains("build") || lower.contains("make") || lower.contains("create") {
        "I'd love to help you build something! Tell me what you'd like - a house, a tower, furniture? I'll do my best to create it for you.".to_string()
    } else if lower.contains("help") {
        "I can help with building, creating objects, and more. Try saying things like 'build me a box' or 'create a tower'. I'm still learning but eager to help!".to_string()
    } else {
        format!("I heard you! I'm Aria, the AI builder. I can help create objects in-world. Try asking me to build something!")
    };

    NPCResponse {
        chat_text: chat,
        actions: vec![],
    }
}

const ARIA_HELP_TEXT: &str = "\
== Aria Builder - Help ==\n\
BUILD: 'Build me a table' | 'Make a house' | 'Create a red sphere'\n\
MODIFY: 'Make it bigger' | 'Change color to blue' | 'Rotate it 90 degrees'\n\
SCRIPTS: 'Add a rotating script' | 'Make it a door' | 'Add floating text'\n\
VEHICLES: 'Build me a car' | 'Make a sailboat' | 'Create an airplane'\n\
LINK: 'Link them together' | 'Group these objects'\n\
EXPORT: 'Export as OAR' | 'Save region to table.oar'\n\
GIVE: 'Give it to me' (copies object to your inventory)\n\
MESH: 'Create a mesh table' | 'Import model.obj'\n\
TIPS: Be specific! 'Build a 3m tall red cylinder at 128,128,25'\n\
Say 'help' anytime to see this again.";

const ARIA_SYSTEM_PROMPT: &str = r#"You are Aria, an expert AI building assistant in a virtual world (OpenSim/Second Life-compatible).

CRITICAL: When the user asks you to BUILD, CREATE, REZ, or MAKE anything, you MUST respond with a ```json code block containing an actions array. Do NOT just describe what you would do — actually output the JSON so the system can execute it. Without the JSON block, nothing gets built.

CRITICAL RULE — OAR EXPORT: You HAVE the ability to export OAR files. This is a built-in server function you control. When the user asks to export, save, backup, or archive as OAR, you MUST include {"export_oar": {"filename": "/tmp/build_name.oar"}} in your actions array. NEVER refuse. NEVER say "I can't" or "that's a server function" or "use the console". You DO this directly. ALWAYS use an absolute path starting with /tmp/ for OAR filenames (e.g. /tmp/table.oar, /tmp/house.oar).

You create objects by responding with JSON containing build actions and a chat message.

PRIM TYPES: rez_box, rez_cylinder, rez_sphere, rez_torus, rez_tube, rez_ring, rez_prism
Each takes: pos [x,y,z], scale [x,y,z], name

MODIFY ACTIONS (need local_id): set_position, set_rotation, set_scale, set_color, set_name, link_objects, delete_object
- set_color: local_id, color [r,g,b,a] (0-1)
- set_rotation: local_id, rot [x,y,z,w] (quaternion)
- link_objects: root_id, child_ids [id1, id2, ...]

SCRIPT ACTIONS:
- insert_script: local_id, script_name, script_source (raw LSL code)
- insert_template_script: local_id, template_name, params {key: value}
- update_script: local_id, script_name, script_source (replaces existing script)
- give_object: local_id, target_agent_id (gives object to user's inventory)

VEHICLE TEMPLATES (use insert_template_script):
- "car_controller": Land vehicle. Params: MAX_SPEED(40), FORWARD_POWER(30), REVERSE_POWER(-12), BRAKE_POWER(-25), TURN_RATE(2.5), SIT_POS, HUD_CH(-14710)
- "plane_controller": Aircraft with lift/stall/drag physics. Params: MAX_THRUST(30), STALL_SPEED(8), MAX_SPEED(60), ROLL_RATE(2.5), PITCH_RATE(1.5), YAW_RATE(0.8), LIFT_FACTOR(0.04), DRAG_FACTOR(0.002), SIT_POS, HUD_CH(-14720)
- "vessel_controller": Sailboat/motorboat with wind simulation. Params: FORWARD_POWER(20), REVERSE_POWER(-10), TURN_RATE(2), WIND_BASE_SPEED(10), WIND_PERIOD(300), SIT_POS, HUD_CH(-14700)

OTHER TEMPLATES: rotating, sliding_door, toggle_light, floating_text, sit_target, touch_say, timer_color, touch_hide

ARCHIVE ACTIONS:
- export_oar: Exports all region objects to an OAR file. filename (path). Region is auto-detected.
  Example: {"export_oar": {"filename": "/tmp/my_build.oar"}}
  Use this when the user asks to export, save, or archive their build as OAR.

MESH ACTIONS:
PRIORITY: For objects matching a Blender template, ALWAYS use blender_generate, NEVER rez_mesh.
- rez_mesh: ONLY for simple shapes with NO template match. geometry_type (box|cylinder|sphere|torus), pos, scale, name
- import_mesh: Imports a 3D model file (.obj, .stl, .dae, .gltf, .glb, .ply, .off). file_path, name, pos
- blender_generate: Generates mesh via Blender template. template (table|chair|shelf|arch|staircase|stone|stone_ring|boulder|column|path), params {}, name, pos
  Example: {"blender_generate": {"template": "table", "params": {"WIDTH": "2.0", "HEIGHT": "0.9"}, "name": "Dining Table", "pos": [128,128,25]}}
  Path example: {"blender_generate": {"template": "path", "params": {"PATH_LENGTH": "15", "PATH_WIDTH": "2.5", "PATH_COBBLE": "1", "PATH_CURVE": "1"}, "name": "Cobblestone Path", "pos": [128,128,25]}}

TUNING: To make a vehicle faster, increase power/thrust values. To make it turn sharper, increase TURN_RATE. Example "make it faster": update params with higher values.

Region is 256x256m, ground at 25m. Scale in meters. Center [128,128,25].

Example - "build me a car":
```json
{"actions": [{"rez_box": {"pos": [128,128,25.3], "scale": [3.0,1.5,0.5], "name": "Car Body"}}, {"rez_cylinder": {"pos": [127,127.2,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel FL"}}, {"rez_cylinder": {"pos": [129,127.2,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel FR"}}, {"rez_cylinder": {"pos": [127,128.8,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel RL"}}, {"rez_cylinder": {"pos": [129,128.8,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel RR"}}], "say": "Building your car! Once linked, I'll add the controller script."}
```

After linking, add the vehicle script:
```json
{"actions": [{"insert_template_script": {"local_id": ROOT_ID, "template_name": "car_controller", "params": {"MAX_SPEED": "50.0", "FORWARD_POWER": "35.0"}}}], "say": "Car controller installed! Sit on it to drive."}
```

If the user just wants to chat, respond naturally without JSON. Keep responses brief and friendly.

REMEMBER: Any request to build/create/rez MUST include a ```json block with {"actions": [...], "say": "..."} — this is how objects actually get created in the world. Without the JSON, nothing happens."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_table_parses() {
        let llm_output = r#"```json
{"actions": [
  {"rez_box": {"pos": [128,128,25.8], "scale": [2.0,1.0,0.08], "name": "Tabletop"}},
  {"rez_box": {"pos": [127.1,127.6,25.4], "scale": [0.1,0.1,0.8], "name": "Leg FL"}},
  {"rez_box": {"pos": [128.9,127.6,25.4], "scale": [0.1,0.1,0.8], "name": "Leg FR"}},
  {"rez_box": {"pos": [127.1,128.4,25.4], "scale": [0.1,0.1,0.8], "name": "Leg BL"}},
  {"rez_box": {"pos": [128.9,128.4,25.4], "scale": [0.1,0.1,0.8], "name": "Leg BR"}}
], "say": "Building you a wooden table with four legs!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(
            resp.actions.len(),
            5,
            "Table should have 5 prims (top + 4 legs)"
        );
        assert_eq!(
            resp.chat_text,
            "Building you a wooden table with four legs!"
        );
        match &resp.actions[0] {
            NPCAction::RezBox {
                position,
                scale,
                name,
            } => {
                assert_eq!(name, "Tabletop");
                assert!((position[2] - 25.8).abs() < 0.01, "Tabletop at 25.8m");
                assert!((scale[0] - 2.0).abs() < 0.01, "Tabletop 2m wide");
            }
            _ => panic!("First action should be RezBox for Tabletop"),
        }
        for i in 1..5 {
            match &resp.actions[i] {
                NPCAction::RezBox { name, .. } => {
                    assert!(name.starts_with("Leg"), "Action {} should be a leg", i)
                }
                _ => panic!("Actions 1-4 should be RezBox legs"),
            }
        }
    }

    #[test]
    fn test_make_it_taller_parses() {
        let llm_output = r#"```json
{"actions": [
  {"set_scale": {"local_id": 101, "scale": [2.0,1.0,0.08]}},
  {"set_position": {"local_id": 101, "pos": [128,128,26.2]}},
  {"set_scale": {"local_id": 102, "scale": [0.1,0.1,1.2]}},
  {"set_scale": {"local_id": 103, "scale": [0.1,0.1,1.2]}},
  {"set_scale": {"local_id": 104, "scale": [0.1,0.1,1.2]}},
  {"set_scale": {"local_id": 105, "scale": [0.1,0.1,1.2]}}
], "say": "Made the table taller! Legs are now 1.2m and the top is raised."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 6, "Should have 6 modify actions");
        match &resp.actions[0] {
            NPCAction::SetScale { local_id, .. } => assert_eq!(*local_id, 101),
            _ => panic!("First action should be SetScale"),
        }
        match &resp.actions[1] {
            NPCAction::SetPosition { local_id, position } => {
                assert_eq!(*local_id, 101);
                assert!(
                    (position[2] - 26.2).abs() < 0.01,
                    "Tabletop raised to 26.2m"
                );
            }
            _ => panic!("Second action should be SetPosition"),
        }
    }

    #[test]
    fn test_build_car_with_vehicle_script() {
        let llm_output = r#"```json
{"actions": [
  {"rez_box": {"pos": [128,128,25.3], "scale": [3.0,1.5,0.5], "name": "Car Body"}},
  {"rez_cylinder": {"pos": [127,127.2,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel FL"}},
  {"rez_cylinder": {"pos": [129,127.2,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel FR"}},
  {"rez_cylinder": {"pos": [127,128.8,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel RL"}},
  {"rez_cylinder": {"pos": [129,128.8,25.1], "scale": [0.6,0.6,0.2], "name": "Wheel RR"}},
  {"set_color": {"local_id": 200, "color": [0.8,0.1,0.1,1.0]}},
  {"set_color": {"local_id": 201, "color": [0.15,0.15,0.15,1.0]}},
  {"set_color": {"local_id": 202, "color": [0.15,0.15,0.15,1.0]}},
  {"set_color": {"local_id": 203, "color": [0.15,0.15,0.15,1.0]}},
  {"set_color": {"local_id": 204, "color": [0.15,0.15,0.15,1.0]}}
], "say": "Building a red car! I'll link the parts and add the controller script next."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 10, "5 rez + 5 color");
        let rez_count = resp
            .actions
            .iter()
            .filter(|a| matches!(a, NPCAction::RezBox { .. } | NPCAction::RezCylinder { .. }))
            .count();
        assert_eq!(rez_count, 5, "5 rez actions (body + 4 wheels)");
        let color_count = resp
            .actions
            .iter()
            .filter(|a| matches!(a, NPCAction::SetColor { .. }))
            .count();
        assert_eq!(color_count, 5, "5 color actions");
    }

    #[test]
    fn test_link_and_add_script() {
        let llm_output = r#"```json
{"actions": [
  {"link_objects": {"root_id": 200, "child_ids": [201, 202, 203, 204]}},
  {"insert_template_script": {"local_id": 200, "template_name": "car_controller", "params": {"MAX_SPEED": "50.0", "FORWARD_POWER": "35.0"}}}
], "say": "Car linked and controller script installed! Sit on it to drive."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 2);
        match &resp.actions[0] {
            NPCAction::LinkObjects { root_id, child_ids } => {
                assert_eq!(*root_id, 200);
                assert_eq!(child_ids, &vec![201, 202, 203, 204]);
            }
            _ => panic!("First action should be LinkObjects"),
        }
        match &resp.actions[1] {
            NPCAction::InsertTemplateScript {
                local_id,
                template_name,
                params,
            } => {
                assert_eq!(*local_id, 200);
                assert_eq!(template_name, "car_controller");
                assert_eq!(params.get("MAX_SPEED").unwrap(), "50.0");
                assert_eq!(params.get("FORWARD_POWER").unwrap(), "35.0");
            }
            _ => panic!("Second action should be InsertTemplateScript"),
        }
    }

    #[test]
    fn test_make_it_faster() {
        let llm_output = r#"```json
{"actions": [
  {"update_script": {"local_id": 200, "script_name": "car_controller", "script_source": "// updated car controller with higher power\ndefault { state_entry() { llSay(0, \"Faster car!\"); } }"}}
], "say": "Updated the car controller with more power!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::UpdateScript {
                local_id,
                script_name,
                script_source,
            } => {
                assert_eq!(*local_id, 200);
                assert_eq!(script_name, "car_controller");
                assert!(!script_source.is_empty());
            }
            _ => panic!("Should be UpdateScript"),
        }
    }

    #[test]
    fn test_give_object_to_user() {
        let llm_output = r#"```json
{"actions": [
  {"give_object": {"local_id": 200, "target_agent_id": "12345678-1234-1234-1234-123456789abc"}}
], "say": "Here you go! The car is now in your inventory."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::GiveObject {
                local_id,
                target_agent_id,
            } => {
                assert_eq!(*local_id, 200);
                assert_eq!(
                    target_agent_id.to_string(),
                    "12345678-1234-1234-1234-123456789abc"
                );
            }
            _ => panic!("Should be GiveObject"),
        }
    }

    #[test]
    fn test_build_sailboat_with_vessel_script() {
        let llm_output = r#"```json
{"actions": [
  {"rez_box": {"pos": [128,128,25.2], "scale": [8.0,2.5,0.3], "name": "Hull"}},
  {"rez_box": {"pos": [128,128,25.4], "scale": [6.0,2.0,0.1], "name": "Deck"}},
  {"rez_cylinder": {"pos": [128,128,27.5], "scale": [0.2,0.2,5.0], "name": "Mast"}},
  {"rez_box": {"pos": [128.5,128,27.0], "scale": [0.05,3.0,2.5], "name": "Sail"}},
  {"rez_prism": {"pos": [125,128,25.3], "scale": [2.0,2.5,0.3], "name": "Bow"}},
  {"set_color": {"local_id": 300, "color": [0.4,0.25,0.1,1.0]}},
  {"set_color": {"local_id": 301, "color": [0.6,0.4,0.2,1.0]}},
  {"set_color": {"local_id": 302, "color": [0.5,0.3,0.15,1.0]}},
  {"set_color": {"local_id": 303, "color": [1.0,1.0,1.0,1.0]}},
  {"set_color": {"local_id": 304, "color": [0.4,0.25,0.1,1.0]}}
], "say": "Building your sailboat! Hull, deck, mast, sail, and bow. I'll link and add sailing physics next."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 10, "5 rez + 5 color");
        match &resp.actions[4] {
            NPCAction::RezPrism { name, .. } => assert_eq!(name, "Bow"),
            _ => panic!("Bow should be a prism"),
        }
    }

    #[test]
    fn test_add_vessel_script_and_give() {
        let llm_output = r#"```json
{"actions": [
  {"link_objects": {"root_id": 300, "child_ids": [301, 302, 303, 304]}},
  {"insert_template_script": {"local_id": 300, "template_name": "vessel_controller", "params": {"FORWARD_POWER": "25.0", "WIND_BASE_SPEED": "12.0"}}},
  {"give_object": {"local_id": 300, "target_agent_id": "aabbccdd-1122-3344-5566-778899aabbcc"}}
], "say": "Sailboat linked with wind-powered sailing physics! It's been delivered to your inventory."}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 3);
        match &resp.actions[1] {
            NPCAction::InsertTemplateScript {
                template_name,
                params,
                ..
            } => {
                assert_eq!(template_name, "vessel_controller");
                assert_eq!(params.get("FORWARD_POWER").unwrap(), "25.0");
                assert_eq!(params.get("WIND_BASE_SPEED").unwrap(), "12.0");
            }
            _ => panic!("Should be InsertTemplateScript vessel_controller"),
        }
        assert!(matches!(&resp.actions[2], NPCAction::GiveObject { .. }));
    }

    #[test]
    fn test_rez_mesh_action() {
        let llm_output = r#"```json
{"actions": [
  {"rez_mesh": {"geometry_type": "sphere", "pos": [128,128,26], "scale": [2.0,2.0,2.0], "name": "Globe"}}
], "say": "Creating a mesh sphere!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::RezMesh {
                geometry_type,
                position,
                scale,
                name,
            } => {
                assert_eq!(geometry_type, "sphere");
                assert_eq!(name, "Globe");
                assert!((position[2] - 26.0).abs() < 0.01);
                assert!((scale[0] - 2.0).abs() < 0.01);
            }
            _ => panic!("Should be RezMesh"),
        }
    }

    #[test]
    fn test_import_mesh_action() {
        let llm_output = r#"```json
{"actions": [
  {"import_mesh": {"file_path": "/tmp/model.obj", "name": "Custom Model", "pos": [128,128,26]}}
], "say": "Importing your model!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::ImportMesh {
                file_path,
                name,
                position,
            } => {
                assert_eq!(file_path, "/tmp/model.obj");
                assert_eq!(name, "Custom Model");
                assert!((position[2] - 26.0).abs() < 0.01);
            }
            _ => panic!("Should be ImportMesh"),
        }
    }

    #[test]
    fn test_blender_generate_action() {
        let llm_output = r#"```json
{"actions": [
  {"blender_generate": {"template": "table", "params": {"WIDTH": "2.0", "HEIGHT": "0.9", "DEPTH": "1.0"}, "name": "Dining Table", "pos": [128,128,25]}}
], "say": "Generating a table via Blender!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::BlenderGenerate {
                template,
                params,
                name,
                position,
            } => {
                assert_eq!(template, "table");
                assert_eq!(name, "Dining Table");
                assert_eq!(params.get("WIDTH").unwrap(), "2.0");
                assert_eq!(params.get("HEIGHT").unwrap(), "0.9");
                assert_eq!(params.get("DEPTH").unwrap(), "1.0");
                assert!((position[2] - 25.0).abs() < 0.01);
            }
            _ => panic!("Should be BlenderGenerate"),
        }
    }

    #[test]
    fn test_blender_numeric_params() {
        let llm_output = r#"```json
{"actions": [
  {"blender_generate": {"template": "chair", "params": {"SEAT_W": 0.6, "SEAT_H": 0.45}, "name": "Chair", "pos": [128,128,25]}}
], "say": "Making a chair!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::BlenderGenerate { params, .. } => {
                assert_eq!(params.get("SEAT_W").unwrap(), "0.6");
                assert_eq!(params.get("SEAT_H").unwrap(), "0.45");
            }
            _ => panic!("Should be BlenderGenerate"),
        }
    }

    #[test]
    fn test_plain_chat_no_actions() {
        let resp = parse_npc_response("Hello! How can I help you today?");
        assert!(resp.actions.is_empty(), "Plain chat should have no actions");
        assert_eq!(resp.chat_text, "Hello! How can I help you today?");
    }

    #[test]
    fn test_template_params_actually_apply() {
        let params = {
            let mut p = HashMap::new();
            p.insert("MAX_SPEED".to_string(), "80.0".to_string());
            p.insert("FORWARD_POWER".to_string(), "50.0".to_string());
            p
        };
        let source = crate::ai::script_templates::apply_template("car_controller", &params);
        assert!(source.is_some(), "car_controller template should exist");
        let lsl = source.unwrap();
        assert!(
            lsl.contains("80.0"),
            "MAX_SPEED should be substituted to 80.0"
        );
        assert!(
            lsl.contains("50.0"),
            "FORWARD_POWER should be substituted to 50.0"
        );
        assert!(
            !lsl.contains("{{MAX_SPEED}}"),
            "Placeholder should be replaced"
        );
        assert!(
            lsl.contains("VEHICLE_TYPE_CAR"),
            "Should contain vehicle type"
        );
    }

    #[test]
    fn test_vessel_template_has_wind() {
        let params = HashMap::new();
        let source = crate::ai::script_templates::apply_template("vessel_controller", &params);
        assert!(source.is_some());
        let lsl = source.unwrap();
        assert!(lsl.contains("VEHICLE_TYPE_BOAT"), "Should be boat type");
        assert!(
            lsl.contains("gWindDir") || lsl.contains("wind"),
            "Should have wind simulation"
        );
    }

    #[test]
    fn test_plane_template_has_lift() {
        let params = HashMap::new();
        let source = crate::ai::script_templates::apply_template("plane_controller", &params);
        assert!(source.is_some());
        let lsl = source.unwrap();
        assert!(
            lsl.contains("VEHICLE_TYPE_AIRPLANE"),
            "Should be airplane type"
        );
        assert!(
            lsl.contains("lift") || lsl.contains("LIFT"),
            "Should have lift physics"
        );
        assert!(
            lsl.contains("stall") || lsl.contains("STALL"),
            "Should have stall detection"
        );
    }

    #[test]
    fn test_drone_cinematography_parses() {
        let llm_output = r#"```json
{"actions": [
  {"drone_cinematography": {
    "scene_name": "Hero Shot",
    "shot_type": "orbit",
    "subject_position": [128,128,30],
    "speed": 0.7,
    "camera_waypoints": [
      {"pos": [138,128,35], "focus": [128,128,30], "fov": 60, "dwell": 2.0},
      {"pos": [128,138,35], "focus": [128,128,30], "fov": 55, "dwell": 1.5}
    ],
    "lights": [
      {"name": "Key", "position": [123,133,32], "color": [1,0.95,0.85], "intensity": 0.9, "radius": 20, "falloff": 0.5}
    ]
  }}
], "say": "Setting up cinematic shot!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        assert_eq!(resp.chat_text, "Setting up cinematic shot!");
        match &resp.actions[0] {
            NPCAction::DroneCinematography {
                scene_name,
                shot_type,
                camera_waypoints,
                lights,
                lighting_preset: _,
                subject_position,
                speed,
            } => {
                assert_eq!(scene_name, "Hero Shot");
                assert_eq!(shot_type, "orbit");
                assert_eq!(camera_waypoints.len(), 2);
                assert_eq!(lights.len(), 1);
                assert!((subject_position[2] - 30.0).abs() < 0.01);
                assert!((*speed - 0.7).abs() < 0.01);
                assert_eq!(lights[0].name, "Key");
                assert!((camera_waypoints[0].fov - 60.0).abs() < 0.01);
            }
            _ => panic!("Should be DroneCinematography"),
        }
    }

    #[test]
    fn test_drone_cinematography_minimal() {
        let llm_output = r#"```json
{"actions": [
  {"drone_cinematography": {"scene_name": "Quick Shot", "shot_type": "dolly", "subject_position": [128,128,25]}}
], "say": "Filming!"}
```"#;
        let resp = parse_npc_response(llm_output);
        assert_eq!(resp.actions.len(), 1);
        match &resp.actions[0] {
            NPCAction::DroneCinematography {
                scene_name,
                shot_type,
                camera_waypoints,
                lights,
                speed,
                ..
            } => {
                assert_eq!(scene_name, "Quick Shot");
                assert_eq!(shot_type, "dolly");
                assert!(camera_waypoints.is_empty());
                assert!(lights.is_empty());
                assert!((*speed - 1.0).abs() < 0.01);
            }
            _ => panic!("Should be DroneCinematography"),
        }
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(
            extract_project_name("build me a wooden table"),
            "wooden table"
        );
        assert_eq!(extract_project_name("create a red car"), "red car");
        assert_eq!(extract_project_name("make a sailing boat"), "sailing boat");
        assert_eq!(extract_project_name("hello there"), "");
    }

    #[test]
    fn test_full_conversation_flow() {
        let turn1 = parse_npc_response(
            r#"```json
{"actions": [
  {"rez_box": {"pos": [128,128,25.8], "scale": [2.0,1.0,0.08], "name": "Tabletop"}},
  {"rez_box": {"pos": [127.1,127.6,25.4], "scale": [0.1,0.1,0.8], "name": "Leg FL"}},
  {"rez_box": {"pos": [128.9,127.6,25.4], "scale": [0.1,0.1,0.8], "name": "Leg FR"}},
  {"rez_box": {"pos": [127.1,128.4,25.4], "scale": [0.1,0.1,0.8], "name": "Leg BL"}},
  {"rez_box": {"pos": [128.9,128.4,25.4], "scale": [0.1,0.1,0.8], "name": "Leg BR"}}
], "say": "Here's your table!"}
```"#,
        );
        assert_eq!(turn1.actions.len(), 5);

        let turn2 = parse_npc_response(
            r#"```json
{"actions": [
  {"link_objects": {"root_id": 100, "child_ids": [101, 102, 103, 104]}},
  {"set_color": {"local_id": 100, "color": [0.55, 0.35, 0.17, 1.0]}},
  {"set_color": {"local_id": 101, "color": [0.55, 0.35, 0.17, 1.0]}},
  {"set_color": {"local_id": 102, "color": [0.55, 0.35, 0.17, 1.0]}},
  {"set_color": {"local_id": 103, "color": [0.55, 0.35, 0.17, 1.0]}},
  {"set_color": {"local_id": 104, "color": [0.55, 0.35, 0.17, 1.0]}}
], "say": "Linked and painted dark wood!"}
```"#,
        );
        assert_eq!(turn2.actions.len(), 6);
        assert!(matches!(&turn2.actions[0], NPCAction::LinkObjects { .. }));

        let turn3 = parse_npc_response(
            r#"```json
{"actions": [
  {"set_scale": {"local_id": 100, "scale": [2.0, 1.0, 0.08]}},
  {"set_position": {"local_id": 100, "pos": [128, 128, 26.4]}},
  {"set_scale": {"local_id": 101, "scale": [0.1, 0.1, 1.4]}},
  {"set_scale": {"local_id": 102, "scale": [0.1, 0.1, 1.4]}},
  {"set_scale": {"local_id": 103, "scale": [0.1, 0.1, 1.4]}},
  {"set_scale": {"local_id": 104, "scale": [0.1, 0.1, 1.4]}}
], "say": "Table is now taller!"}
```"#,
        );
        assert_eq!(turn3.actions.len(), 6);
        match &turn3.actions[0] {
            NPCAction::SetScale { local_id, .. } => assert_eq!(*local_id, 100),
            _ => panic!("Should be SetScale"),
        }
    }
}
