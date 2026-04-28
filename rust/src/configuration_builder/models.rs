use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentType {
    Native,
    Docker,
    Kubernetes,
}

impl Default for DeploymentType {
    fn default() -> Self {
        DeploymentType::Native
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Draft,
    Ready,
    Deployed,
    Failed,
}

impl Default for DeploymentStatus {
    fn default() -> Self {
        DeploymentStatus::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SimulatorType {
    Mainland,
    Island,
    Marina,
    Sandbox,
    Welcome,
    Event,
    Shopping,
    Roleplay,
    Residential,
    Void,
    CustomTerrain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OsslThreatLevel {
    None,
    Nuisance,
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
    Severe,
}

impl Default for OsslThreatLevel {
    fn default() -> Self {
        OsslThreatLevel::Low
    }
}

impl OsslThreatLevel {
    pub fn to_ini_string(&self) -> &'static str {
        match self {
            OsslThreatLevel::None => "None",
            OsslThreatLevel::Nuisance => "Nuisance",
            OsslThreatLevel::VeryLow => "VeryLow",
            OsslThreatLevel::Low => "Low",
            OsslThreatLevel::Moderate => "Moderate",
            OsslThreatLevel::High => "High",
            OsslThreatLevel::VeryHigh => "VeryHigh",
            OsslThreatLevel::Severe => "Severe",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsEngine {
    UbOde,
    BulletSim,
    BasicPhysics,
    PosPlugin,
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        PhysicsEngine::UbOde
    }
}

impl PhysicsEngine {
    pub fn to_ini_string(&self) -> &'static str {
        match self {
            PhysicsEngine::UbOde => "ubODE",
            PhysicsEngine::BulletSim => "BulletSim",
            PhysicsEngine::BasicPhysics => "BasicPhysics",
            PhysicsEngine::PosPlugin => "POS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseProvider {
    Sqlite,
    Postgresql,
    Mysql,
    Mariadb,
}

impl Default for DatabaseProvider {
    fn default() -> Self {
        DatabaseProvider::Sqlite
    }
}

impl DatabaseProvider {
    pub fn to_ini_string(&self) -> &'static str {
        match self {
            DatabaseProvider::Sqlite => "SQLite",
            DatabaseProvider::Postgresql => "Pgsql",
            DatabaseProvider::Mysql => "MySQL",
            DatabaseProvider::Mariadb => "MySQL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemRequirements {
    pub min_memory_mb: i32,
    pub recommended_memory_mb: i32,
    pub min_cpu_cores: i32,
    pub recommended_cpu_cores: i32,
    pub network_bandwidth_mbps: i32,
    pub disk_space_gb: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainerConfig {
    #[serde(default)]
    pub deployment_type: DeploymentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker_image: Option<String>,
    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: i32,
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit: f64,
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(default = "default_replicas")]
    pub replicas: i32,
    #[serde(default)]
    pub enable_hpa: bool,
    #[serde(default = "default_min_replicas")]
    pub min_replicas: i32,
    #[serde(default = "default_max_replicas")]
    pub max_replicas: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress_host: Option<String>,
    #[serde(default)]
    pub enable_tls: bool,
}

fn default_memory_limit() -> i32 {
    2048
}
fn default_cpu_limit() -> f64 {
    2.0
}
fn default_replicas() -> i32 {
    1
}
fn default_min_replicas() -> i32 {
    1
}
fn default_max_replicas() -> i32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSimIniConfig {
    pub grid_name: String,
    pub welcome_message: String,
    #[serde(default)]
    pub allow_anonymous_login: bool,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    pub external_host_name: String,
    #[serde(default = "default_internal_ip")]
    pub internal_ip: String,
    #[serde(default)]
    pub database_provider: DatabaseProvider,
    pub connection_string: String,
    #[serde(default)]
    pub physics_engine: PhysicsEngine,
    #[serde(default)]
    pub enable_voice: bool,
    #[serde(default)]
    pub enable_search: bool,
    #[serde(default)]
    pub enable_currency: bool,
    #[serde(default)]
    pub additional_sections: HashMap<String, HashMap<String, String>>,
}

fn default_http_port() -> u16 {
    9000
}
fn default_internal_ip() -> String {
    "0.0.0.0".to_string()
}

impl Default for OpenSimIniConfig {
    fn default() -> Self {
        Self {
            grid_name: "OpenSim Next".to_string(),
            welcome_message: "Welcome to OpenSim Next!".to_string(),
            allow_anonymous_login: false,
            http_port: 9000,
            external_host_name: "localhost".to_string(),
            internal_ip: "0.0.0.0".to_string(),
            database_provider: DatabaseProvider::Sqlite,
            connection_string: "Data Source=opensim.db;Version=3".to_string(),
            physics_engine: PhysicsEngine::UbOde,
            enable_voice: false,
            enable_search: true,
            enable_currency: false,
            additional_sections: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionIniConfig {
    pub region_name: String,
    pub region_uuid: String,
    #[serde(default = "default_location")]
    pub location_x: i32,
    #[serde(default = "default_location")]
    pub location_y: i32,
    #[serde(default = "default_region_size")]
    pub size_x: i32,
    #[serde(default = "default_region_size")]
    pub size_y: i32,
    #[serde(default = "default_http_port")]
    pub internal_port: u16,
    #[serde(default = "default_max_agents")]
    pub max_agents: i32,
    #[serde(default = "default_max_prims")]
    pub max_prims: i32,
    pub estate_name: String,
    pub estate_owner: String,
}

fn default_location() -> i32 {
    1000
}
fn default_region_size() -> i32 {
    256
}
fn default_max_agents() -> i32 {
    40
}
fn default_max_prims() -> i32 {
    45000
}

impl Default for RegionIniConfig {
    fn default() -> Self {
        Self {
            region_name: "New Region".to_string(),
            region_uuid: uuid::Uuid::new_v4().to_string(),
            location_x: 1000,
            location_y: 1000,
            size_x: 256,
            size_y: 256,
            internal_port: 9000,
            max_agents: 40,
            max_prims: 45000,
            estate_name: "My Estate".to_string(),
            estate_owner: "Admin".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsslConfig {
    #[serde(default)]
    pub default_threat_level: OsslThreatLevel,
    #[serde(default)]
    pub allowed_functions: Vec<String>,
    #[serde(default)]
    pub blocked_functions: Vec<String>,
    #[serde(default)]
    pub enable_npc: bool,
    #[serde(default)]
    pub enable_teleport: bool,
    #[serde(default = "default_true")]
    pub enable_dynamic_textures: bool,
    #[serde(default = "default_true")]
    pub enable_json_store: bool,
}

fn default_true() -> bool {
    true
}

impl Default for OsslConfig {
    fn default() -> Self {
        Self {
            default_threat_level: OsslThreatLevel::Low,
            allowed_functions: Vec::new(),
            blocked_functions: Vec::new(),
            enable_npc: false,
            enable_teleport: false,
            enable_dynamic_textures: true,
            enable_json_store: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub template_type: SimulatorType,
    pub opensim_ini: OpenSimIniConfig,
    pub region_ini: RegionIniConfig,
    pub ossl_config: OsslConfig,
    #[serde(default)]
    pub config_includes: HashMap<String, String>,
    pub system_requirements: SystemRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_config: Option<ContainerConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfiguration {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub based_on_template: Option<String>,
    pub opensim_ini: OpenSimIniConfig,
    pub region_ini: RegionIniConfig,
    pub ossl_config: OsslConfig,
    #[serde(default)]
    pub config_includes: HashMap<String, String>,
    pub system_requirements: SystemRequirements,
    #[serde(default)]
    pub deployment_type: DeploymentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_config: Option<ContainerConfig>,
    #[serde(default)]
    pub deployment_status: DeploymentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_path: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deployed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub config_id: String,
    pub target_path: String,
    pub deployment_type: DeploymentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_config: Option<ContainerConfig>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub register_with_manager: bool,
    #[serde(default)]
    pub create_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProgress {
    pub config_id: String,
    pub step: String,
    pub progress: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub config_id: String,
    pub instance_id: String,
    pub deployment_type: DeploymentType,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub deployed_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TerrainType {
    Water,
    Land,
    Mixed,
    Void,
}

impl Default for TerrainType {
    fn default() -> Self {
        TerrainType::Land
    }
}

impl TerrainType {
    pub fn default_height(&self) -> i32 {
        match self {
            TerrainType::Water => -30,
            TerrainType::Land => 22,
            TerrainType::Mixed => 0,
            TerrainType::Void => 0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TerrainType::Water => "Underwater terrain for boats and marine activities",
            TerrainType::Land => "Above-ground terrain for buildings and landscaping",
            TerrainType::Mixed => "Variable terrain with both water and land areas",
            TerrainType::Void => "Empty/flat terrain for custom heightmaps",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayout {
    pub grid_width: i32,
    pub grid_height: i32,
    pub base_location_x: i32,
    pub base_location_y: i32,
    pub base_port: u16,
    pub region_size: i32,
    pub naming_pattern: String,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            grid_width: 1,
            grid_height: 1,
            base_location_x: 1000,
            base_location_y: 1000,
            base_port: 9000,
            region_size: 256,
            naming_pattern: "{name}{index:02}".to_string(),
        }
    }
}

impl GridLayout {
    pub fn total_regions(&self) -> i32 {
        self.grid_width * self.grid_height
    }

    pub fn port_range_end(&self) -> u16 {
        self.base_port + (self.total_regions() as u16) - 1
    }

    pub fn world_size_x(&self) -> i32 {
        self.grid_width * self.region_size
    }

    pub fn world_size_y(&self) -> i32 {
        self.grid_height * self.region_size
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortAllocation {
    pub base_port: u16,
    pub allocated_ports: Vec<u16>,
    pub reserved_ranges: Vec<(u16, u16)>,
}

impl Default for PortAllocation {
    fn default() -> Self {
        Self {
            base_port: 9000,
            allocated_ports: Vec::new(),
            reserved_ranges: vec![(9000, 9099), (9100, 9199), (9200, 9299)],
        }
    }
}

impl PortAllocation {
    pub fn is_port_available(&self, port: u16) -> bool {
        !self.allocated_ports.contains(&port)
    }

    pub fn allocate_range(&mut self, count: i32) -> Option<Vec<u16>> {
        let mut ports = Vec::new();
        let mut current_port = self.base_port;

        while ports.len() < count as usize {
            if self.is_port_available(current_port) {
                ports.push(current_port);
                self.allocated_ports.push(current_port);
            }
            current_port += 1;
            if current_port > 65535 {
                return None;
            }
        }

        Some(ports)
    }

    pub fn find_available_range(&self, count: i32, start_from: u16) -> Option<u16> {
        let mut current = start_from;
        let mut consecutive = 0;

        while current <= 65535 - count as u16 {
            if self.is_port_available(current) {
                consecutive += 1;
                if consecutive >= count {
                    return Some(current - count as u16 + 1);
                }
            } else {
                consecutive = 0;
            }
            current += 1;
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRegion {
    pub index: i32,
    pub name: String,
    pub uuid: String,
    pub location_x: i32,
    pub location_y: i32,
    pub internal_port: u16,
    pub size_x: i32,
    pub size_y: i32,
    pub terrain_height: i32,
    pub ini_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionGridConfig {
    pub grid_name: String,
    pub grid_layout: GridLayout,
    pub terrain_type: TerrainType,
    pub terrain_base_height: i32,
    pub max_agents_per_region: i32,
    pub max_prims_per_region: i32,
    pub estate_name: String,
    pub estate_owner: String,
    #[serde(default)]
    pub generated_regions: Vec<GeneratedRegion>,
}

impl Default for RegionGridConfig {
    fn default() -> Self {
        Self {
            grid_name: "New Grid".to_string(),
            grid_layout: GridLayout::default(),
            terrain_type: TerrainType::Land,
            terrain_base_height: 22,
            max_agents_per_region: 40,
            max_prims_per_region: 45000,
            estate_name: "My Estate".to_string(),
            estate_owner: "Admin".to_string(),
            generated_regions: Vec::new(),
        }
    }
}

impl RegionGridConfig {
    pub fn generate_regions(&mut self) {
        self.generated_regions.clear();
        let mut index = 1;

        for y in 0..self.grid_layout.grid_height {
            for x in 0..self.grid_layout.grid_width {
                let region_name = self.format_region_name(index);
                let ini_filename = format!("{}.ini", region_name.to_lowercase().replace(' ', "_"));

                let region = GeneratedRegion {
                    index,
                    name: region_name,
                    uuid: uuid::Uuid::new_v4().to_string(),
                    location_x: self.grid_layout.base_location_x + x,
                    location_y: self.grid_layout.base_location_y + y,
                    internal_port: self.grid_layout.base_port + (index as u16) - 1,
                    size_x: self.grid_layout.region_size,
                    size_y: self.grid_layout.region_size,
                    terrain_height: self.terrain_base_height,
                    ini_filename,
                };

                self.generated_regions.push(region);
                index += 1;
            }
        }
    }

    fn format_region_name(&self, index: i32) -> String {
        self.grid_layout
            .naming_pattern
            .replace("{name}", &self.grid_name)
            .replace("{index:02}", &format!("{:02}", index))
            .replace("{index}", &index.to_string())
    }

    pub fn aggregate_requirements(&self, per_region: &SystemRequirements) -> SystemRequirements {
        let count = self.grid_layout.total_regions();
        let scaling_factor = 0.85_f64;

        SystemRequirements {
            min_memory_mb: (per_region.min_memory_mb as f64 * count as f64 * scaling_factor) as i32,
            recommended_memory_mb: (per_region.recommended_memory_mb as f64
                * count as f64
                * scaling_factor) as i32,
            min_cpu_cores: ((per_region.min_cpu_cores as f64 * count as f64)
                .sqrt()
                .ceil() as i32)
                .max(per_region.min_cpu_cores),
            recommended_cpu_cores: ((per_region.recommended_cpu_cores as f64 * count as f64 * 0.7)
                .ceil() as i32)
                .max(per_region.recommended_cpu_cores),
            network_bandwidth_mbps: per_region.network_bandwidth_mbps * count,
            disk_space_gb: per_region.disk_space_gb * count,
            notes: Some(format!(
                "{} regions, {} total agents capacity",
                count,
                self.max_agents_per_region * count
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorTypeDefaults {
    pub typical_min_sims: i32,
    pub typical_max_sims: i32,
    pub recommended_layout: String,
    pub terrain_type: TerrainType,
    pub terrain_height: i32,
    pub suggested_port_range_start: u16,
}

impl SimulatorType {
    pub fn get_defaults(&self) -> SimulatorTypeDefaults {
        match self {
            SimulatorType::Marina => SimulatorTypeDefaults {
                typical_min_sims: 16,
                typical_max_sims: 64,
                recommended_layout: "4x4 to 8x8".to_string(),
                terrain_type: TerrainType::Water,
                terrain_height: -30,
                suggested_port_range_start: 9000,
            },
            SimulatorType::Mainland => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 100,
                recommended_layout: "variable".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9100,
            },
            SimulatorType::Event => SimulatorTypeDefaults {
                typical_min_sims: 4,
                typical_max_sims: 4,
                recommended_layout: "2x2".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9200,
            },
            SimulatorType::Welcome => SimulatorTypeDefaults {
                typical_min_sims: 8,
                typical_max_sims: 8,
                recommended_layout: "2x4".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9210,
            },
            SimulatorType::Sandbox => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 4,
                recommended_layout: "1x1 to 2x2".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9220,
            },
            SimulatorType::Shopping => SimulatorTypeDefaults {
                typical_min_sims: 4,
                typical_max_sims: 16,
                recommended_layout: "2x2 to 4x4".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9230,
            },
            SimulatorType::Island => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 1,
                recommended_layout: "1x1".to_string(),
                terrain_type: TerrainType::Mixed,
                terrain_height: 0,
                suggested_port_range_start: 9250,
            },
            SimulatorType::Roleplay => SimulatorTypeDefaults {
                typical_min_sims: 4,
                typical_max_sims: 16,
                recommended_layout: "2x2 to 4x4".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9260,
            },
            SimulatorType::Residential => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 4,
                recommended_layout: "1x1 to 2x2".to_string(),
                terrain_type: TerrainType::Land,
                terrain_height: 22,
                suggested_port_range_start: 9280,
            },
            SimulatorType::Void => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 1,
                recommended_layout: "1x1".to_string(),
                terrain_type: TerrainType::Void,
                terrain_height: 0,
                suggested_port_range_start: 9290,
            },
            SimulatorType::CustomTerrain => SimulatorTypeDefaults {
                typical_min_sims: 1,
                typical_max_sims: 16,
                recommended_layout: "variable".to_string(),
                terrain_type: TerrainType::Mixed,
                terrain_height: 0,
                suggested_port_range_start: 9300,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub id: String,
    pub name: String,
    pub world_name: String,
    pub simulator_type: SimulatorType,
    pub grid_config: RegionGridConfig,
    pub base_requirements: SystemRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub custom_region_names: Vec<String>,
}

impl CartItem {
    pub fn new(
        name: String,
        world_name: String,
        simulator_type: SimulatorType,
        grid_config: RegionGridConfig,
        base_requirements: SystemRequirements,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            world_name,
            simulator_type,
            grid_config,
            base_requirements,
            description: None,
            custom_region_names: Vec::new(),
        }
    }

    pub fn with_custom_region_names(mut self, names: Vec<String>) -> Self {
        self.custom_region_names = names;
        self
    }

    pub fn region_count(&self) -> i32 {
        self.grid_config.grid_layout.total_regions()
    }

    pub fn port_range(&self) -> (u16, u16) {
        let layout = &self.grid_config.grid_layout;
        (layout.base_port, layout.port_range_end())
    }

    pub fn max_agent_capacity(&self) -> i32 {
        self.grid_config.max_agents_per_region * self.region_count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealisticUsageProfile {
    pub average_avatars_per_region: f64,
    pub peak_usage_multiplier: f64,
    pub empty_region_percentage: f64,
    pub active_script_percentage: f64,
    pub physics_load_factor: f64,
}

impl Default for RealisticUsageProfile {
    fn default() -> Self {
        Self {
            average_avatars_per_region: 2.0,
            peak_usage_multiplier: 3.0,
            empty_region_percentage: 0.6,
            active_script_percentage: 0.3,
            physics_load_factor: 0.4,
        }
    }
}

impl RealisticUsageProfile {
    pub fn light_usage() -> Self {
        Self {
            average_avatars_per_region: 1.0,
            peak_usage_multiplier: 2.0,
            empty_region_percentage: 0.8,
            active_script_percentage: 0.2,
            physics_load_factor: 0.2,
        }
    }

    pub fn moderate_usage() -> Self {
        Self::default()
    }

    pub fn heavy_usage() -> Self {
        Self {
            average_avatars_per_region: 5.0,
            peak_usage_multiplier: 4.0,
            empty_region_percentage: 0.3,
            active_script_percentage: 0.5,
            physics_load_factor: 0.6,
        }
    }

    pub fn event_usage() -> Self {
        Self {
            average_avatars_per_region: 20.0,
            peak_usage_multiplier: 5.0,
            empty_region_percentage: 0.1,
            active_script_percentage: 0.4,
            physics_load_factor: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageProjection {
    pub max_memory_mb: i32,
    pub realistic_memory_mb: i32,
    pub max_cpu_cores: i32,
    pub realistic_cpu_cores: i32,
    pub max_bandwidth_mbps: i32,
    pub realistic_bandwidth_mbps: i32,
    pub max_avatars: i32,
    pub realistic_avatars: i32,
    pub peak_avatars: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSummary {
    pub world_name: String,
    pub items: Vec<String>,
    pub total_regions: i32,
    pub port_range: (u16, u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartAggregation {
    pub total_regions: i32,
    pub total_items: i32,
    pub port_ranges: Vec<(String, u16, u16)>,
    pub port_conflicts: Vec<(String, String, u16)>,
    pub max_requirements: SystemRequirements,
    pub realistic_projection: UsageProjection,
    pub regions_by_type: HashMap<String, i32>,
    pub terrain_summary: HashMap<String, i32>,
    pub worlds: Vec<WorldSummary>,
    pub total_max_agents: i32,
    pub total_max_prims: i32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingRegion {
    pub name: String,
    pub uuid: String,
    pub location_x: i32,
    pub location_y: i32,
    pub port: u16,
    pub size_x: i32,
    pub size_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingWorld {
    pub world_name: String,
    pub grid_uri: Option<String>,
    pub regions: Vec<ExistingRegion>,
    pub allocated_ports: Vec<u16>,
    pub occupied_locations: Vec<(i32, i32)>,
}

impl ExistingWorld {
    pub fn new(world_name: String) -> Self {
        Self {
            world_name,
            grid_uri: None,
            regions: Vec::new(),
            allocated_ports: Vec::new(),
            occupied_locations: Vec::new(),
        }
    }

    pub fn add_region(&mut self, region: ExistingRegion) {
        self.allocated_ports.push(region.port);
        let size_units_x = region.size_x / 256;
        let size_units_y = region.size_y / 256;
        for dx in 0..size_units_x {
            for dy in 0..size_units_y {
                self.occupied_locations
                    .push((region.location_x + dx, region.location_y + dy));
            }
        }
        self.regions.push(region);
    }

    pub fn is_port_available(&self, port: u16) -> bool {
        !self.allocated_ports.contains(&port)
    }

    pub fn is_location_available(&self, x: i32, y: i32, size_x: i32, size_y: i32) -> bool {
        let size_units_x = size_x / 256;
        let size_units_y = size_y / 256;
        for dx in 0..size_units_x {
            for dy in 0..size_units_y {
                if self.occupied_locations.contains(&(x + dx, y + dy)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_next_available_port(&self, start_from: u16) -> u16 {
        let mut port = start_from;
        while !self.is_port_available(port) && port < 65535 {
            port += 1;
        }
        port
    }

    pub fn find_available_location(
        &self,
        near_x: i32,
        near_y: i32,
        size_x: i32,
        size_y: i32,
    ) -> Option<(i32, i32)> {
        for radius in 0i32..100 {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    if dx.abs() == radius || dy.abs() == radius {
                        let x = near_x + dx;
                        let y = near_y + dy;
                        if x >= 0 && y >= 0 && self.is_location_available(x, y, size_x, size_y) {
                            return Some((x, y));
                        }
                    }
                }
            }
        }
        None
    }

    pub fn suggest_safe_grid_layout(
        &self,
        width: i32,
        height: i32,
        region_size: i32,
    ) -> Option<GridLayout> {
        let near_x = self
            .occupied_locations
            .iter()
            .map(|(x, _)| *x)
            .max()
            .unwrap_or(1000);
        let near_y = self
            .occupied_locations
            .iter()
            .map(|(_, y)| *y)
            .max()
            .unwrap_or(1000);

        if let Some((base_x, base_y)) = self.find_available_location(
            near_x + 1,
            near_y + 1,
            width * region_size,
            height * region_size,
        ) {
            let base_port = self.find_next_available_port(9000);
            let ports_needed = width * height;
            let mut all_ports_available = true;
            for i in 0..ports_needed {
                if !self.is_port_available(base_port + i as u16) {
                    all_ports_available = false;
                    break;
                }
            }

            if all_ports_available {
                return Some(GridLayout {
                    grid_width: width,
                    grid_height: height,
                    base_location_x: base_x,
                    base_location_y: base_y,
                    base_port,
                    region_size,
                    naming_pattern: "{name}{index:02}".to_string(),
                });
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GridDeploymentCart {
    pub items: Vec<CartItem>,
    pub usage_profile: RealisticUsageProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_world: Option<ExistingWorld>,
}

impl GridDeploymentCart {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_existing_world(existing: ExistingWorld) -> Self {
        Self {
            existing_world: Some(existing),
            ..Default::default()
        }
    }

    pub fn add_item(&mut self, item: CartItem) {
        self.items.push(item);
    }

    pub fn validate_against_existing(&self, item: &CartItem) -> Vec<String> {
        let mut conflicts = Vec::new();

        if let Some(existing) = &self.existing_world {
            let (port_start, port_end) = item.port_range();
            for port in port_start..=port_end {
                if !existing.is_port_available(port) {
                    conflicts.push(format!("Port {} conflicts with existing region", port));
                }
            }

            for region in &item.grid_config.generated_regions {
                if !existing.is_location_available(
                    region.location_x,
                    region.location_y,
                    region.size_x,
                    region.size_y,
                ) {
                    conflicts.push(format!(
                        "Location ({},{}) conflicts with existing region",
                        region.location_x, region.location_y
                    ));
                }
            }
        }

        conflicts
    }

    pub fn suggest_safe_layout_for_item(
        &self,
        width: i32,
        height: i32,
        region_size: i32,
    ) -> Option<GridLayout> {
        if let Some(existing) = &self.existing_world {
            let mut combined = existing.clone();
            for item in &self.items {
                for region in &item.grid_config.generated_regions {
                    combined.add_region(ExistingRegion {
                        name: region.name.clone(),
                        uuid: region.uuid.clone(),
                        location_x: region.location_x,
                        location_y: region.location_y,
                        port: region.internal_port,
                        size_x: region.size_x,
                        size_y: region.size_y,
                    });
                }
            }
            combined.suggest_safe_grid_layout(width, height, region_size)
        } else {
            let mut pseudo_world = ExistingWorld::new("temp".to_string());
            for item in &self.items {
                for region in &item.grid_config.generated_regions {
                    pseudo_world.add_region(ExistingRegion {
                        name: region.name.clone(),
                        uuid: region.uuid.clone(),
                        location_x: region.location_x,
                        location_y: region.location_y,
                        port: region.internal_port,
                        size_x: region.size_x,
                        size_y: region.size_y,
                    });
                }
            }
            if pseudo_world.regions.is_empty() {
                Some(GridLayout {
                    grid_width: width,
                    grid_height: height,
                    base_location_x: 1000,
                    base_location_y: 1000,
                    base_port: 9000,
                    region_size,
                    naming_pattern: "{name}{index:02}".to_string(),
                })
            } else {
                pseudo_world.suggest_safe_grid_layout(width, height, region_size)
            }
        }
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<CartItem> {
        if let Some(pos) = self.items.iter().position(|i| i.id == item_id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn total_regions(&self) -> i32 {
        self.items.iter().map(|i| i.region_count()).sum()
    }

    pub fn find_port_conflicts(&self) -> Vec<(String, String, u16)> {
        let mut conflicts = Vec::new();
        let mut port_map: HashMap<u16, &str> = HashMap::new();

        for item in &self.items {
            let (start, end) = item.port_range();
            for port in start..=end {
                if let Some(existing) = port_map.get(&port) {
                    conflicts.push((existing.to_string(), item.name.clone(), port));
                } else {
                    port_map.insert(port, &item.name);
                }
            }
        }

        conflicts
    }

    pub fn aggregate(&self) -> CartAggregation {
        let total_regions = self.total_regions();
        let mut max_memory = 0;
        let mut max_cpu = 0;
        let mut max_bandwidth = 0;
        let mut total_max_agents = 0;
        let mut total_max_prims = 0;
        let mut regions_by_type: HashMap<String, i32> = HashMap::new();
        let mut terrain_summary: HashMap<String, i32> = HashMap::new();
        let mut port_ranges = Vec::new();
        let mut warnings = Vec::new();
        let mut worlds_map: HashMap<String, (Vec<String>, i32, u16, u16)> = HashMap::new();

        for item in &self.items {
            let req = item
                .grid_config
                .aggregate_requirements(&item.base_requirements);
            max_memory += req.min_memory_mb;
            max_cpu = max_cpu.max(req.min_cpu_cores);
            max_bandwidth += req.network_bandwidth_mbps;

            let region_count = item.region_count();
            total_max_agents += item.grid_config.max_agents_per_region * region_count;
            total_max_prims += item.grid_config.max_prims_per_region * region_count;

            let type_name = format!("{:?}", item.simulator_type);
            *regions_by_type.entry(type_name).or_insert(0) += region_count;

            let terrain_name = format!("{:?}", item.grid_config.terrain_type);
            *terrain_summary.entry(terrain_name).or_insert(0) += region_count;

            let (start, end) = item.port_range();
            port_ranges.push((item.name.clone(), start, end));

            let world_entry = worlds_map
                .entry(item.world_name.clone())
                .or_insert_with(|| (Vec::new(), 0, u16::MAX, 0));
            world_entry.0.push(item.name.clone());
            world_entry.1 += region_count;
            world_entry.2 = world_entry.2.min(start);
            world_entry.3 = world_entry.3.max(end);
        }

        let worlds: Vec<WorldSummary> = worlds_map
            .into_iter()
            .map(
                |(world_name, (items, total_regions, port_start, port_end))| WorldSummary {
                    world_name,
                    items,
                    total_regions,
                    port_range: (port_start, port_end),
                },
            )
            .collect();

        let port_conflicts = self.find_port_conflicts();
        if !port_conflicts.is_empty() {
            warnings.push(format!("{} port conflicts detected", port_conflicts.len()));
        }

        if max_memory > 65536 {
            warnings.push(format!(
                "Max memory ({} GB) exceeds typical server capacity",
                max_memory / 1024
            ));
        }

        if total_regions > 256 {
            warnings.push(format!(
                "{} regions is very large - consider multi-server deployment",
                total_regions
            ));
        }

        let profile = &self.usage_profile;
        let active_regions =
            (total_regions as f64 * (1.0 - profile.empty_region_percentage)) as i32;

        let realistic_avatars = (active_regions as f64 * profile.average_avatars_per_region) as i32;
        let peak_avatars = (realistic_avatars as f64 * profile.peak_usage_multiplier) as i32;

        let memory_per_avatar_mb = 50;
        let base_region_memory_mb = 256;
        let realistic_memory = (total_regions * base_region_memory_mb)
            + (realistic_avatars * memory_per_avatar_mb)
            + (max_memory as f64 * profile.active_script_percentage * 0.3) as i32;

        let realistic_cpu = ((max_cpu as f64)
            * (0.3
                + profile.physics_load_factor * 0.4
                + (realistic_avatars as f64 / total_max_agents.max(1) as f64) * 0.3))
            .ceil() as i32;

        let realistic_bandwidth = ((max_bandwidth as f64)
            * (realistic_avatars as f64 / total_max_agents.max(1) as f64).max(0.1))
        .ceil() as i32;

        CartAggregation {
            total_regions,
            total_items: self.items.len() as i32,
            port_ranges,
            port_conflicts,
            max_requirements: SystemRequirements {
                min_memory_mb: max_memory,
                recommended_memory_mb: (max_memory as f64 * 1.5) as i32,
                min_cpu_cores: max_cpu,
                recommended_cpu_cores: (max_cpu as f64 * 1.5).ceil() as i32,
                network_bandwidth_mbps: max_bandwidth,
                disk_space_gb: total_regions * 2,
                notes: Some(format!(
                    "Maximum capacity: {} avatars, {} prims",
                    total_max_agents, total_max_prims
                )),
            },
            realistic_projection: UsageProjection {
                max_memory_mb: max_memory,
                realistic_memory_mb: realistic_memory.max(1024),
                max_cpu_cores: max_cpu,
                realistic_cpu_cores: realistic_cpu.max(2),
                max_bandwidth_mbps: max_bandwidth,
                realistic_bandwidth_mbps: realistic_bandwidth.max(10),
                max_avatars: total_max_agents,
                realistic_avatars,
                peak_avatars: peak_avatars.min(total_max_agents),
            },
            regions_by_type,
            terrain_summary,
            worlds,
            total_max_agents,
            total_max_prims,
            warnings,
        }
    }

    pub fn suggested_next_port(&self) -> u16 {
        let mut max_port: u16 = 9000;
        for item in &self.items {
            let (_, end) = item.port_range();
            max_port = max_port.max(end + 1);
        }
        ((max_port + 99) / 100) * 100
    }
}
