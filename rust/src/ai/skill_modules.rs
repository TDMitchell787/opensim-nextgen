use crate::ai::npc_avatar::NPCAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillDomain {
    Building,
    Clothing,
    Scripting,
    Landscaping,
    Guiding,
    Media,
}

impl SkillDomain {
    pub fn display_name(&self) -> &str {
        match self {
            SkillDomain::Building => "Building",
            SkillDomain::Clothing => "Clothing",
            SkillDomain::Scripting => "Scripting",
            SkillDomain::Landscaping => "Landscaping",
            SkillDomain::Guiding => "Guiding",
            SkillDomain::Media => "Media",
        }
    }
}

pub trait SkillModule: Send + Sync {
    fn domain(&self) -> SkillDomain;
    fn can_handle(&self, action: &NPCAction) -> bool;
    fn post_process(&self, _results: &[(String, u32)]) -> Option<String> {
        None
    }
}

pub struct BuildingModule;

impl SkillModule for BuildingModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Building
    }

    fn can_handle(&self, action: &NPCAction) -> bool {
        matches!(
            action,
            NPCAction::RezBox { .. }
                | NPCAction::RezCylinder { .. }
                | NPCAction::RezSphere { .. }
                | NPCAction::RezTorus { .. }
                | NPCAction::RezTube { .. }
                | NPCAction::RezRing { .. }
                | NPCAction::RezPrism { .. }
                | NPCAction::SetPosition { .. }
                | NPCAction::SetRotation { .. }
                | NPCAction::SetScale { .. }
                | NPCAction::SetColor { .. }
                | NPCAction::SetTexture { .. }
                | NPCAction::SetName { .. }
                | NPCAction::LinkObjects { .. }
                | NPCAction::DeleteObject { .. }
                | NPCAction::RezMesh { .. }
                | NPCAction::ImportMesh { .. }
                | NPCAction::BlenderGenerate { .. }
                | NPCAction::ExportOar { .. }
                | NPCAction::GiveObject { .. }
                | NPCAction::CreateBadge { .. }
        )
    }
}

pub struct ClothingModule;

impl SkillModule for ClothingModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Clothing
    }

    fn can_handle(&self, action: &NPCAction) -> bool {
        matches!(action, NPCAction::CreateTShirt { .. })
    }
}

pub struct ScriptingModule;

impl SkillModule for ScriptingModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Scripting
    }

    fn can_handle(&self, action: &NPCAction) -> bool {
        matches!(
            action,
            NPCAction::InsertScript { .. }
                | NPCAction::InsertTemplateScript { .. }
                | NPCAction::UpdateScript { .. }
        )
    }
}

pub struct LandscapingModule;

impl SkillModule for LandscapingModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Landscaping
    }

    fn can_handle(&self, action: &NPCAction) -> bool {
        matches!(
            action,
            NPCAction::TerrainGenerate { .. }
                | NPCAction::TerrainLoadR32 { .. }
                | NPCAction::TerrainLoadImage { .. }
                | NPCAction::TerrainPreview { .. }
                | NPCAction::TerrainApply { .. }
                | NPCAction::TerrainReject { .. }
        )
    }
}

pub struct GuidingModule;

impl SkillModule for GuidingModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Guiding
    }

    fn can_handle(&self, _action: &NPCAction) -> bool {
        false
    }
}

pub struct MediaModule;

impl SkillModule for MediaModule {
    fn domain(&self) -> SkillDomain {
        SkillDomain::Media
    }

    fn can_handle(&self, action: &NPCAction) -> bool {
        matches!(
            action,
            NPCAction::ComposeFilm { .. }
                | NPCAction::ComposeMusic { .. }
                | NPCAction::ComposeAd { .. }
                | NPCAction::ComposePhoto { .. }
                | NPCAction::DroneCinematography { .. }
                | NPCAction::LuxorSnapshot { .. }
                | NPCAction::LuxorVideo { .. }
        )
    }

