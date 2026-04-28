use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub max_prims_per_object: u32,
    pub max_prims_per_linkset: u32,
    pub max_prims_per_region: u32,
    pub max_texture_width: u32,
    pub max_texture_height: u32,
    pub max_script_size_bytes: usize,
    pub max_script_memory_bytes: usize,
    pub allowed_texture_formats: Vec<String>,
    pub max_mesh_vertices: u32,
    pub max_mesh_triangles: u32,
    pub max_mesh_submeshes: u32,
    pub max_physics_vertices: u32,
    pub max_physics_triangles: u32,
    pub max_land_impact: u32,
    pub max_object_size: f32,
    pub min_object_size: f32,
    pub max_link_distance: f32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_prims_per_object: 256,
            max_prims_per_linkset: 256,
            max_prims_per_region: 15000,
            max_texture_width: 1024,
            max_texture_height: 1024,
            max_script_size_bytes: 65536,
            max_script_memory_bytes: 65536,
            allowed_texture_formats: vec![
                "jpeg".to_string(),
                "jpg".to_string(),
                "png".to_string(),
                "tga".to_string(),
                "bmp".to_string(),
                "j2c".to_string(),
                "jp2".to_string(),
            ],
            max_mesh_vertices: 65535,
            max_mesh_triangles: 21844,
            max_mesh_submeshes: 8,
            max_physics_vertices: 256,
            max_physics_triangles: 256,
            max_land_impact: 1000,
            max_object_size: 256.0,
            min_object_size: 0.01,
            max_link_distance: 54.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
    pub metrics: ValidationMetrics,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            metrics: ValidationMetrics::default(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.suggestions.extend(other.suggestions);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub message: String,
    pub object_id: Option<Uuid>,
    pub details: Option<String>,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationErrorType {
    PrimLimitExceeded,
    LinksetLimitExceeded,
    RegionLimitExceeded,
    TextureTooLarge,
    InvalidTextureFormat,
    ScriptTooLarge,
    ScriptMemoryExceeded,
    MeshVertexLimitExceeded,
    MeshTriangleLimitExceeded,
    MeshSubmeshLimitExceeded,
    PhysicsComplexityExceeded,
    LandImpactExceeded,
    ObjectTooLarge,
    ObjectTooSmall,
    LinkDistanceExceeded,
    InvalidGeometry,
    InvalidMaterial,
    InvalidScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: ValidationWarningType,
    pub message: String,
    pub object_id: Option<Uuid>,
    pub impact: WarningImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationWarningType {
    HighPrimCount,
    LargeTexture,
    ComplexScript,
    HighLandImpact,
    UnoptimizedMesh,
    NonStandardPhysics,
    DeprecatedFeature,
    PerformanceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarningImpact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationMetrics {
    pub total_prims: u32,
    pub total_scripts: u32,
    pub total_textures: u32,
    pub total_meshes: u32,
    pub estimated_land_impact: u32,
    pub estimated_download_weight: f32,
    pub estimated_physics_weight: f32,
    pub estimated_server_weight: f32,
}

#[derive(Debug, Clone)]
pub struct ContentValidator {
    config: ValidationConfig,
}

impl ContentValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ValidationConfig::default())
    }

    pub fn validate_content(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        result.merge(self.validate_prim_count(content));
        result.merge(self.validate_textures(content));
        result.merge(self.validate_scripts(content));
        result.merge(self.validate_meshes(content));
        result.merge(self.validate_physics(content));
        result.merge(self.validate_dimensions(content));
        result.merge(self.validate_linksets(content));

        self.calculate_metrics(&mut result, content);
        self.generate_suggestions(&mut result, content);

        result
    }

    pub fn validate_prim_count(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        let total_prims = content.objects.iter().map(|o| o.prim_count).sum::<u32>();

        for obj in &content.objects {
            if obj.prim_count > self.config.max_prims_per_object {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::PrimLimitExceeded,
                    message: format!(
                        "Object '{}' has {} prims, exceeds limit of {}",
                        obj.name, obj.prim_count, self.config.max_prims_per_object
                    ),
                    object_id: Some(obj.id),
                    details: Some(format!(
                        "Consider splitting into {} separate objects",
                        (obj.prim_count as f32 / self.config.max_prims_per_object as f32).ceil()
                            as u32
                    )),
                    auto_fixable: false,
                });
            } else if obj.prim_count > self.config.max_prims_per_object * 80 / 100 {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::HighPrimCount,
                    message: format!(
                        "Object '{}' has {} prims, approaching limit",
                        obj.name, obj.prim_count
                    ),
                    object_id: Some(obj.id),
                    impact: WarningImpact::Medium,
                });
            }
        }

        if total_prims > self.config.max_prims_per_region {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::RegionLimitExceeded,
                message: format!(
                    "Total prim count {} exceeds region limit of {}",
                    total_prims, self.config.max_prims_per_region
                ),
                object_id: None,
                details: Some("Consider reducing object count or complexity".to_string()),
                auto_fixable: false,
            });
        }

        result
    }

    pub fn validate_textures(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();
        let allowed_formats: HashSet<_> = self
            .config
            .allowed_texture_formats
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        for texture in &content.textures {
            if texture.width > self.config.max_texture_width
                || texture.height > self.config.max_texture_height
            {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::TextureTooLarge,
                    message: format!(
                        "Texture '{}' is {}x{}, exceeds max {}x{}",
                        texture.name,
                        texture.width,
                        texture.height,
                        self.config.max_texture_width,
                        self.config.max_texture_height
                    ),
                    object_id: texture.object_id,
                    details: Some(format!(
                        "Resize to {}x{} or smaller",
                        self.config.max_texture_width, self.config.max_texture_height
                    )),
                    auto_fixable: true,
                });
            } else if texture.width > 512 || texture.height > 512 {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::LargeTexture,
                    message: format!(
                        "Texture '{}' is {}x{}, consider using 512x512 for better performance",
                        texture.name, texture.width, texture.height
                    ),
                    object_id: texture.object_id,
                    impact: WarningImpact::Low,
                });
            }

            let format_lower = texture.format.to_lowercase();
            if !allowed_formats.contains(&format_lower) {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::InvalidTextureFormat,
                    message: format!(
                        "Texture '{}' has unsupported format '{}'",
                        texture.name, texture.format
                    ),
                    object_id: texture.object_id,
                    details: Some(format!(
                        "Supported formats: {:?}",
                        self.config.allowed_texture_formats
                    )),
                    auto_fixable: true,
                });
            }
        }

        result
    }

    pub fn validate_scripts(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        for script in &content.scripts {
            if script.size_bytes > self.config.max_script_size_bytes {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::ScriptTooLarge,
                    message: format!(
                        "Script '{}' is {} bytes, exceeds limit of {} bytes",
                        script.name, script.size_bytes, self.config.max_script_size_bytes
                    ),
                    object_id: script.object_id,
                    details: Some(
                        "Consider splitting into multiple scripts or optimizing code".to_string(),
                    ),
                    auto_fixable: false,
                });
            }

            if script.estimated_memory > self.config.max_script_memory_bytes {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::ScriptMemoryExceeded,
                    message: format!(
                        "Script '{}' uses estimated {} bytes memory, exceeds limit of {} bytes",
                        script.name, script.estimated_memory, self.config.max_script_memory_bytes
                    ),
                    object_id: script.object_id,
                    details: Some("Reduce list sizes and global variables".to_string()),
                    auto_fixable: false,
                });
            }

            if script.complexity_score > 7.0 {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::ComplexScript,
                    message: format!(
                        "Script '{}' has high complexity score ({:.1})",
                        script.name, script.complexity_score
                    ),
                    object_id: script.object_id,
                    impact: WarningImpact::Medium,
                });
            }

            if script.has_deprecated_functions {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::DeprecatedFeature,
                    message: format!("Script '{}' uses deprecated functions", script.name),
                    object_id: script.object_id,
                    impact: WarningImpact::Low,
                });
            }
        }

        result
    }

    pub fn validate_meshes(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        for mesh in &content.meshes {
            if mesh.vertex_count > self.config.max_mesh_vertices {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::MeshVertexLimitExceeded,
                    message: format!(
                        "Mesh '{}' has {} vertices, exceeds limit of {}",
                        mesh.name, mesh.vertex_count, self.config.max_mesh_vertices
                    ),
                    object_id: mesh.object_id,
                    details: Some("Use mesh decimation to reduce vertex count".to_string()),
                    auto_fixable: false,
                });
            }

            if mesh.triangle_count > self.config.max_mesh_triangles {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::MeshTriangleLimitExceeded,
                    message: format!(
                        "Mesh '{}' has {} triangles, exceeds limit of {}",
                        mesh.name, mesh.triangle_count, self.config.max_mesh_triangles
                    ),
                    object_id: mesh.object_id,
                    details: Some("Reduce polygon count or split mesh".to_string()),
                    auto_fixable: false,
                });
            }

            if mesh.submesh_count > self.config.max_mesh_submeshes {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::MeshSubmeshLimitExceeded,
                    message: format!(
                        "Mesh '{}' has {} submeshes, exceeds limit of {}",
                        mesh.name, mesh.submesh_count, self.config.max_mesh_submeshes
                    ),
                    object_id: mesh.object_id,
                    details: Some("Combine materials or split into separate meshes".to_string()),
                    auto_fixable: false,
                });
            }

            if !mesh.has_lod_levels {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::UnoptimizedMesh,
                    message: format!("Mesh '{}' lacks LOD levels", mesh.name),
                    object_id: mesh.object_id,
                    impact: WarningImpact::Medium,
                });
            }
        }

        result
    }

    pub fn validate_physics(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        for physics in &content.physics_shapes {
            if physics.vertex_count > self.config.max_physics_vertices {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::PhysicsComplexityExceeded,
                    message: format!(
                        "Physics shape for '{}' has {} vertices, exceeds limit of {}",
                        physics.object_name, physics.vertex_count, self.config.max_physics_vertices
                    ),
                    object_id: physics.object_id,
                    details: Some("Simplify physics shape or use convex decomposition".to_string()),
                    auto_fixable: false,
                });
            }

            if physics.triangle_count > self.config.max_physics_triangles {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::PhysicsComplexityExceeded,
                    message: format!(
                        "Physics shape for '{}' has {} triangles, exceeds limit of {}",
                        physics.object_name,
                        physics.triangle_count,
                        self.config.max_physics_triangles
                    ),
                    object_id: physics.object_id,
                    details: Some("Use simpler collision shape".to_string()),
                    auto_fixable: false,
                });
            }

            if physics.is_non_convex && physics.triangle_count > 64 {
                result.add_warning(ValidationWarning {
                    warning_type: ValidationWarningType::NonStandardPhysics,
                    message: format!(
                        "Non-convex physics for '{}' may cause performance issues",
                        physics.object_name
                    ),
                    object_id: physics.object_id,
                    impact: WarningImpact::High,
                });
            }
        }

        result
    }

    pub fn validate_dimensions(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        for obj in &content.objects {
            let max_dim = obj.scale.0.max(obj.scale.1).max(obj.scale.2);
            let min_dim = obj.scale.0.min(obj.scale.1).min(obj.scale.2);

            if max_dim > self.config.max_object_size {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::ObjectTooLarge,
                    message: format!(
                        "Object '{}' dimension {:.2}m exceeds max {:.2}m",
                        obj.name, max_dim, self.config.max_object_size
                    ),
                    object_id: Some(obj.id),
                    details: Some("Scale down or split into multiple objects".to_string()),
                    auto_fixable: true,
                });
            }

            if min_dim < self.config.min_object_size {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::ObjectTooSmall,
                    message: format!(
                        "Object '{}' dimension {:.4}m is below min {:.2}m",
                        obj.name, min_dim, self.config.min_object_size
                    ),
                    object_id: Some(obj.id),
                    details: Some("Scale up to minimum size".to_string()),
                    auto_fixable: true,
                });
            }
        }

        result
    }

    pub fn validate_linksets(&self, content: &ValidatableContent) -> ValidationResult {
        let mut result = ValidationResult::new();

        for linkset in &content.linksets {
            if linkset.child_count > self.config.max_prims_per_linkset {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::LinksetLimitExceeded,
                    message: format!(
                        "Linkset '{}' has {} prims, exceeds limit of {}",
                        linkset.root_name, linkset.child_count, self.config.max_prims_per_linkset
                    ),
                    object_id: Some(linkset.root_id),
                    details: Some("Split into multiple linksets".to_string()),
                    auto_fixable: false,
                });
            }

            if linkset.max_link_distance > self.config.max_link_distance {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::LinkDistanceExceeded,
                    message: format!(
                        "Linkset '{}' has link distance {:.2}m, exceeds max {:.2}m",
                        linkset.root_name, linkset.max_link_distance, self.config.max_link_distance
                    ),
                    object_id: Some(linkset.root_id),
                    details: Some("Move linked prims closer together".to_string()),
                    auto_fixable: false,
                });
            }
        }

        result
    }

    fn calculate_metrics(&self, result: &mut ValidationResult, content: &ValidatableContent) {
        result.metrics.total_prims = content.objects.iter().map(|o| o.prim_count).sum();
        result.metrics.total_scripts = content.scripts.len() as u32;
        result.metrics.total_textures = content.textures.len() as u32;
        result.metrics.total_meshes = content.meshes.len() as u32;

        let download_weight: f32 = content
            .meshes
            .iter()
            .map(|m| m.vertex_count as f32 * 0.001 + m.triangle_count as f32 * 0.0005)
            .sum::<f32>()
            + content
                .textures
                .iter()
                .map(|t| (t.width * t.height) as f32 * 0.00001)
                .sum::<f32>();

        let physics_weight: f32 = content
            .physics_shapes
            .iter()
            .map(|p| p.vertex_count as f32 * 0.01 + p.triangle_count as f32 * 0.02)
            .sum();

        let server_weight: f32 = content
            .scripts
            .iter()
            .map(|s| s.complexity_score as f32 * 0.1)
            .sum::<f32>()
            + result.metrics.total_prims as f32 * 0.01;

        result.metrics.estimated_download_weight = download_weight;
        result.metrics.estimated_physics_weight = physics_weight;
        result.metrics.estimated_server_weight = server_weight;
        result.metrics.estimated_land_impact =
            (result.metrics.total_prims as f32 + download_weight + physics_weight) as u32;
    }

    fn generate_suggestions(&self, result: &mut ValidationResult, content: &ValidatableContent) {
        if result.metrics.total_prims > 100 {
            result.add_suggestion("Consider using mesh objects to reduce prim count".to_string());
        }

        if result.metrics.total_textures > 20 {
            result.add_suggestion(
                "Consider using texture atlases to reduce texture count".to_string(),
            );
        }

        let large_textures = content
            .textures
            .iter()
            .filter(|t| t.width > 512 || t.height > 512)
            .count();
        if large_textures > 5 {
            result.add_suggestion(format!(
                "Found {} textures larger than 512x512, consider downscaling for performance",
                large_textures
            ));
        }

        if content.scripts.len() > 10 {
            result.add_suggestion(
                "Many scripts detected. Consider consolidating into fewer scripts".to_string(),
            );
        }

        let unoptimized_meshes = content.meshes.iter().filter(|m| !m.has_lod_levels).count();
        if unoptimized_meshes > 0 {
            result.add_suggestion(format!(
                "{} meshes lack LOD levels. Add LODs for better rendering performance",
                unoptimized_meshes
            ));
        }

        if result.metrics.estimated_land_impact > self.config.max_land_impact / 2 {
            result.add_suggestion(format!(
                "Estimated land impact {} is high. Consider optimization",
                result.metrics.estimated_land_impact
            ));
        }
    }

    pub fn validate_quick(&self, prim_count: u32, script_count: u32, has_mesh: bool) -> bool {
        prim_count <= self.config.max_prims_per_object
            && script_count <= 100
            && (!has_mesh || prim_count <= self.config.max_mesh_vertices / 100)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableContent {
    pub objects: Vec<ValidatableObject>,
    pub textures: Vec<ValidatableTexture>,
    pub scripts: Vec<ValidatableScript>,
    pub meshes: Vec<ValidatableMesh>,
    pub physics_shapes: Vec<ValidatablePhysics>,
    pub linksets: Vec<ValidatableLinkset>,
}

impl ValidatableContent {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            textures: Vec::new(),
            scripts: Vec::new(),
            meshes: Vec::new(),
            physics_shapes: Vec::new(),
            linksets: Vec::new(),
        }
    }
}

