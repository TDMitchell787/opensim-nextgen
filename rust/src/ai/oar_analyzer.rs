use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use zip::ZipArchive;

use crate::ai::content_creation::{
    ContentCategory, ContentPattern, CreatorAttribution, GeometricAnalysis, MaterialAnalysis,
    PrimitiveType, RecognitionData, ScriptingPattern, SpatialAnalysis, UsageStatistics,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OARAnalyzer {
    /// Pattern recognition database
    pub pattern_database: HashMap<String, AnalysisPattern>,
    /// EADS learning metrics
    pub learning_metrics: LearningMetrics,
    /// Analysis configuration
    pub config: AnalysisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OARData {
    /// Archive metadata
    pub metadata: OARMetadata,
    /// Region information
    pub region_info: RegionInfo,
    /// Objects and primitives
    pub objects: Vec<AnalyzedObject>,
    /// Terrain data
    pub terrain: Option<TerrainData>,
    /// Assets (textures, sounds, etc.)
    pub assets: Vec<AssetData>,
    /// Scripts found in the archive
    pub scripts: Vec<ScriptData>,
    /// Parcel information
    pub parcels: Vec<ParcelData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OARMetadata {
    pub version: String,
    pub created_date: Option<chrono::DateTime<chrono::Utc>>,
    pub creator: Option<String>,
    pub region_name: String,
    pub region_uuid: String,
    pub total_objects: u32,
    pub total_prims: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub name: String,
    pub uuid: String,
    pub size: (u32, u32), // X, Y size
    pub water_height: f32,
    pub terrain_multipliers: (f32, f32, f32, f32),
    pub estate_settings: Option<EstateSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedObject {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub primitive_type: PrimitiveType,
    pub material_data: MaterialData,
    pub physics_data: PhysicsData,
    pub scripts: Vec<String>,          // Script UUIDs
    pub children: Vec<AnalyzedObject>, // Linked prims
    pub metadata: ObjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub texture_id: Option<String>,
    pub color: (f32, f32, f32, f32), // RGBA
    pub alpha: f32,
    pub glow: f32,
    pub shine: f32,
    pub texture_scale: (f32, f32),
    pub texture_offset: (f32, f32),
    pub texture_rotation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsData {
    pub physics_type: PhysicsType,
    pub phantom: bool,
    pub physical: bool,
    pub temporary: bool,
    pub volume_detect: bool,
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsType {
    None,
    Prim,
    Convex,
    Mesh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainData {
    pub width: u32,
    pub height: u32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub water_height: f32,
    pub heightmap: Vec<u8>,
    pub texture_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetData {
    pub uuid: String,
    pub name: String,
    pub asset_type: AssetType,
    pub data_length: u64,
    pub creator: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Sound,
    CallingCard,
    Landmark,
    Script,
    Clothing,
    Object,
    Notecard,
    Gesture,
    Animation,
    Mesh,
    Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptData {
    pub uuid: String,
    pub name: String,
    pub source_code: String,
    pub engine_type: ScriptEngineType,
    pub compiled: bool,
    pub functions_used: Vec<String>,
    pub events_handled: Vec<String>,
    pub complexity_score: f64,
    pub ossl_functions: Vec<OSSLFunctionUsage>,
    pub ossl_threat_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSSLFunctionUsage {
    pub name: String,
    pub threat_level: OSSLThreatLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptEngineType {
    LSL,
    OSSL,
    Mixed,
    CSharp,
    VB,
    JScript,
    Unknown(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OSSLThreatLevel {
    None,
    Nuisance,
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
    Severe,
}

impl OSSLThreatLevel {
    pub fn weight(&self) -> f64 {
        match self {
            OSSLThreatLevel::None => 0.0,
            OSSLThreatLevel::Nuisance => 0.1,
            OSSLThreatLevel::VeryLow => 0.2,
            OSSLThreatLevel::Low => 0.4,
            OSSLThreatLevel::Moderate => 0.6,
            OSSLThreatLevel::High => 0.8,
            OSSLThreatLevel::VeryHigh => 1.0,
            OSSLThreatLevel::Severe => 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSSLFunctionInfo {
    pub name: &'static str,
    pub threat_level: OSSLThreatLevel,
    pub description: &'static str,
}

pub const OSSL_FUNCTIONS: &[OSSLFunctionInfo] = &[
    OSSLFunctionInfo {
        name: "osGetAgentIP",
        threat_level: OSSLThreatLevel::Severe,
        description: "Get agent IP address",
    },
    OSSLFunctionInfo {
        name: "osGetSimulatorVersion",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get simulator version",
    },
    OSSLFunctionInfo {
        name: "osGetGridName",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get grid name",
    },
    OSSLFunctionInfo {
        name: "osGetGridNick",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get grid nickname",
    },
    OSSLFunctionInfo {
        name: "osGetRegionSize",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get region dimensions",
    },
    OSSLFunctionInfo {
        name: "osGetNotecard",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get notecard contents",
    },
    OSSLFunctionInfo {
        name: "osGetNotecardLine",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get notecard line",
    },
    OSSLFunctionInfo {
        name: "osGetNumberOfNotecardLines",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get notecard line count",
    },
    OSSLFunctionInfo {
        name: "osMakeNotecard",
        threat_level: OSSLThreatLevel::Low,
        description: "Create notecard",
    },
    OSSLFunctionInfo {
        name: "osKey2Name",
        threat_level: OSSLThreatLevel::Low,
        description: "Get avatar name from key",
    },
    OSSLFunctionInfo {
        name: "osOwnerSaveAppearance",
        threat_level: OSSLThreatLevel::High,
        description: "Save owner appearance",
    },
    OSSLFunctionInfo {
        name: "osAgentSaveAppearance",
        threat_level: OSSLThreatLevel::VeryHigh,
        description: "Save any agent appearance",
    },
    OSSLFunctionInfo {
        name: "osAvatarPlayAnimation",
        threat_level: OSSLThreatLevel::VeryHigh,
        description: "Play animation on avatar",
    },
    OSSLFunctionInfo {
        name: "osAvatarStopAnimation",
        threat_level: OSSLThreatLevel::VeryHigh,
        description: "Stop animation on avatar",
    },
    OSSLFunctionInfo {
        name: "osTeleportAgent",
        threat_level: OSSLThreatLevel::High,
        description: "Teleport an agent",
    },
    OSSLFunctionInfo {
        name: "osTeleportOwner",
        threat_level: OSSLThreatLevel::Low,
        description: "Teleport owner",
    },
    OSSLFunctionInfo {
        name: "osSetDynamicTextureURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set texture from URL",
    },
    OSSLFunctionInfo {
        name: "osSetDynamicTextureURLBlend",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set blended texture from URL",
    },
    OSSLFunctionInfo {
        name: "osSetDynamicTextureData",
        threat_level: OSSLThreatLevel::Low,
        description: "Set dynamic texture data",
    },
    OSSLFunctionInfo {
        name: "osGetDrawStringSize",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get draw string size",
    },
    OSSLFunctionInfo {
        name: "osDrawText",
        threat_level: OSSLThreatLevel::Low,
        description: "Draw text on texture",
    },
    OSSLFunctionInfo {
        name: "osMovePen",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Move draw pen",
    },
    OSSLFunctionInfo {
        name: "osDrawLine",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Draw line",
    },
    OSSLFunctionInfo {
        name: "osDrawFilledRectangle",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Draw filled rectangle",
    },
    OSSLFunctionInfo {
        name: "osDrawRectangle",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Draw rectangle",
    },
    OSSLFunctionInfo {
        name: "osDrawEllipse",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Draw ellipse",
    },
    OSSLFunctionInfo {
        name: "osSetFontSize",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set font size",
    },
    OSSLFunctionInfo {
        name: "osSetFontName",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set font name",
    },
    OSSLFunctionInfo {
        name: "osSetPenSize",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set pen size",
    },
    OSSLFunctionInfo {
        name: "osSetPenColor",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set pen color",
    },
    OSSLFunctionInfo {
        name: "osSetPenCap",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set pen cap style",
    },
    OSSLFunctionInfo {
        name: "osSetSpeed",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set agent movement speed",
    },
    OSSLFunctionInfo {
        name: "osKickAvatar",
        threat_level: OSSLThreatLevel::Severe,
        description: "Kick avatar from region",
    },
    OSSLFunctionInfo {
        name: "osSetParcelDetails",
        threat_level: OSSLThreatLevel::High,
        description: "Set parcel properties",
    },
    OSSLFunctionInfo {
        name: "osGetParcelDetails",
        threat_level: OSSLThreatLevel::Low,
        description: "Get parcel properties",
    },
    OSSLFunctionInfo {
        name: "osSetTerrainHeight",
        threat_level: OSSLThreatLevel::High,
        description: "Modify terrain",
    },
    OSSLFunctionInfo {
        name: "osGetTerrainHeight",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get terrain height",
    },
    OSSLFunctionInfo {
        name: "osSetRegionWaterHeight",
        threat_level: OSSLThreatLevel::High,
        description: "Set water level",
    },
    OSSLFunctionInfo {
        name: "osSetRegionSunSettings",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set sun settings",
    },
    OSSLFunctionInfo {
        name: "osConsoleCommand",
        threat_level: OSSLThreatLevel::Severe,
        description: "Execute console command",
    },
    OSSLFunctionInfo {
        name: "osRegionRestart",
        threat_level: OSSLThreatLevel::Severe,
        description: "Restart region",
    },
    OSSLFunctionInfo {
        name: "osSetParcelMediaURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set parcel media URL",
    },
    OSSLFunctionInfo {
        name: "osSetPrimMediaURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set prim media URL",
    },
    OSSLFunctionInfo {
        name: "osGetPhysicsEngineType",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get physics engine type",
    },
    OSSLFunctionInfo {
        name: "osGetPhysicsEngineName",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get physics engine name",
    },
    OSSLFunctionInfo {
        name: "osNpcCreate",
        threat_level: OSSLThreatLevel::High,
        description: "Create NPC",
    },
    OSSLFunctionInfo {
        name: "osNpcRemove",
        threat_level: OSSLThreatLevel::High,
        description: "Remove NPC",
    },
    OSSLFunctionInfo {
        name: "osNpcMoveTo",
        threat_level: OSSLThreatLevel::High,
        description: "Move NPC",
    },
    OSSLFunctionInfo {
        name: "osNpcSay",
        threat_level: OSSLThreatLevel::Low,
        description: "NPC say message",
    },
    OSSLFunctionInfo {
        name: "osNpcShout",
        threat_level: OSSLThreatLevel::Low,
        description: "NPC shout message",
    },
    OSSLFunctionInfo {
        name: "osNpcWhisper",
        threat_level: OSSLThreatLevel::Low,
        description: "NPC whisper message",
    },
    OSSLFunctionInfo {
        name: "osNpcPlayAnimation",
        threat_level: OSSLThreatLevel::Moderate,
        description: "NPC play animation",
    },
    OSSLFunctionInfo {
        name: "osNpcStopAnimation",
        threat_level: OSSLThreatLevel::Moderate,
        description: "NPC stop animation",
    },
    OSSLFunctionInfo {
        name: "osNpcGetOwner",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get NPC owner",
    },
    OSSLFunctionInfo {
        name: "osNpcSetProfileImage",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set NPC profile image",
    },
    OSSLFunctionInfo {
        name: "osNpcSetProfileAbout",
        threat_level: OSSLThreatLevel::Low,
        description: "Set NPC profile about",
    },
    OSSLFunctionInfo {
        name: "osMessageObject",
        threat_level: OSSLThreatLevel::Low,
        description: "Send message to object",
    },
    OSSLFunctionInfo {
        name: "osSetObjectDescription",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set object description",
    },
    OSSLFunctionInfo {
        name: "osSetObjectName",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set object name",
    },
    OSSLFunctionInfo {
        name: "osGetAvatarList",
        threat_level: OSSLThreatLevel::Low,
        description: "Get list of avatars",
    },
    OSSLFunctionInfo {
        name: "osGetAgents",
        threat_level: OSSLThreatLevel::Low,
        description: "Get list of agents",
    },
    OSSLFunctionInfo {
        name: "osGetAvatarHomeURI",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Get avatar home grid URI",
    },
    OSSLFunctionInfo {
        name: "osFormatString",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Format string",
    },
    OSSLFunctionInfo {
        name: "osMatchString",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Match string pattern",
    },
    OSSLFunctionInfo {
        name: "osReplaceString",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Replace in string",
    },
    OSSLFunctionInfo {
        name: "osLoadedCreationDate",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get creation date",
    },
    OSSLFunctionInfo {
        name: "osLoadedCreationID",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get creation ID",
    },
    OSSLFunctionInfo {
        name: "osLoadedCreationTime",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get creation time",
    },
    OSSLFunctionInfo {
        name: "osGetLinkPrimitiveParams",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get link prim params",
    },
    OSSLFunctionInfo {
        name: "osGetPrimitiveParams",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get prim params",
    },
    OSSLFunctionInfo {
        name: "osSetPrimitiveParams",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set prim params",
    },
    OSSLFunctionInfo {
        name: "osGetWindParam",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Get wind parameter",
    },
    OSSLFunctionInfo {
        name: "osSetWindParam",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set wind parameter",
    },
    OSSLFunctionInfo {
        name: "osParcelJoin",
        threat_level: OSSLThreatLevel::High,
        description: "Join parcels",
    },
    OSSLFunctionInfo {
        name: "osParcelSubdivide",
        threat_level: OSSLThreatLevel::High,
        description: "Subdivide parcel",
    },
    OSSLFunctionInfo {
        name: "osSetParcelMusicURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Set parcel music URL",
    },
    OSSLFunctionInfo {
        name: "osVolumeDetect",
        threat_level: OSSLThreatLevel::VeryLow,
        description: "Set volume detect",
    },
    OSSLFunctionInfo {
        name: "osRequestURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Request external URL",
    },
    OSSLFunctionInfo {
        name: "osRequestSecureURL",
        threat_level: OSSLThreatLevel::Moderate,
        description: "Request secure external URL",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelData {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub owner_uuid: String,
    pub area: u32,
    pub bounds: ((f32, f32), (f32, f32)),
    pub flags: ParcelFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParcelFlags {
    pub allow_fly: bool,
    pub allow_scripts: bool,
    pub allow_create: bool,
    pub allow_damage: bool,
    pub for_sale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPattern {
    pub id: String,
    pub name: String,
    pub frequency: u32,
    pub quality_score: f64,
    pub characteristics: Vec<String>,
    pub example_objects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub total_oars_analyzed: u32,
    pub patterns_recognized: u32,
    pub pattern_accuracy: f64,
    pub improvement_rate: f64,
    pub last_learning_session: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub deep_analysis: bool,
    pub script_analysis: bool,
    pub pattern_learning: bool,
    pub quality_assessment: bool,
    pub performance_metrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstateSettings {
    pub estate_name: String,
    pub estate_owner: String,
    pub public_access: bool,
    pub voice_enabled: bool,
    pub tax_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectMetadata {
    pub creator: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub creation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub land_impact: u32,
    pub script_count: u32,
}

impl OARAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            pattern_database: HashMap::new(),
            learning_metrics: LearningMetrics {
                total_oars_analyzed: 0,
                patterns_recognized: 0,
                pattern_accuracy: 0.0,
                improvement_rate: 0.0,
                last_learning_session: None,
            },
            config,
        }
    }

    /// Analyze an OAR file and extract comprehensive data
    pub async fn analyze_oar_file(&mut self, oar_path: &Path) -> Result<OARData> {
        tracing::info!("Starting OAR analysis: {}", oar_path.display());

        // Open and read the OAR archive
        let file = File::open(oar_path)
            .with_context(|| format!("Failed to open OAR file: {}", oar_path.display()))?;
        let mut archive = ZipArchive::new(file).with_context(|| "Failed to read OAR archive")?;

        // Extract metadata
        let metadata = self.extract_metadata(&mut archive).await?;

        // Extract region information
        let region_info = self.extract_region_info(&mut archive).await?;

        // Analyze objects and primitives
        let objects = self.analyze_objects(&mut archive).await?;

        // Extract terrain data
        let terrain = self.extract_terrain_data(&mut archive).await?;

        // Analyze assets
        let assets = self.analyze_assets(&mut archive).await?;

        // Analyze scripts
        let scripts = if self.config.script_analysis {
            self.analyze_scripts(&mut archive).await?
        } else {
            Vec::new()
        };

        // Extract parcel information
        let parcels = self.extract_parcel_data(&mut archive).await?;

        let oar_data = OARData {
            metadata,
            region_info,
            objects,
            terrain,
            assets,
            scripts,
            parcels,
        };

        // Perform pattern learning if enabled
        if self.config.pattern_learning {
            self.learn_patterns_from_data(&oar_data).await?;
        }

        // Update learning metrics
        self.learning_metrics.total_oars_analyzed += 1;
        self.learning_metrics.last_learning_session = Some(chrono::Utc::now());

        tracing::info!("OAR analysis completed successfully");
        Ok(oar_data)
    }

    /// Convert analyzed OAR data into content patterns for the AI system
    pub fn extract_content_patterns(&self, oar_data: &OARData) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();

        // Analyze architectural patterns
        let architectural_patterns = self.analyze_architectural_patterns(&oar_data.objects)?;
        patterns.extend(architectural_patterns);

        // Analyze spatial relationship patterns
        let spatial_patterns = self.analyze_spatial_patterns(&oar_data.objects)?;
        patterns.extend(spatial_patterns);

        // Analyze material usage patterns
        let material_patterns = self.analyze_material_patterns(&oar_data.objects)?;
        patterns.extend(material_patterns);

        // Analyze scripting patterns
        let scripting_patterns = self.analyze_scripting_patterns(&oar_data.scripts)?;
        patterns.extend(scripting_patterns);

        // Analyze landscape patterns
        if let Some(terrain) = &oar_data.terrain {
            let landscape_patterns = self.analyze_landscape_patterns(terrain)?;
            patterns.extend(landscape_patterns);
        }

        tracing::info!(
            "Extracted {} content patterns from OAR data",
            patterns.len()
        );
        Ok(patterns)
    }

    /// Generate quality assessment for the analyzed content
    pub fn assess_quality(&self, oar_data: &OARData) -> Result<QualityAssessment> {
        let mut assessment = QualityAssessment::new();

        // Assess object quality
        assessment.object_quality = self.assess_object_quality(&oar_data.objects)?;

        // Assess script quality
        assessment.script_quality = self.assess_script_quality(&oar_data.scripts)?;

        // Assess material usage
        assessment.material_quality = self.assess_material_quality(&oar_data.objects)?;

        // Assess spatial organization
        assessment.spatial_quality = self.assess_spatial_quality(&oar_data.objects)?;

        // Calculate overall quality score
        assessment.overall_score = (assessment.object_quality * 0.3
            + assessment.script_quality * 0.25
            + assessment.material_quality * 0.25
            + assessment.spatial_quality * 0.2);

        Ok(assessment)
    }

    // Private implementation methods
    async fn extract_metadata<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<OARMetadata> {
        // Look for archive.xml or similar metadata files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("archive.xml") || file.name().contains("metadata") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                return self.parse_metadata_xml(&contents);
            }
        }

        // If no metadata file found, create default metadata
        Ok(OARMetadata {
            version: "Unknown".to_string(),
            created_date: None,
            creator: None,
            region_name: "Unknown Region".to_string(),
            region_uuid: Uuid::new_v4().to_string(),
            total_objects: 0,
            total_prims: 0,
        })
    }

    async fn extract_region_info<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<RegionInfo> {
        // Look for region settings files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("settings") || file.name().contains("region") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                return self.parse_region_settings(&contents);
            }
        }

        // Default region info
        Ok(RegionInfo {
            name: "Unknown Region".to_string(),
            uuid: Uuid::new_v4().to_string(),
            size: (256, 256),
            water_height: 20.0,
            terrain_multipliers: (1.0, 1.0, 1.0, 1.0),
            estate_settings: None,
        })
    }

    async fn analyze_objects<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<AnalyzedObject>> {
        let mut objects = Vec::new();

        // Look for object files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("objects/") && file.name().ends_with(".xml") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                if let Ok(object) = self.parse_object_xml(&contents) {
                    objects.push(object);
                }
            }
        }

        tracing::info!("Analyzed {} objects from OAR", objects.len());
        Ok(objects)
    }

    async fn extract_terrain_data<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Option<TerrainData>> {
        // Look for terrain files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("terrain") {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;
                return Ok(Some(self.parse_terrain_data(&contents)?));
            }
        }
        Ok(None)
    }

    async fn analyze_assets<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<AssetData>> {
        let mut assets = Vec::new();

        // Look for asset files
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            if file.name().contains("assets/") {
                let asset = AssetData {
                    uuid: Uuid::new_v4().to_string(),
                    name: file.name().to_string(),
                    asset_type: self.determine_asset_type(file.name()),
                    data_length: file.size(),
                    creator: None,
                    description: None,
                };
                assets.push(asset);
            }
        }

        tracing::info!("Found {} assets in OAR", assets.len());
        Ok(assets)
    }

    async fn analyze_scripts<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<ScriptData>> {
        let mut scripts = Vec::new();

        // Look for script files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("scripts/") && file.name().ends_with(".lsl") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let script = self.analyze_script_content(&contents, file.name())?;
                scripts.push(script);
            }
        }

        tracing::info!("Analyzed {} scripts from OAR", scripts.len());
        Ok(scripts)
    }

    async fn extract_parcel_data<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<Vec<ParcelData>> {
        let mut parcels = Vec::new();

        // Look for parcel/land files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().contains("parcels") || file.name().contains("land") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                if let Ok(parcel_list) = self.parse_parcel_data(&contents) {
                    parcels.extend(parcel_list);
                }
            }
        }

        Ok(parcels)
    }

    async fn learn_patterns_from_data(&mut self, oar_data: &OARData) -> Result<()> {
        // EADS-style pattern learning implementation

        // Learn architectural patterns
        self.learn_architectural_patterns(&oar_data.objects).await?;

        // Learn material usage patterns
        self.learn_material_patterns(&oar_data.objects).await?;

        // Learn spatial organization patterns
        self.learn_spatial_patterns(&oar_data.objects).await?;

        // Learn scripting patterns
        self.learn_scripting_patterns(&oar_data.scripts).await?;

        self.learning_metrics.patterns_recognized += 1;
        tracing::info!("Pattern learning completed");
        Ok(())
    }

    // Helper methods for parsing and analysis
    fn parse_metadata_xml(&self, xml_content: &str) -> Result<OARMetadata> {
        let version = Self::extract_xml_value(xml_content, "version").unwrap_or("1.0".to_string());
        let region_name =
            Self::extract_xml_value(xml_content, "region_name").unwrap_or("Unknown".to_string());
        let region_uuid = Self::extract_xml_value(xml_content, "region_uuid")
            .unwrap_or(Uuid::new_v4().to_string());
        let total_objects = Self::extract_xml_value(xml_content, "object_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let total_prims = Self::extract_xml_value(xml_content, "prim_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(OARMetadata {
            version,
            created_date: Some(chrono::Utc::now()),
            creator: Self::extract_xml_value(xml_content, "creator"),
            region_name,
            region_uuid,
            total_objects,
            total_prims,
        })
    }

    fn parse_region_settings(&self, settings_content: &str) -> Result<RegionInfo> {
        let name = Self::extract_xml_value(settings_content, "RegionName")
            .unwrap_or("Unknown".to_string());
        let uuid = Self::extract_xml_value(settings_content, "RegionUUID")
            .unwrap_or(Uuid::new_v4().to_string());
        let water_height = Self::extract_xml_value(settings_content, "WaterHeight")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20.0);

        Ok(RegionInfo {
            name,
            uuid,
            size: (256, 256),
            water_height,
            terrain_multipliers: (1.0, 1.0, 1.0, 1.0),
            estate_settings: None,
        })
    }

    fn parse_object_xml(&self, xml_content: &str) -> Result<AnalyzedObject> {
        let uuid =
            Self::extract_xml_value(xml_content, "UUID").unwrap_or(Uuid::new_v4().to_string());
        let name = Self::extract_xml_value(xml_content, "Name").unwrap_or("Object".to_string());
        let description = Self::extract_xml_value(xml_content, "Description").unwrap_or_default();

        Ok(AnalyzedObject {
            uuid,
            name,
            description,
            position: (128.0, 128.0, 25.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
            primitive_type: PrimitiveType::Box,
            material_data: MaterialData {
                texture_id: None,
                color: (1.0, 1.0, 1.0, 1.0),
                alpha: 1.0,
                glow: 0.0,
                shine: 0.0,
                texture_scale: (1.0, 1.0),
                texture_offset: (0.0, 0.0),
                texture_rotation: 0.0,
            },
            physics_data: PhysicsData {
                physics_type: PhysicsType::Convex,
                phantom: false,
                physical: false,
                temporary: false,
                volume_detect: false,
                density: 1000.0,
                friction: 0.5,
                restitution: 0.3,
            },
            scripts: Vec::new(),
            children: Vec::new(),
            metadata: ObjectMetadata::default(),
        })
    }

    fn parse_terrain_data(&self, terrain_data: &[u8]) -> Result<TerrainData> {
        let size = (terrain_data.len() as f64).sqrt() as u32;
        Ok(TerrainData {
            width: size.max(256),
            height: size.max(256),
            min_elevation: 0.0,
            max_elevation: 100.0,
            water_height: 20.0,
            heightmap: terrain_data.to_vec(),
            texture_ids: vec![Uuid::nil().to_string(); 4],
        })
    }

    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);
        if let Some(start) = xml.find(&start_tag) {
            let value_start = start + start_tag.len();
            if let Some(end) = xml[value_start..].find(&end_tag) {
                return Some(xml[value_start..value_start + end].to_string());
            }
        }
        None
    }

    fn determine_asset_type(&self, filename: &str) -> AssetType {
        if filename.ends_with(".jpg") || filename.ends_with(".png") || filename.ends_with(".tga") {
            AssetType::Texture
        } else if filename.ends_with(".wav") || filename.ends_with(".ogg") {
            AssetType::Sound
        } else if filename.ends_with(".lsl") {
            AssetType::Script
        } else {
            AssetType::Object
        }
    }

    fn analyze_script_content(&self, script_content: &str, filename: &str) -> Result<ScriptData> {
        let engine_type = self.detect_script_language(script_content);
        let functions_used = self.extract_functions_from_script(script_content);
        let events_handled = self.extract_events_from_script(script_content);

        let ossl_raw = self.extract_ossl_functions(script_content);
        let ossl_threat_score = self.calculate_ossl_threat_score(&ossl_raw);

        let ossl_functions: Vec<OSSLFunctionUsage> = ossl_raw
            .iter()
            .filter_map(|(name, level)| {
                self.get_ossl_function_info(name)
                    .map(|info| OSSLFunctionUsage {
                        name: name.clone(),
                        threat_level: *level,
                        description: info.description.to_string(),
                    })
            })
            .collect();

        let complexity_score =
            self.calculate_script_complexity_with_ossl(script_content, ossl_threat_score);

        Ok(ScriptData {
            uuid: Uuid::new_v4().to_string(),
            name: filename.to_string(),
            source_code: script_content.to_string(),
            engine_type,
            compiled: false,
            functions_used,
            events_handled,
            complexity_score,
            ossl_functions,
            ossl_threat_score,
        })
    }

    fn parse_parcel_data(&self, parcel_content: &str) -> Result<Vec<ParcelData>> {
        let mut parcels = Vec::new();
        let parcel_name =
            Self::extract_xml_value(parcel_content, "Name").unwrap_or("Default Parcel".to_string());
        let parcel_desc =
            Self::extract_xml_value(parcel_content, "Description").unwrap_or_default();

        parcels.push(ParcelData {
            uuid: Uuid::new_v4().to_string(),
            name: parcel_name,
            description: parcel_desc,
            owner_uuid: Uuid::nil().to_string(),
            area: 65536,
            bounds: ((0.0, 0.0), (256.0, 256.0)),
            flags: ParcelFlags::default(),
        });
        Ok(parcels)
    }

    // Pattern analysis methods
    fn analyze_architectural_patterns(
        &self,
        objects: &[AnalyzedObject],
    ) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();
        if objects.is_empty() {
            return Ok(patterns);
        }

        let pattern = ContentPattern {
            id: Uuid::new_v4(),
            name: "architectural_pattern".to_string(),
            category: ContentCategory::Architecture,
            recognition_data: RecognitionData {
                geometry: GeometricAnalysis {
                    primitive_types: objects.iter().map(|o| o.primitive_type.clone()).collect(),
                    scale_analysis: crate::ai::content_creation::ScaleAnalysis,
                    symmetry_patterns: Vec::new(),
                    complexity_score: objects.len() as f64 * 0.1,
                },
                materials: MaterialAnalysis,
                spatial_relations: SpatialAnalysis,
                scripting_patterns: Vec::new(),
            },
            construction_methods: Vec::new(),
            usage_stats: UsageStatistics,
            attribution: CreatorAttribution,
        };
        patterns.push(pattern);
        Ok(patterns)
    }

    fn analyze_spatial_patterns(&self, objects: &[AnalyzedObject]) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();
        if objects.len() < 2 {
            return Ok(patterns);
        }

        let pattern = ContentPattern {
            id: Uuid::new_v4(),
            name: "spatial_arrangement".to_string(),
            category: ContentCategory::Environments,
            recognition_data: RecognitionData {
                geometry: GeometricAnalysis {
                    primitive_types: Vec::new(),
                    scale_analysis: crate::ai::content_creation::ScaleAnalysis,
                    symmetry_patterns: Vec::new(),
                    complexity_score: objects.len() as f64 * 0.05,
                },
                materials: MaterialAnalysis,
                spatial_relations: SpatialAnalysis,
                scripting_patterns: Vec::new(),
            },
            construction_methods: Vec::new(),
            usage_stats: UsageStatistics,
            attribution: CreatorAttribution,
        };
        patterns.push(pattern);
        Ok(patterns)
    }

    fn analyze_material_patterns(&self, objects: &[AnalyzedObject]) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();
        let textured_count = objects
            .iter()
            .filter(|o| o.material_data.texture_id.is_some())
            .count();

        if textured_count > 0 {
            let pattern = ContentPattern {
                id: Uuid::new_v4(),
                name: "material_usage".to_string(),
                category: ContentCategory::Primitives,
                recognition_data: RecognitionData {
                    geometry: GeometricAnalysis {
                        primitive_types: Vec::new(),
                        scale_analysis: crate::ai::content_creation::ScaleAnalysis,
                        symmetry_patterns: Vec::new(),
                        complexity_score: textured_count as f64 * 0.15,
                    },
                    materials: MaterialAnalysis,
                    spatial_relations: SpatialAnalysis,
                    scripting_patterns: Vec::new(),
                },
                construction_methods: Vec::new(),
                usage_stats: UsageStatistics,
                attribution: CreatorAttribution,
            };
            patterns.push(pattern);
        }
        Ok(patterns)
    }

    fn analyze_scripting_patterns(&self, scripts: &[ScriptData]) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();
        if scripts.is_empty() {
            return Ok(patterns);
        }

        let pattern = ContentPattern {
            id: Uuid::new_v4(),
            name: "scripting_behavior".to_string(),
            category: ContentCategory::Interactive,
            recognition_data: RecognitionData {
                geometry: GeometricAnalysis {
                    primitive_types: Vec::new(),
                    scale_analysis: crate::ai::content_creation::ScaleAnalysis,
                    symmetry_patterns: Vec::new(),
                    complexity_score: scripts.iter().map(|s| s.complexity_score).sum::<f64>()
                        / scripts.len() as f64,
                },
                materials: MaterialAnalysis,
                spatial_relations: SpatialAnalysis,
                scripting_patterns: scripts.iter().map(|s| ScriptingPattern).collect(),
            },
            construction_methods: Vec::new(),
            usage_stats: UsageStatistics,
            attribution: CreatorAttribution,
        };
        patterns.push(pattern);
        Ok(patterns)
    }

    fn analyze_landscape_patterns(&self, terrain: &TerrainData) -> Result<Vec<ContentPattern>> {
        let mut patterns = Vec::new();
        let elevation_range = terrain.max_elevation - terrain.min_elevation;

        let pattern = ContentPattern {
            id: Uuid::new_v4(),
            name: "terrain_profile".to_string(),
            category: ContentCategory::Landscape,
            recognition_data: RecognitionData {
                geometry: GeometricAnalysis {
                    primitive_types: Vec::new(),
                    scale_analysis: crate::ai::content_creation::ScaleAnalysis,
                    symmetry_patterns: Vec::new(),
                    complexity_score: elevation_range as f64 * 0.01,
                },
                materials: MaterialAnalysis,
                spatial_relations: SpatialAnalysis,
                scripting_patterns: Vec::new(),
            },
            construction_methods: Vec::new(),
            usage_stats: UsageStatistics,
            attribution: CreatorAttribution,
        };
        patterns.push(pattern);
        Ok(patterns)
    }

    // Quality assessment methods
    fn assess_object_quality(&self, objects: &[AnalyzedObject]) -> Result<f64> {
        if objects.is_empty() {
            return Ok(5.0);
        }
        let complexity_factor = (objects.len() as f64).log2().min(3.0);
        let has_scripts = objects.iter().any(|o| !o.scripts.is_empty());
        let has_textures = objects.iter().any(|o| o.material_data.texture_id.is_some());
        let base_score = 6.0;
        let script_bonus = if has_scripts { 1.5 } else { 0.0 };
        let texture_bonus = if has_textures { 1.0 } else { 0.0 };
        Ok((base_score + complexity_factor + script_bonus + texture_bonus).min(10.0))
    }

    fn assess_script_quality(&self, scripts: &[ScriptData]) -> Result<f64> {
        if scripts.is_empty() {
            return Ok(5.0);
        }
        let avg_complexity: f64 =
            scripts.iter().map(|s| s.complexity_score).sum::<f64>() / scripts.len() as f64;
        let base_score = 5.0 + avg_complexity.min(3.0);
        let function_bonus = scripts
            .iter()
            .filter(|s| !s.functions_used.is_empty())
            .count() as f64
            * 0.5;
        Ok((base_score + function_bonus).min(10.0))
    }

    fn assess_material_quality(&self, objects: &[AnalyzedObject]) -> Result<f64> {
        if objects.is_empty() {
            return Ok(5.0);
        }
        let textured_ratio = objects
            .iter()
            .filter(|o| o.material_data.texture_id.is_some())
            .count() as f64
            / objects.len() as f64;
        let has_glow = objects.iter().any(|o| o.material_data.glow > 0.0);
        let base_score = 5.0 + textured_ratio * 3.0;
        let glow_bonus = if has_glow { 1.0 } else { 0.0 };
        Ok((base_score + glow_bonus).min(10.0))
    }

    fn assess_spatial_quality(&self, objects: &[AnalyzedObject]) -> Result<f64> {
        if objects.len() < 2 {
            return Ok(5.0);
        }
        let has_hierarchy = objects.iter().any(|o| !o.children.is_empty());
        let variety = objects
            .iter()
            .map(|o| std::mem::discriminant(&o.primitive_type))
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;
        let base_score = 5.0 + variety.min(3.0);
        let hierarchy_bonus = if has_hierarchy { 2.0 } else { 0.0 };
        Ok((base_score + hierarchy_bonus).min(10.0))
    }

    // Learning methods
    async fn learn_architectural_patterns(&mut self, objects: &[AnalyzedObject]) -> Result<()> {
        // Architectural pattern learning
        Ok(())
    }

    async fn learn_material_patterns(&mut self, objects: &[AnalyzedObject]) -> Result<()> {
        // Material pattern learning
        Ok(())
    }

    async fn learn_spatial_patterns(&mut self, objects: &[AnalyzedObject]) -> Result<()> {
        // Spatial pattern learning
        Ok(())
    }

    async fn learn_scripting_patterns(&mut self, scripts: &[ScriptData]) -> Result<()> {
        // Scripting pattern learning
        Ok(())
    }

    fn detect_script_language(&self, script_content: &str) -> ScriptEngineType {
        let has_ossl = OSSL_FUNCTIONS
            .iter()
            .any(|f| script_content.contains(f.name));
        let has_lsl = self.has_lsl_functions(script_content);

        match (has_lsl, has_ossl) {
            (true, true) => ScriptEngineType::Mixed,
            (false, true) => ScriptEngineType::OSSL,
            (true, false) => ScriptEngineType::LSL,
            (false, false) => ScriptEngineType::LSL,
        }
    }

    fn has_lsl_functions(&self, script_content: &str) -> bool {
        let lsl_indicators = [
            "llSay",
            "llWhisper",
            "llShout",
            "llListen",
            "llSetText",
            "llGetKey",
            "llGetOwner",
            "llRequestPermissions",
            "llDialog",
            "llSetTimerEvent",
            "llSensor",
            "llRezObject",
        ];
        lsl_indicators.iter().any(|f| script_content.contains(f))
    }

    fn extract_ossl_functions(&self, script_content: &str) -> Vec<(String, OSSLThreatLevel)> {
        let mut ossl_functions = Vec::new();
        for func_info in OSSL_FUNCTIONS {
            if script_content.contains(func_info.name) {
                ossl_functions.push((func_info.name.to_string(), func_info.threat_level));
            }
        }
        ossl_functions
    }

    fn calculate_ossl_threat_score(&self, ossl_functions: &[(String, OSSLThreatLevel)]) -> f64 {
        if ossl_functions.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = ossl_functions.iter().map(|(_, level)| level.weight()).sum();

        let max_threat = ossl_functions
            .iter()
            .map(|(_, level)| level.weight())
            .fold(0.0f64, |a, b| a.max(b));

        (total_weight + max_threat * 2.0).min(10.0)
    }

    fn get_ossl_function_info(&self, name: &str) -> Option<&'static OSSLFunctionInfo> {
        OSSL_FUNCTIONS.iter().find(|f| f.name == name)
    }

    fn extract_functions_from_script(&self, script_content: &str) -> Vec<String> {
        let mut functions = Vec::new();
        let common_lsl_functions = [
            "llSay",
            "llWhisper",
            "llShout",
            "llRegionSay",
            "llOwnerSay",
            "llSetText",
            "llSetAlpha",
            "llSetColor",
            "llSetTexture",
            "llSetScale",
            "llSetPos",
            "llSetRot",
            "llSetPrimitiveParams",
            "llGetPos",
            "llGetRot",
            "llGetKey",
            "llGetOwner",
            "llGetLinkKey",
            "llGetObjectName",
            "llListen",
            "llListenRemove",
            "llDialog",
            "llTextBox",
            "llGiveInventory",
            "llRezObject",
            "llDie",
            "llDetach",
            "llRequestPermissions",
            "llTakeControls",
            "llReleaseControls",
            "llSensor",
            "llSensorRepeat",
            "llSensorRemove",
            "llSetTimerEvent",
            "llSleep",
            "llResetScript",
            "llApplyImpulse",
            "llApplyRotationalImpulse",
            "llMoveToTarget",
            "llStartAnimation",
            "llStopAnimation",
            "llGetAnimation",
            "llPlaySound",
            "llLoopSound",
            "llStopSound",
            "llTriggerSound",
            "llHTTPRequest",
            "llEmail",
            "llGetNotecardLine",
            "llGetNumberOfNotecardLines",
            "llStringLength",
            "llSubStringIndex",
            "llDeleteSubString",
            "llGetSubString",
            "llList2String",
            "llList2Integer",
            "llList2Float",
            "llListFindList",
            "llGetListLength",
        ];

        for func in &common_lsl_functions {
            if script_content.contains(func) {
                functions.push(func.to_string());
            }
        }
        functions
    }

    fn extract_events_from_script(&self, script_content: &str) -> Vec<String> {
        let mut events = Vec::new();
        let lsl_events = [
            "state_entry",
            "state_exit",
            "touch_start",
            "touch",
            "touch_end",
            "collision_start",
            "collision",
            "collision_end",
            "sensor",
            "no_sensor",
            "timer",
            "listen",
            "at_target",
            "not_at_target",
            "control",
            "money",
            "email",
            "run_time_permissions",
            "changed",
            "on_rez",
            "object_rez",
            "attach",
            "dataserver",
            "moving_start",
            "moving_end",
            "link_message",
            "http_response",
            "land_collision_start",
            "land_collision",
            "land_collision_end",
            "experience_permissions",
            "experience_permissions_denied",
        ];

        for event in &lsl_events {
            let event_pattern = format!("{}(", event);
            if script_content.contains(&event_pattern) {
                events.push(event.to_string());
            }
        }
        events
    }

    fn calculate_script_complexity(&self, script_content: &str) -> f64 {
        let line_count = script_content.lines().count();
        let function_count = self.extract_functions_from_script(script_content).len();
        let event_count = self.extract_events_from_script(script_content).len();

        let nesting_depth = script_content.matches('{').count().min(20) as f64;
        let conditional_count = script_content.matches("if").count()
            + script_content.matches("while").count()
            + script_content.matches("for").count();

        let line_factor = (line_count as f64 * 0.02).min(2.0);
        let function_factor = (function_count as f64 * 0.1).min(3.0);
        let event_factor = (event_count as f64 * 0.15).min(2.0);
        let nesting_factor = (nesting_depth * 0.05).min(1.0);
        let conditional_factor = (conditional_count as f64 * 0.05).min(2.0);

        (line_factor + function_factor + event_factor + nesting_factor + conditional_factor)
            .min(10.0)
    }

    fn calculate_script_complexity_with_ossl(
        &self,
        script_content: &str,
        ossl_threat_score: f64,
    ) -> f64 {
        let base_complexity = self.calculate_script_complexity(script_content);
        let ossl_factor = ossl_threat_score * 0.2;
        (base_complexity + ossl_factor).min(10.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub object_quality: f64,
    pub script_quality: f64,
    pub material_quality: f64,
    pub spatial_quality: f64,
    pub overall_score: f64,
}

impl QualityAssessment {
    pub fn new() -> Self {
        Self {
            object_quality: 0.0,
            script_quality: 0.0,
            material_quality: 0.0,
            spatial_quality: 0.0,
            overall_score: 0.0,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            deep_analysis: true,
            script_analysis: true,
            pattern_learning: true,
            quality_assessment: true,
            performance_metrics: true,
        }
    }
}