    fn post_process(&self, results: &[(String, u32)]) -> Option<String> {
        if results.is_empty() {
            return None;
        }
        let has_camera = results.iter().any(|(n, _)| n.contains("Camera"));
        let light_count = results.iter().filter(|(n, _)| n.contains("Light")).count();
        if has_camera {
            Some(format!(
                "Cinematic scene ready: {} lights + camera drone. Sit on the drone to start.",
                light_count
            ))
        } else {
            Some(format!(
                "Media composition complete: {} elements created.",
                results.len()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_building_module_handles_rez() {
        let module = BuildingModule;
        assert_eq!(module.domain(), SkillDomain::Building);
        assert!(module.can_handle(&NPCAction::RezBox {
            position: [128.0, 128.0, 25.0],
            scale: [1.0, 1.0, 1.0],
            name: "Box".to_string(),
        }));
        assert!(module.can_handle(&NPCAction::RezSphere {
            position: [128.0, 128.0, 25.0],
            scale: [1.0, 1.0, 1.0],
            name: "Sphere".to_string(),
        }));
        assert!(module.can_handle(&NPCAction::SetColor {
            local_id: 100,
            color: [1.0, 0.0, 0.0, 1.0],
        }));
    }

    #[test]
    fn test_scripting_module_handles_scripts() {
        let module = ScriptingModule;
        assert_eq!(module.domain(), SkillDomain::Scripting);
        assert!(module.can_handle(&NPCAction::InsertScript {
            local_id: 100,
            script_name: "test".to_string(),
            script_source: "default{}".to_string(),
        }));
        assert!(module.can_handle(&NPCAction::InsertTemplateScript {
            local_id: 100,
            template_name: "rotating".to_string(),
            params: HashMap::new(),
        }));
        assert!(!module.can_handle(&NPCAction::RezBox {
            position: [0.0; 3],
            scale: [1.0; 3],
            name: "x".to_string(),
        }));
    }

    #[test]
    fn test_media_module_handles_media() {
        let module = MediaModule;
        assert_eq!(module.domain(), SkillDomain::Media);
        assert!(module.can_handle(&NPCAction::ComposeFilm {
            scene_name: "test".to_string(),
            description: "desc".to_string(),
        }));
        assert!(module.can_handle(&NPCAction::ComposeMusic {
            title: "test".to_string(),
            description: "desc".to_string(),
        }));
        assert!(module.can_handle(&NPCAction::ComposeAd {
            board_name: "test".to_string(),
            description: "desc".to_string(),
        }));
    }

    #[test]
    fn test_media_post_process() {
        let module = MediaModule;
        let results = vec![
            ("Screen".to_string(), 100u32),
            ("Backdrop".to_string(), 101),
        ];
        let msg = module.post_process(&results);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("2 elements"));
    }

    #[test]
    fn test_media_module_handles_cinematography() {
        let module = MediaModule;
        assert!(module.can_handle(&NPCAction::DroneCinematography {
            scene_name: "test".to_string(),
            shot_type: "orbit".to_string(),
            camera_waypoints: vec![],
            lights: vec![],
            lighting_preset: None,
            subject_position: [128.0, 128.0, 25.0],
            speed: 1.0,
        }));
    }

    #[test]
    fn test_media_cinematography_post_process() {
        let module = MediaModule;
        let results = vec![
            ("Key Light".to_string(), 100u32),
            ("Fill Light".to_string(), 101),
            ("Rim Light".to_string(), 102),
            ("Scene Camera".to_string(), 103),
        ];
        let msg = module.post_process(&results);
        assert!(msg.is_some());
        let text = msg.unwrap();
        assert!(text.contains("3 lights"));
        assert!(text.contains("camera drone"));
    }

    #[test]
    fn test_clothing_handles_nothing() {
        let module = ClothingModule;
        assert_eq!(module.domain(), SkillDomain::Clothing);
        assert!(!module.can_handle(&NPCAction::RezBox {
            position: [0.0; 3],
            scale: [1.0; 3],
            name: "x".to_string(),
        }));
    }

    #[test]
    fn test_skill_domain_display_names() {
        assert_eq!(SkillDomain::Building.display_name(), "Building");
        assert_eq!(SkillDomain::Clothing.display_name(), "Clothing");
        assert_eq!(SkillDomain::Scripting.display_name(), "Scripting");
        assert_eq!(SkillDomain::Landscaping.display_name(), "Landscaping");
        assert_eq!(SkillDomain::Guiding.display_name(), "Guiding");
        assert_eq!(SkillDomain::Media.display_name(), "Media");
    }
}