impl Default for ValidatableContent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableObject {
    pub id: Uuid,
    pub name: String,
    pub prim_count: u32,
    pub scale: (f32, f32, f32),
    pub position: (f32, f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableTexture {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub object_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableScript {
    pub id: Uuid,
    pub name: String,
    pub size_bytes: usize,
    pub estimated_memory: usize,
    pub complexity_score: f64,
    pub has_deprecated_functions: bool,
    pub object_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableMesh {
    pub id: Uuid,
    pub name: String,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub submesh_count: u32,
    pub has_lod_levels: bool,
    pub object_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatablePhysics {
    pub object_id: Option<Uuid>,
    pub object_name: String,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub is_non_convex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatableLinkset {
    pub root_id: Uuid,
    pub root_name: String,
    pub child_count: u32,
    pub max_link_distance: f32,
}

pub fn convert_generated_content_to_validatable(
    objects: &[super::oar_analyzer::AnalyzedObject],
    scripts: &[super::oar_analyzer::ScriptData],
) -> ValidatableContent {
    let mut content = ValidatableContent::new();

    for obj in objects {
        content.objects.push(ValidatableObject {
            id: Uuid::parse_str(&obj.uuid).unwrap_or_else(|_| Uuid::new_v4()),
            name: obj.name.clone(),
            prim_count: 1 + obj.children.len() as u32,
            scale: obj.scale,
            position: obj.position,
        });

        if let Some(texture_id) = &obj.material_data.texture_id {
            content.textures.push(ValidatableTexture {
                id: Uuid::parse_str(texture_id).unwrap_or_else(|_| Uuid::new_v4()),
                name: format!("texture_{}", texture_id),
                width: 512,
                height: 512,
                format: "j2c".to_string(),
                object_id: Some(Uuid::parse_str(&obj.uuid).unwrap_or_else(|_| Uuid::new_v4())),
            });
        }

        if !obj.children.is_empty() {
            let obj_id = Uuid::parse_str(&obj.uuid).unwrap_or_else(|_| Uuid::new_v4());
            let positions: Vec<_> = obj.children.iter().map(|c| c.position).collect();
            let max_distance = positions
                .iter()
                .map(|p| {
                    ((p.0 - obj.position.0).powi(2)
                        + (p.1 - obj.position.1).powi(2)
                        + (p.2 - obj.position.2).powi(2))
                    .sqrt()
                })
                .fold(0.0_f32, f32::max);

            content.linksets.push(ValidatableLinkset {
                root_id: obj_id,
                root_name: obj.name.clone(),
                child_count: obj.children.len() as u32,
                max_link_distance: max_distance,
            });
        }
    }

    for script in scripts {
        content.scripts.push(ValidatableScript {
            id: Uuid::parse_str(&script.uuid).unwrap_or_else(|_| Uuid::new_v4()),
            name: script.name.clone(),
            size_bytes: script.source_code.len(),
            estimated_memory: script.source_code.len() * 2,
            complexity_score: script.complexity_score,
            has_deprecated_functions: false,
            object_id: None,
        });
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidationConfig::default();
        assert_eq!(config.max_prims_per_object, 256);
        assert_eq!(config.max_texture_width, 1024);
        assert_eq!(config.max_script_size_bytes, 65536);
    }

    #[test]
    fn test_empty_content_is_valid() {
        let validator = ContentValidator::with_defaults();
        let content = ValidatableContent::new();
        let result = validator.validate_content(&content);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_prim_limit_exceeded() {
        let validator = ContentValidator::with_defaults();
        let mut content = ValidatableContent::new();
        content.objects.push(ValidatableObject {
            id: Uuid::new_v4(),
            name: "test_object".to_string(),
            prim_count: 300,
            scale: (1.0, 1.0, 1.0),
            position: (0.0, 0.0, 0.0),
        });

        let result = validator.validate_content(&content);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::PrimLimitExceeded));
    }

    #[test]
    fn test_texture_size_exceeded() {
        let validator = ContentValidator::with_defaults();
        let mut content = ValidatableContent::new();
        content.textures.push(ValidatableTexture {
            id: Uuid::new_v4(),
            name: "large_texture".to_string(),
            width: 2048,
            height: 2048,
            format: "png".to_string(),
            object_id: None,
        });

        let result = validator.validate_content(&content);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::TextureTooLarge));
    }

    #[test]
    fn test_invalid_texture_format() {
        let validator = ContentValidator::with_defaults();
        let mut content = ValidatableContent::new();
        content.textures.push(ValidatableTexture {
            id: Uuid::new_v4(),
            name: "test_texture".to_string(),
            width: 512,
            height: 512,
            format: "gif".to_string(),
            object_id: None,
        });

        let result = validator.validate_content(&content);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::InvalidTextureFormat));
    }

    #[test]
    fn test_quick_validation() {
        let validator = ContentValidator::with_defaults();
        assert!(validator.validate_quick(100, 5, false));
        assert!(!validator.validate_quick(300, 5, false));
    }
}
