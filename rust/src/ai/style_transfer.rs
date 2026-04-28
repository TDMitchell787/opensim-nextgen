use super::oar_analyzer::{OARAnalyzer, OARData};
use super::pattern_repository::PatternRepository;
use super::{AIError, ContentParameters, ContentType, GeneratedContent};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedStyle {
    pub style_id: Uuid,
    pub name: String,
    pub architectural_features: Vec<f32>,
    pub material_palette: Vec<MaterialDescriptor>,
    pub spatial_arrangement: SpatialPattern,
    pub complexity_profile: ComplexityProfile,
    pub color_palette: ColorPalette,
    pub texture_preferences: TexturePreferences,
    pub metadata: StyleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDescriptor {
    pub material_type: String,
    pub primary_color: [f32; 3],
    pub secondary_color: Option<[f32; 3]>,
    pub shininess: f32,
    pub transparency: f32,
    pub texture_scale: f32,
    pub bump_mapping: bool,
    pub usage_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialPattern {
    pub symmetry_type: SymmetryType,
    pub vertical_emphasis: f32,
    pub horizontal_emphasis: f32,
    pub depth_variation: f32,
    pub clustering_tendency: f32,
    pub spacing_uniformity: f32,
    pub height_distribution: HeightDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymmetryType {
    Bilateral,
    Radial,
    Asymmetric,
    Translational,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightDistribution {
    pub min_height: f32,
    pub max_height: f32,
    pub avg_height: f32,
    pub height_variance: f32,
    pub tall_structure_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityProfile {
    pub overall_complexity: f32,
    pub geometric_complexity: f32,
    pub material_variety: f32,
    pub detail_density: f32,
    pub prim_count_tendency: PrimCountTendency,
    pub script_complexity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimCountTendency {
    Minimalist,
    Moderate,
    Detailed,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary_colors: Vec<[f32; 3]>,
    pub accent_colors: Vec<[f32; 3]>,
    pub color_harmony: ColorHarmony,
    pub saturation_range: (f32, f32),
    pub brightness_range: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorHarmony {
    Monochromatic,
    Complementary,
    Analogous,
    Triadic,
    SplitComplementary,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexturePreferences {
    pub preferred_textures: Vec<String>,
    pub texture_resolution_preference: TextureResolution,
    pub seamless_tiling: bool,
    pub normal_mapping_usage: f32,
    pub pbr_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureResolution {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleMetadata {
    pub source_type: StyleSourceType,
    pub creation_date: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub description: String,
    pub popularity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleSourceType {
    Preset,
    LearnedFromOAR,
    UserCreated,
    Blended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleBlendRequest {
    pub styles: Vec<Uuid>,
    pub weights: Vec<f32>,
    pub blend_mode: BlendMode,
    pub output_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Linear,
    Weighted,
    Dominant,
    Layered,
}

pub struct StyleEncoder {
    pattern_repository: Arc<PatternRepository>,
    oar_analyzer: Arc<OARAnalyzer>,
    style_cache: Arc<RwLock<HashMap<Uuid, EncodedStyle>>>,
}

impl StyleEncoder {
    pub async fn new(
        pattern_repository: Arc<PatternRepository>,
        _db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>, AIError> {
        use super::oar_analyzer::AnalysisConfig;

        let oar_config = AnalysisConfig {
            deep_analysis: true,
            script_analysis: true,
            pattern_learning: true,
            quality_assessment: true,
            performance_metrics: true,
        };

        let oar_analyzer = Arc::new(OARAnalyzer::new(oar_config));

        let encoder = Self {
            pattern_repository,
            oar_analyzer,
            style_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        encoder.load_preset_styles().await?;

        Ok(Arc::new(encoder))
    }

    async fn load_preset_styles(&self) -> Result<(), AIError> {
        let presets = vec![
            self.create_modern_style(),
            self.create_victorian_style(),
            self.create_futuristic_style(),
            self.create_medieval_style(),
            self.create_minimalist_style(),
            self.create_industrial_style(),
            self.create_nature_style(),
            self.create_sci_fi_style(),
        ];

        let mut cache = self.style_cache.write().await;
        for style in presets {
            cache.insert(style.style_id, style);
        }

        Ok(())
    }

    fn create_modern_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Modern".to_string(),
            architectural_features: vec![0.8, 0.2, 0.9, 0.7, 0.3, 0.8, 0.6, 0.4],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Glass".to_string(),
                    primary_color: [0.8, 0.9, 1.0],
                    secondary_color: None,
                    shininess: 0.9,
                    transparency: 0.7,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.4,
                },
                MaterialDescriptor {
                    material_type: "Concrete".to_string(),
                    primary_color: [0.7, 0.7, 0.7],
                    secondary_color: None,
                    shininess: 0.1,
                    transparency: 0.0,
                    texture_scale: 2.0,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
                MaterialDescriptor {
                    material_type: "Steel".to_string(),
                    primary_color: [0.8, 0.8, 0.85],
                    secondary_color: None,
                    shininess: 0.8,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.3,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Bilateral,
                vertical_emphasis: 0.7,
                horizontal_emphasis: 0.5,
                depth_variation: 0.3,
                clustering_tendency: 0.2,
                spacing_uniformity: 0.8,
                height_distribution: HeightDistribution {
                    min_height: 3.0,
                    max_height: 50.0,
                    avg_height: 15.0,
                    height_variance: 0.6,
                    tall_structure_ratio: 0.4,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.6,
                geometric_complexity: 0.4,
                material_variety: 0.5,
                detail_density: 0.5,
                prim_count_tendency: PrimCountTendency::Moderate,
                script_complexity: 0.3,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.9, 0.9, 0.9], [0.2, 0.2, 0.2], [0.5, 0.5, 0.5]],
                accent_colors: vec![[0.0, 0.5, 0.8], [0.9, 0.6, 0.0]],
                color_harmony: ColorHarmony::Neutral,
                saturation_range: (0.0, 0.3),
                brightness_range: (0.3, 0.95),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "concrete".to_string(),
                    "glass".to_string(),
                    "metal".to_string(),
                ],
                texture_resolution_preference: TextureResolution::High,
                seamless_tiling: true,
                normal_mapping_usage: 0.5,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "modern".to_string(),
                    "urban".to_string(),
                    "clean".to_string(),
                ],
                description: "Clean, geometric modern architectural style".to_string(),
                popularity_score: 0.8,
            },
        }
    }

    fn create_victorian_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Victorian".to_string(),
            architectural_features: vec![0.3, 0.9, 0.4, 0.8, 0.7, 0.3, 0.9, 0.8],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Brick".to_string(),
                    primary_color: [0.6, 0.3, 0.2],
                    secondary_color: Some([0.5, 0.25, 0.15]),
                    shininess: 0.1,
                    transparency: 0.0,
                    texture_scale: 0.5,
                    bump_mapping: true,
                    usage_frequency: 0.4,
                },
                MaterialDescriptor {
                    material_type: "Wood".to_string(),
                    primary_color: [0.4, 0.25, 0.15],
                    secondary_color: None,
                    shininess: 0.3,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: true,
                    usage_frequency: 0.35,
                },
                MaterialDescriptor {
                    material_type: "Iron".to_string(),
                    primary_color: [0.2, 0.2, 0.22],
                    secondary_color: None,
                    shininess: 0.4,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.25,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Bilateral,
                vertical_emphasis: 0.8,
                horizontal_emphasis: 0.4,
                depth_variation: 0.6,
                clustering_tendency: 0.5,
                spacing_uniformity: 0.6,
                height_distribution: HeightDistribution {
                    min_height: 5.0,
                    max_height: 25.0,
                    avg_height: 12.0,
                    height_variance: 0.4,
                    tall_structure_ratio: 0.3,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.8,
                geometric_complexity: 0.7,
                material_variety: 0.6,
                detail_density: 0.9,
                prim_count_tendency: PrimCountTendency::Detailed,
                script_complexity: 0.2,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.6, 0.3, 0.2], [0.4, 0.25, 0.15], [0.8, 0.75, 0.7]],
                accent_colors: vec![[0.1, 0.3, 0.2], [0.5, 0.0, 0.0]],
                color_harmony: ColorHarmony::Analogous,
                saturation_range: (0.2, 0.6),
                brightness_range: (0.2, 0.8),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "brick".to_string(),
                    "wood".to_string(),
                    "ornate".to_string(),
                ],
                texture_resolution_preference: TextureResolution::High,
                seamless_tiling: true,
                normal_mapping_usage: 0.7,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "victorian".to_string(),
                    "historic".to_string(),
                    "ornate".to_string(),
                ],
                description: "Ornate Victorian era architectural style".to_string(),
                popularity_score: 0.7,
            },
        }
    }

    fn create_futuristic_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Futuristic".to_string(),
            architectural_features: vec![0.9, 0.1, 0.95, 0.5, 0.2, 0.95, 0.3, 0.2],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Holographic".to_string(),
                    primary_color: [0.4, 0.8, 1.0],
                    secondary_color: Some([0.8, 0.4, 1.0]),
                    shininess: 1.0,
                    transparency: 0.5,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.3,
                },
                MaterialDescriptor {
                    material_type: "Titanium".to_string(),
                    primary_color: [0.85, 0.85, 0.9],
                    secondary_color: None,
                    shininess: 0.95,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.4,
                },
                MaterialDescriptor {
                    material_type: "LED_Panel".to_string(),
                    primary_color: [0.0, 1.0, 0.8],
                    secondary_color: None,
                    shininess: 0.2,
                    transparency: 0.1,
                    texture_scale: 0.5,
                    bump_mapping: false,
                    usage_frequency: 0.3,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Radial,
                vertical_emphasis: 0.9,
                horizontal_emphasis: 0.6,
                depth_variation: 0.4,
                clustering_tendency: 0.3,
                spacing_uniformity: 0.9,
                height_distribution: HeightDistribution {
                    min_height: 10.0,
                    max_height: 200.0,
                    avg_height: 60.0,
                    height_variance: 0.8,
                    tall_structure_ratio: 0.6,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.7,
                geometric_complexity: 0.8,
                material_variety: 0.4,
                detail_density: 0.5,
                prim_count_tendency: PrimCountTendency::Moderate,
                script_complexity: 0.7,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.1, 0.1, 0.15], [0.85, 0.85, 0.9], [0.2, 0.2, 0.25]],
                accent_colors: vec![[0.0, 1.0, 0.8], [0.8, 0.0, 1.0], [1.0, 0.5, 0.0]],
                color_harmony: ColorHarmony::Complementary,
                saturation_range: (0.6, 1.0),
                brightness_range: (0.1, 1.0),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "metal".to_string(),
                    "glow".to_string(),
                    "circuit".to_string(),
                ],
                texture_resolution_preference: TextureResolution::Ultra,
                seamless_tiling: true,
                normal_mapping_usage: 0.3,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "futuristic".to_string(),
                    "sci-fi".to_string(),
                    "neon".to_string(),
                ],
                description: "High-tech futuristic architectural style".to_string(),
                popularity_score: 0.85,
            },
        }
    }

    fn create_medieval_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Medieval".to_string(),
            architectural_features: vec![0.2, 0.8, 0.3, 0.9, 0.8, 0.2, 0.7, 0.9],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Stone".to_string(),
                    primary_color: [0.5, 0.5, 0.45],
                    secondary_color: Some([0.4, 0.4, 0.35]),
                    shininess: 0.1,
                    transparency: 0.0,
                    texture_scale: 0.3,
                    bump_mapping: true,
                    usage_frequency: 0.5,
                },
                MaterialDescriptor {
                    material_type: "Timber".to_string(),
                    primary_color: [0.35, 0.2, 0.1],
                    secondary_color: None,
                    shininess: 0.2,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
                MaterialDescriptor {
                    material_type: "Thatch".to_string(),
                    primary_color: [0.6, 0.5, 0.3],
                    secondary_color: None,
                    shininess: 0.0,
                    transparency: 0.0,
                    texture_scale: 0.5,
                    bump_mapping: true,
                    usage_frequency: 0.2,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Asymmetric,
                vertical_emphasis: 0.6,
                horizontal_emphasis: 0.5,
                depth_variation: 0.7,
                clustering_tendency: 0.7,
                spacing_uniformity: 0.3,
                height_distribution: HeightDistribution {
                    min_height: 3.0,
                    max_height: 30.0,
                    avg_height: 10.0,
                    height_variance: 0.5,
                    tall_structure_ratio: 0.2,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.7,
                geometric_complexity: 0.5,
                material_variety: 0.4,
                detail_density: 0.6,
                prim_count_tendency: PrimCountTendency::Detailed,
                script_complexity: 0.1,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.5, 0.5, 0.45], [0.35, 0.2, 0.1], [0.6, 0.5, 0.3]],
                accent_colors: vec![[0.6, 0.1, 0.1], [0.1, 0.3, 0.1]],
                color_harmony: ColorHarmony::Analogous,
                saturation_range: (0.1, 0.4),
                brightness_range: (0.2, 0.6),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "stone".to_string(),
                    "wood".to_string(),
                    "thatch".to_string(),
                ],
                texture_resolution_preference: TextureResolution::Medium,
                seamless_tiling: true,
                normal_mapping_usage: 0.6,
                pbr_enabled: false,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "medieval".to_string(),
                    "fantasy".to_string(),
                    "rustic".to_string(),
                ],
                description: "Medieval castle and village architectural style".to_string(),
                popularity_score: 0.75,
            },
        }
    }

    fn create_minimalist_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Minimalist".to_string(),
            architectural_features: vec![0.95, 0.05, 0.9, 0.3, 0.1, 0.95, 0.2, 0.1],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "White_Plaster".to_string(),
                    primary_color: [0.98, 0.98, 0.98],
                    secondary_color: None,
                    shininess: 0.05,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.6,
                },
                MaterialDescriptor {
                    material_type: "Light_Wood".to_string(),
                    primary_color: [0.9, 0.8, 0.65],
                    secondary_color: None,
                    shininess: 0.3,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: true,
                    usage_frequency: 0.25,
                },
                MaterialDescriptor {
                    material_type: "Clear_Glass".to_string(),
                    primary_color: [0.95, 0.97, 1.0],
                    secondary_color: None,
                    shininess: 0.9,
                    transparency: 0.85,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.15,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Bilateral,
                vertical_emphasis: 0.4,
                horizontal_emphasis: 0.7,
                depth_variation: 0.2,
                clustering_tendency: 0.1,
                spacing_uniformity: 0.95,
                height_distribution: HeightDistribution {
                    min_height: 2.5,
                    max_height: 8.0,
                    avg_height: 4.0,
                    height_variance: 0.2,
                    tall_structure_ratio: 0.1,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.2,
                geometric_complexity: 0.15,
                material_variety: 0.2,
                detail_density: 0.1,
                prim_count_tendency: PrimCountTendency::Minimalist,
                script_complexity: 0.1,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.98, 0.98, 0.98], [0.9, 0.8, 0.65]],
                accent_colors: vec![[0.2, 0.2, 0.2]],
                color_harmony: ColorHarmony::Monochromatic,
                saturation_range: (0.0, 0.15),
                brightness_range: (0.85, 1.0),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec!["smooth".to_string(), "matte".to_string()],
                texture_resolution_preference: TextureResolution::Medium,
                seamless_tiling: true,
                normal_mapping_usage: 0.1,
                pbr_enabled: false,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "minimalist".to_string(),
                    "clean".to_string(),
                    "simple".to_string(),
                ],
                description: "Clean minimalist architectural style".to_string(),
                popularity_score: 0.7,
            },
        }
    }

    fn create_industrial_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Industrial".to_string(),
            architectural_features: vec![0.6, 0.4, 0.7, 0.6, 0.5, 0.6, 0.5, 0.5],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Exposed_Brick".to_string(),
                    primary_color: [0.55, 0.28, 0.2],
                    secondary_color: None,
                    shininess: 0.1,
                    transparency: 0.0,
                    texture_scale: 0.4,
                    bump_mapping: true,
                    usage_frequency: 0.35,
                },
                MaterialDescriptor {
                    material_type: "Rusty_Metal".to_string(),
                    primary_color: [0.45, 0.35, 0.25],
                    secondary_color: Some([0.5, 0.3, 0.2]),
                    shininess: 0.3,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: true,
                    usage_frequency: 0.35,
                },
                MaterialDescriptor {
                    material_type: "Concrete_Raw".to_string(),
                    primary_color: [0.55, 0.55, 0.52],
                    secondary_color: None,
                    shininess: 0.05,
                    transparency: 0.0,
                    texture_scale: 1.5,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Mixed,
                vertical_emphasis: 0.6,
                horizontal_emphasis: 0.6,
                depth_variation: 0.5,
                clustering_tendency: 0.4,
                spacing_uniformity: 0.5,
                height_distribution: HeightDistribution {
                    min_height: 4.0,
                    max_height: 20.0,
                    avg_height: 10.0,
                    height_variance: 0.4,
                    tall_structure_ratio: 0.3,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.6,
                geometric_complexity: 0.5,
                material_variety: 0.5,
                detail_density: 0.6,
                prim_count_tendency: PrimCountTendency::Moderate,
                script_complexity: 0.3,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.55, 0.28, 0.2], [0.45, 0.35, 0.25], [0.55, 0.55, 0.52]],
                accent_colors: vec![[0.9, 0.6, 0.1], [0.2, 0.2, 0.2]],
                color_harmony: ColorHarmony::Analogous,
                saturation_range: (0.1, 0.5),
                brightness_range: (0.2, 0.6),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "brick".to_string(),
                    "rust".to_string(),
                    "concrete".to_string(),
                ],
                texture_resolution_preference: TextureResolution::High,
                seamless_tiling: true,
                normal_mapping_usage: 0.8,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "industrial".to_string(),
                    "loft".to_string(),
                    "urban".to_string(),
                ],
                description: "Industrial loft architectural style".to_string(),
                popularity_score: 0.72,
            },
        }
    }

    fn create_nature_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Nature".to_string(),
            architectural_features: vec![0.3, 0.7, 0.4, 0.8, 0.6, 0.3, 0.8, 0.7],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Living_Wood".to_string(),
                    primary_color: [0.4, 0.3, 0.2],
                    secondary_color: Some([0.35, 0.45, 0.25]),
                    shininess: 0.2,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: true,
                    usage_frequency: 0.4,
                },
                MaterialDescriptor {
                    material_type: "Moss".to_string(),
                    primary_color: [0.2, 0.4, 0.15],
                    secondary_color: None,
                    shininess: 0.1,
                    transparency: 0.0,
                    texture_scale: 0.5,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
                MaterialDescriptor {
                    material_type: "Natural_Stone".to_string(),
                    primary_color: [0.5, 0.48, 0.42],
                    secondary_color: None,
                    shininess: 0.15,
                    transparency: 0.0,
                    texture_scale: 0.4,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Asymmetric,
                vertical_emphasis: 0.5,
                horizontal_emphasis: 0.5,
                depth_variation: 0.8,
                clustering_tendency: 0.6,
                spacing_uniformity: 0.3,
                height_distribution: HeightDistribution {
                    min_height: 2.0,
                    max_height: 15.0,
                    avg_height: 6.0,
                    height_variance: 0.6,
                    tall_structure_ratio: 0.2,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.7,
                geometric_complexity: 0.8,
                material_variety: 0.6,
                detail_density: 0.8,
                prim_count_tendency: PrimCountTendency::Detailed,
                script_complexity: 0.2,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.2, 0.4, 0.15], [0.4, 0.3, 0.2], [0.5, 0.48, 0.42]],
                accent_colors: vec![[0.8, 0.6, 0.3], [0.6, 0.2, 0.3]],
                color_harmony: ColorHarmony::Analogous,
                saturation_range: (0.2, 0.6),
                brightness_range: (0.15, 0.5),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "bark".to_string(),
                    "moss".to_string(),
                    "leaves".to_string(),
                ],
                texture_resolution_preference: TextureResolution::High,
                seamless_tiling: false,
                normal_mapping_usage: 0.7,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "nature".to_string(),
                    "organic".to_string(),
                    "forest".to_string(),
                ],
                description: "Organic nature-integrated architectural style".to_string(),
                popularity_score: 0.68,
            },
        }
    }

    fn create_sci_fi_style(&self) -> EncodedStyle {
        EncodedStyle {
            style_id: Uuid::new_v4(),
            name: "Sci-Fi".to_string(),
            architectural_features: vec![0.85, 0.15, 0.9, 0.6, 0.3, 0.85, 0.4, 0.3],
            material_palette: vec![
                MaterialDescriptor {
                    material_type: "Alien_Metal".to_string(),
                    primary_color: [0.3, 0.35, 0.4],
                    secondary_color: Some([0.4, 0.5, 0.55]),
                    shininess: 0.85,
                    transparency: 0.0,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.4,
                },
                MaterialDescriptor {
                    material_type: "Energy_Field".to_string(),
                    primary_color: [0.2, 0.8, 0.9],
                    secondary_color: Some([0.9, 0.2, 0.5]),
                    shininess: 1.0,
                    transparency: 0.7,
                    texture_scale: 1.0,
                    bump_mapping: false,
                    usage_frequency: 0.3,
                },
                MaterialDescriptor {
                    material_type: "Bio_Organic".to_string(),
                    primary_color: [0.3, 0.5, 0.35],
                    secondary_color: None,
                    shininess: 0.5,
                    transparency: 0.2,
                    texture_scale: 0.5,
                    bump_mapping: true,
                    usage_frequency: 0.3,
                },
            ],
            spatial_arrangement: SpatialPattern {
                symmetry_type: SymmetryType::Radial,
                vertical_emphasis: 0.8,
                horizontal_emphasis: 0.7,
                depth_variation: 0.5,
                clustering_tendency: 0.4,
                spacing_uniformity: 0.7,
                height_distribution: HeightDistribution {
                    min_height: 5.0,
                    max_height: 150.0,
                    avg_height: 40.0,
                    height_variance: 0.7,
                    tall_structure_ratio: 0.5,
                },
            },
            complexity_profile: ComplexityProfile {
                overall_complexity: 0.8,
                geometric_complexity: 0.9,
                material_variety: 0.5,
                detail_density: 0.7,
                prim_count_tendency: PrimCountTendency::Complex,
                script_complexity: 0.8,
            },
            color_palette: ColorPalette {
                primary_colors: vec![[0.1, 0.15, 0.2], [0.3, 0.35, 0.4], [0.2, 0.25, 0.3]],
                accent_colors: vec![[0.2, 0.8, 0.9], [0.9, 0.2, 0.5], [0.5, 1.0, 0.3]],
                color_harmony: ColorHarmony::Triadic,
                saturation_range: (0.5, 1.0),
                brightness_range: (0.1, 1.0),
            },
            texture_preferences: TexturePreferences {
                preferred_textures: vec![
                    "alien".to_string(),
                    "energy".to_string(),
                    "tech".to_string(),
                ],
                texture_resolution_preference: TextureResolution::Ultra,
                seamless_tiling: true,
                normal_mapping_usage: 0.4,
                pbr_enabled: true,
            },
            metadata: StyleMetadata {
                source_type: StyleSourceType::Preset,
                creation_date: "2026-01-22".to_string(),
                version: 1,
                tags: vec![
                    "sci-fi".to_string(),
                    "alien".to_string(),
                    "space".to_string(),
                ],
                description: "Alien sci-fi architectural style".to_string(),
                popularity_score: 0.78,
            },
        }
    }

    pub async fn encode_from_oar(&self, oar_data: &OARData) -> Result<EncodedStyle, AIError> {
        let architectural_features = self.extract_architectural_features(oar_data);
        let material_palette = self.extract_material_palette(oar_data);
        let spatial_arrangement = self.analyze_spatial_arrangement(oar_data);
        let complexity_profile = self.analyze_complexity(oar_data);
        let color_palette = self.extract_color_palette(oar_data);
        let texture_preferences = self.analyze_texture_preferences(oar_data);

        let region_name = &oar_data.metadata.region_name;

        Ok(EncodedStyle {
            style_id: Uuid::new_v4(),
            name: format!("Learned from {}", region_name),
            architectural_features,
            material_palette,
            spatial_arrangement,
            complexity_profile,
            color_palette,
            texture_preferences,
            metadata: StyleMetadata {
                source_type: StyleSourceType::LearnedFromOAR,
                creation_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: 1,
                tags: vec!["learned".to_string(), "oar".to_string()],
                description: format!("Style learned from OAR: {}", region_name),
                popularity_score: 0.5,
            },
        })
    }

    fn extract_architectural_features(&self, _analysis: &super::oar_analyzer::OARData) -> Vec<f32> {
        vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]
    }

    fn extract_material_palette(
        &self,
        _analysis: &super::oar_analyzer::OARData,
    ) -> Vec<MaterialDescriptor> {
        vec![MaterialDescriptor {
            material_type: "Default".to_string(),
            primary_color: [0.5, 0.5, 0.5],
            secondary_color: None,
            shininess: 0.5,
            transparency: 0.0,
            texture_scale: 1.0,
            bump_mapping: false,
            usage_frequency: 1.0,
        }]
    }

    fn analyze_spatial_arrangement(
        &self,
        _analysis: &super::oar_analyzer::OARData,
    ) -> SpatialPattern {
        SpatialPattern {
            symmetry_type: SymmetryType::Mixed,
            vertical_emphasis: 0.5,
            horizontal_emphasis: 0.5,
            depth_variation: 0.5,
            clustering_tendency: 0.5,
            spacing_uniformity: 0.5,
            height_distribution: HeightDistribution {
                min_height: 1.0,
                max_height: 20.0,
                avg_height: 5.0,
                height_variance: 0.5,
                tall_structure_ratio: 0.3,
            },
        }
    }

    fn analyze_complexity(&self, _analysis: &super::oar_analyzer::OARData) -> ComplexityProfile {
        ComplexityProfile {
            overall_complexity: 0.5,
            geometric_complexity: 0.5,
            material_variety: 0.5,
            detail_density: 0.5,
            prim_count_tendency: PrimCountTendency::Moderate,
            script_complexity: 0.3,
        }
    }

    fn extract_color_palette(&self, _analysis: &super::oar_analyzer::OARData) -> ColorPalette {
        ColorPalette {
            primary_colors: vec![[0.5, 0.5, 0.5]],
            accent_colors: vec![[0.7, 0.3, 0.3]],
            color_harmony: ColorHarmony::Neutral,
            saturation_range: (0.2, 0.8),
            brightness_range: (0.2, 0.8),
        }
    }

    fn analyze_texture_preferences(
        &self,
        _analysis: &super::oar_analyzer::OARData,
    ) -> TexturePreferences {
        TexturePreferences {
            preferred_textures: vec!["default".to_string()],
            texture_resolution_preference: TextureResolution::Medium,
            seamless_tiling: true,
            normal_mapping_usage: 0.5,
            pbr_enabled: false,
        }
    }

    pub async fn blend_styles(&self, request: &StyleBlendRequest) -> Result<EncodedStyle, AIError> {
        if request.styles.len() != request.weights.len() {
            return Err(AIError::ConfigurationError(
                "Number of styles must match number of weights".to_string(),
            ));
        }

        let total_weight: f32 = request.weights.iter().sum();
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(AIError::ConfigurationError(
                "Weights must sum to 1.0".to_string(),
            ));
        }

        let cache = self.style_cache.read().await;
        let mut styles: Vec<&EncodedStyle> = Vec::new();

        for style_id in &request.styles {
            let style = cache.get(style_id).ok_or_else(|| {
                AIError::ConfigurationError(format!("Style {} not found", style_id))
            })?;
            styles.push(style);
        }

        let blended_features = self.blend_vectors(
            &styles
                .iter()
                .map(|s| &s.architectural_features)
                .collect::<Vec<_>>(),
            &request.weights,
        );

        let primary_style = styles[0];
        let blended_complexity = ComplexityProfile {
            overall_complexity: styles
                .iter()
                .enumerate()
                .map(|(i, s)| s.complexity_profile.overall_complexity * request.weights[i])
                .sum(),
            geometric_complexity: styles
                .iter()
                .enumerate()
                .map(|(i, s)| s.complexity_profile.geometric_complexity * request.weights[i])
                .sum(),
            material_variety: styles
                .iter()
                .enumerate()
                .map(|(i, s)| s.complexity_profile.material_variety * request.weights[i])
                .sum(),
            detail_density: styles
                .iter()
                .enumerate()
                .map(|(i, s)| s.complexity_profile.detail_density * request.weights[i])
                .sum(),
            prim_count_tendency: primary_style.complexity_profile.prim_count_tendency.clone(),
            script_complexity: styles
                .iter()
                .enumerate()
                .map(|(i, s)| s.complexity_profile.script_complexity * request.weights[i])
                .sum(),
        };

        Ok(EncodedStyle {
            style_id: Uuid::new_v4(),
            name: request.output_name.clone(),
            architectural_features: blended_features,
            material_palette: primary_style.material_palette.clone(),
            spatial_arrangement: primary_style.spatial_arrangement.clone(),
            complexity_profile: blended_complexity,
            color_palette: primary_style.color_palette.clone(),
            texture_preferences: primary_style.texture_preferences.clone(),
            metadata: StyleMetadata {
                source_type: StyleSourceType::Blended,
                creation_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                version: 1,
                tags: vec!["blended".to_string()],
                description: format!("Blended style from {} sources", styles.len()),
                popularity_score: 0.5,
            },
        })
    }

    fn blend_vectors(&self, vectors: &[&Vec<f32>], weights: &[f32]) -> Vec<f32> {
        if vectors.is_empty() {
            return Vec::new();
        }

        let len = vectors[0].len();
        let mut result = vec![0.0; len];

        for (vec, weight) in vectors.iter().zip(weights.iter()) {
            for (i, val) in vec.iter().enumerate() {
                if i < result.len() {
                    result[i] += val * weight;
                }
            }
        }

        result
    }

    pub async fn get_style(&self, style_id: Uuid) -> Option<EncodedStyle> {
        self.style_cache.read().await.get(&style_id).cloned()
    }

    pub async fn get_all_styles(&self) -> Vec<EncodedStyle> {
        self.style_cache.read().await.values().cloned().collect()
    }

    pub async fn get_styles_by_tag(&self, tag: &str) -> Vec<EncodedStyle> {
        self.style_cache
            .read()
            .await
            .values()
            .filter(|s| s.metadata.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    pub async fn save_style(&self, style: EncodedStyle) -> Result<Uuid, AIError> {
        let style_id = style.style_id;
        self.style_cache.write().await.insert(style_id, style);
        Ok(style_id)
    }
}

pub struct StyleApplicator {
    encoder: Arc<StyleEncoder>,
}

impl StyleApplicator {
    pub async fn new(encoder: Arc<StyleEncoder>) -> Result<Arc<Self>, AIError> {
        Ok(Arc::new(Self { encoder }))
    }

    pub async fn apply_style(
        &self,
        base_content: &GeneratedContent,
        target_style: &EncodedStyle,
        strength: f32,
    ) -> Result<GeneratedContent, AIError> {
        let strength = strength.max(0.0).min(1.0);

        let modified_data =
            self.modify_content_with_style(&base_content.data, target_style, strength)?;

        let mut metadata = base_content.metadata.clone();
        metadata.insert("applied_style".to_string(), target_style.name.clone());
        metadata.insert("style_strength".to_string(), strength.to_string());

        Ok(GeneratedContent {
            content_type: base_content.content_type.clone(),
            data: modified_data,
            metadata,
            generation_time_ms: base_content.generation_time_ms,
        })
    }

    fn modify_content_with_style(
        &self,
        _data: &[u8],
        _style: &EncodedStyle,
        _strength: f32,
    ) -> Result<Vec<u8>, AIError> {
        Ok(Vec::new())
    }

    pub async fn generate_in_style(
        &self,
        content_type: ContentType,
        parameters: ContentParameters,
        style: &EncodedStyle,
    ) -> Result<GeneratedContent, AIError> {
        let mut metadata = parameters.additional_params.clone();
        metadata.insert("style_name".to_string(), style.name.clone());
        metadata.insert("style_id".to_string(), style.style_id.to_string());

        Ok(GeneratedContent {
            content_type,
            data: Vec::new(),
            metadata,
            generation_time_ms: 0,
        })
    }

    pub fn calculate_style_similarity(
        &self,
        style_a: &EncodedStyle,
        style_b: &EncodedStyle,
    ) -> f64 {
        let feature_similarity = self.vector_cosine_similarity(
            &style_a.architectural_features,
            &style_b.architectural_features,
        );

        let complexity_similarity = 1.0
            - ((style_a.complexity_profile.overall_complexity
                - style_b.complexity_profile.overall_complexity)
                .abs() as f64);

        (feature_similarity * 0.6 + complexity_similarity * 0.4)
            .max(0.0)
            .min(1.0)
    }

    fn vector_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        (dot_product / (magnitude_a * magnitude_b)) as f64
    }
}
