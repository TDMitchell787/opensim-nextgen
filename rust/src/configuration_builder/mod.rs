pub mod models;
pub mod templates;
pub mod ini_generator;
pub mod grid_generator;
pub mod deployment;
pub mod validator;

pub use models::*;
pub use templates::BuiltInTemplates;
pub use ini_generator::IniGenerator;
pub use grid_generator::GridGenerator;
pub use deployment::DeploymentEngine;
pub use validator::ConfigurationValidator;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ConfigBuilderMessage {
    GetTemplates,
    GetTemplate { id: String },
    GetSimulatorTypeDefaults { simulator_type: SimulatorType },
    SaveConfiguration(SavedConfiguration),
    GetConfigurations,
    GetConfiguration { id: String },
    DeleteConfiguration { id: String },
    ValidateConfiguration(SavedConfiguration),
    ValidateGridConfig(RegionGridConfig),
    GenerateGridRegions(RegionGridConfig),
    SuggestGridLayout { region_count: i32 },
    DeployConfiguration(DeploymentRequest),
    DeployGridConfiguration { config: SavedConfiguration, grid_config: RegionGridConfig, request: DeploymentRequest },
    ExportConfiguration { id: String, format: ExportFormat },
    AddToCart { item: CartItem },
    RemoveFromCart { item_id: String },
    ClearCart,
    GetCart,
    UpdateCartUsageProfile { profile: RealisticUsageProfile },
    AggregateCart,
    DeployCart { request: DeploymentRequest },
    LoadExistingWorld { path: String },
    ScanForExistingWorlds { search_path: String },
    SuggestSafeLayout { width: i32, height: i32, region_size: i32 },
    ValidateCartItem { item: CartItem },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Zip,
    Tar,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ConfigBuilderResponse {
    Templates(Vec<SimulatorTemplate>),
    Template(Option<SimulatorTemplate>),
    SimulatorTypeDefaults(SimulatorTypeDefaults),
    Configurations(Vec<SavedConfiguration>),
    Configuration(Option<SavedConfiguration>),
    ConfigurationSaved { id: String },
    ConfigurationDeleted { id: String },
    ValidationResult(ValidationResult),
    GridValidationResult { issues: Vec<grid_generator::GridValidationIssue> },
    GridRegionsGenerated { grid_config: RegionGridConfig, summary: grid_generator::GridSummary },
    GridLayoutSuggestions(Vec<grid_generator::GridLayoutSuggestion>),
    DeploymentProgress(DeploymentProgress),
    DeploymentResult(DeploymentResult),
    ExportReady { url: String },
    Cart(GridDeploymentCart),
    CartItemAdded { item_id: String, suggested_next_port: u16 },
    CartItemRemoved { item_id: String },
    CartCleared,
    CartAggregation(CartAggregation),
    CartDeploymentResult { results: Vec<DeploymentResult>, warnings: Vec<String> },
    ExistingWorldLoaded(ExistingWorld),
    ExistingWorldsFound(Vec<ExistingWorld>),
    SafeLayoutSuggestion(Option<GridLayout>),
    CartItemValidation { conflicts: Vec<String>, safe: bool },
    Error { message: String },
}

pub struct ConfigurationBuilderService {
    deployment_engine: DeploymentEngine,
}

impl ConfigurationBuilderService {
    pub fn new() -> Self {
        Self {
            deployment_engine: DeploymentEngine::new(),
        }
    }

    pub fn with_progress_channel(sender: mpsc::Sender<DeploymentProgress>) -> Self {
        Self {
            deployment_engine: DeploymentEngine::with_progress_channel(sender),
        }
    }

    pub fn get_templates(&self) -> Vec<SimulatorTemplate> {
        BuiltInTemplates::get_all()
    }

    pub fn get_template(&self, id: &str) -> Option<SimulatorTemplate> {
        BuiltInTemplates::get_by_id(id)
    }

    pub fn validate_configuration(&self, config: &SavedConfiguration) -> ValidationResult {
        let validator = ConfigurationValidator::new();
        validator.validate(config)
    }

    pub async fn deploy_configuration(
        &self,
        config: &SavedConfiguration,
        request: &DeploymentRequest,
    ) -> Result<DeploymentResult, String> {
        self.deployment_engine
            .deploy(config, request)
            .await
            .map_err(|e| e.to_string())
    }

    pub fn generate_opensim_ini(&self, config: &OpenSimIniConfig) -> String {
        IniGenerator::generate_opensim_ini(config)
    }

    pub fn generate_region_ini(&self, config: &RegionIniConfig) -> String {
        IniGenerator::generate_region_ini(config)
    }

    pub fn generate_ossl_ini(&self, config: &OsslConfig) -> String {
        IniGenerator::generate_ossl_enable(config)
    }

    pub fn generate_docker_compose(
        &self,
        config: &SavedConfiguration,
        instance_name: &str,
    ) -> String {
        IniGenerator::generate_docker_compose_override(config, instance_name)
    }

    pub fn generate_kubernetes_configmap(
        &self,
        config: &SavedConfiguration,
        instance_name: &str,
    ) -> String {
        IniGenerator::generate_kubernetes_configmap(config, instance_name)
    }

    pub fn generate_helm_values(
        &self,
        config: &SavedConfiguration,
        instance_name: &str,
    ) -> String {
        IniGenerator::generate_helm_values(config, instance_name)
    }

    pub fn get_simulator_type_defaults(&self, simulator_type: &SimulatorType) -> SimulatorTypeDefaults {
        simulator_type.get_defaults()
    }

    pub fn generate_grid_regions(&self, config: &mut RegionGridConfig) -> grid_generator::GridSummary {
        config.generate_regions();
        GridGenerator::generate_grid_summary(config)
    }

    pub fn validate_grid_config(&self, config: &RegionGridConfig) -> Vec<grid_generator::GridValidationIssue> {
        GridGenerator::validate_grid_config(config)
    }

    pub fn suggest_grid_layout(&self, region_count: i32) -> Vec<grid_generator::GridLayoutSuggestion> {
        GridGenerator::suggest_layout_for_count(region_count)
    }

    pub fn generate_grid_ini_files(&self, config: &RegionGridConfig) -> std::collections::HashMap<String, String> {
        GridGenerator::generate_all_region_inis(config)
    }

    pub fn generate_combined_regions_ini(&self, config: &RegionGridConfig) -> String {
        GridGenerator::generate_combined_regions_ini(config)
    }

    pub fn calculate_server_capacity(&self, requirements: &SystemRequirements) -> grid_generator::ServerCapacityEstimate {
        GridGenerator::calculate_server_capacity(requirements)
    }

    pub fn calculate_aggregate_requirements(&self, grid_config: &RegionGridConfig, per_region: &SystemRequirements) -> SystemRequirements {
        grid_config.aggregate_requirements(per_region)
    }

    pub fn create_cart_item(
        &self,
        name: String,
        world_name: String,
        simulator_type: SimulatorType,
        mut grid_config: RegionGridConfig,
    ) -> CartItem {
        let template = self.get_template(&format!("{:?}", simulator_type).to_lowercase());
        let base_requirements = template
            .map(|t| t.system_requirements)
            .unwrap_or_default();

        grid_config.generate_regions();

        CartItem::new(name, world_name, simulator_type, grid_config, base_requirements)
    }

    pub fn aggregate_cart(&self, cart: &GridDeploymentCart) -> CartAggregation {
        cart.aggregate()
    }

    pub fn get_usage_profiles() -> Vec<(&'static str, RealisticUsageProfile)> {
        vec![
            ("Light (mostly empty)", RealisticUsageProfile::light_usage()),
            ("Moderate (2 avg/region)", RealisticUsageProfile::moderate_usage()),
            ("Heavy (5 avg/region)", RealisticUsageProfile::heavy_usage()),
            ("Event (crowded)", RealisticUsageProfile::event_usage()),
        ]
    }

    pub fn load_existing_world_from_path(&self, path: &str) -> Result<ExistingWorld, String> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        let world_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown World")
            .to_string();

        let mut existing = ExistingWorld::new(world_name);

        let regions_dir = if path.is_dir() {
            if path.join("Regions").exists() {
                path.join("Regions")
            } else {
                path.to_path_buf()
            }
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        if let Ok(entries) = fs::read_dir(&regions_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "ini").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        self.parse_region_ini(&content, &mut existing);
                    }
                }
            }
        }

        Ok(existing)
    }

    fn parse_region_ini(&self, content: &str, world: &mut ExistingWorld) {
        let mut current_section: Option<String> = None;
        let mut region_data: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with('[') && line.ends_with(']') {
                if let Some(name) = current_section.take() {
                    self.add_region_from_data(&name, &region_data, world);
                }
                current_section = Some(line[1..line.len()-1].to_string());
                region_data.clear();
            } else if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_lowercase();
                let value = line[pos+1..].trim().to_string();
                region_data.insert(key, value);
            }
        }

        if let Some(name) = current_section {
            self.add_region_from_data(&name, &region_data, world);
        }
    }

    fn add_region_from_data(&self, name: &str, data: &std::collections::HashMap<String, String>, world: &mut ExistingWorld) {
        let uuid = data.get("regionuuid").cloned().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let (location_x, location_y) = if let Some(loc) = data.get("location") {
            let parts: Vec<&str> = loc.split(',').collect();
            (
                parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(1000),
                parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1000),
            )
        } else {
            (1000, 1000)
        };

        let port = data.get("internalport")
            .and_then(|s| s.parse().ok())
            .unwrap_or(9000);

        let size_x = data.get("sizex")
            .and_then(|s| s.parse().ok())
            .unwrap_or(256);

        let size_y = data.get("sizey")
            .and_then(|s| s.parse().ok())
            .unwrap_or(256);

        world.add_region(ExistingRegion {
            name: name.to_string(),
            uuid,
            location_x,
            location_y,
            port,
            size_x,
            size_y,
        });
    }

    pub fn scan_for_existing_worlds(&self, search_path: &str) -> Vec<ExistingWorld> {
        use std::fs;
        use std::path::Path;

        let mut worlds = Vec::new();
        let path = Path::new(search_path);

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if entry_path.join("Regions").exists() || entry_path.join("bin").join("Regions").exists() {
                        if let Ok(world) = self.load_existing_world_from_path(entry_path.to_str().unwrap_or("")) {
                            if !world.regions.is_empty() {
                                worlds.push(world);
                            }
                        }
                    }
                }
            }
        }

        worlds
    }
}

impl Default for ConfigurationBuilderService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = ConfigurationBuilderService::new();
        let templates = service.get_templates();
        assert_eq!(templates.len(), 11);
    }

    #[test]
    fn test_get_template_by_id() {
        let service = ConfigurationBuilderService::new();
        let template = service.get_template("event");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Event Venue");
    }

    #[test]
    fn test_ini_generation() {
        let service = ConfigurationBuilderService::new();
        let template = service.get_template("mainland").unwrap();

        let opensim_ini = service.generate_opensim_ini(&template.opensim_ini);
        assert!(opensim_ini.contains("[Startup]"));
        assert!(opensim_ini.contains("[Network]"));

        let region_ini = service.generate_region_ini(&template.region_ini);
        assert!(region_ini.contains("[Mainland Region]"));

        let ossl_ini = service.generate_ossl_ini(&template.ossl_config);
        assert!(ossl_ini.contains("[OSSL]"));
    }

    #[test]
    fn test_deployment_cart() {
        let service = ConfigurationBuilderService::new();
        let mut cart = GridDeploymentCart::new();

        let marina_config = RegionGridConfig {
            grid_name: "Marina West".to_string(),
            grid_layout: GridLayout {
                grid_width: 8,
                grid_height: 8,
                base_port: 9000,
                ..Default::default()
            },
            terrain_type: TerrainType::Water,
            terrain_base_height: -30,
            ..Default::default()
        };

        let marina_item = service.create_cart_item(
            "Marina West".to_string(),
            "My World".to_string(),
            SimulatorType::Marina,
            marina_config,
        );
        cart.add_item(marina_item);

        let island_config = RegionGridConfig {
            grid_name: "Paradise Island".to_string(),
            grid_layout: GridLayout {
                grid_width: 2,
                grid_height: 2,
                base_port: 9100,
                ..Default::default()
            },
            terrain_type: TerrainType::Mixed,
            terrain_base_height: 0,
            ..Default::default()
        };

        let island_item = service.create_cart_item(
            "Paradise Island".to_string(),
            "My World".to_string(),
            SimulatorType::Island,
            island_config,
        );
        cart.add_item(island_item);

        let shopping_config = RegionGridConfig {
            grid_name: "Shopping District".to_string(),
            grid_layout: GridLayout {
                grid_width: 4,
                grid_height: 4,
                base_port: 9200,
                ..Default::default()
            },
            terrain_type: TerrainType::Land,
            terrain_base_height: 22,
            ..Default::default()
        };

        let shopping_item = service.create_cart_item(
            "Shopping District".to_string(),
            "My World".to_string(),
            SimulatorType::Shopping,
            shopping_config,
        );
        cart.add_item(shopping_item);

        let residential_config = RegionGridConfig {
            grid_name: "Residential Area".to_string(),
            grid_layout: GridLayout {
                grid_width: 5,
                grid_height: 5,
                base_port: 9300,
                ..Default::default()
            },
            terrain_type: TerrainType::Land,
            terrain_base_height: 22,
            ..Default::default()
        };

        let residential_item = service.create_cart_item(
            "Residential Area".to_string(),
            "My World".to_string(),
            SimulatorType::Residential,
            residential_config,
        );
        cart.add_item(residential_item);

        assert_eq!(cart.items.len(), 4);
        assert_eq!(cart.total_regions(), 64 + 4 + 16 + 25);

        let aggregation = service.aggregate_cart(&cart);

        assert_eq!(aggregation.total_regions, 109);
        assert_eq!(aggregation.total_items, 4);
        assert_eq!(aggregation.worlds.len(), 1);
        assert!(aggregation.port_conflicts.is_empty());

        assert!(aggregation.realistic_projection.realistic_avatars < aggregation.realistic_projection.max_avatars);
        assert!(aggregation.realistic_projection.realistic_memory_mb < aggregation.max_requirements.min_memory_mb);

        println!("Cart aggregation for {} regions:", aggregation.total_regions);
        println!("  Max memory: {} MB", aggregation.max_requirements.min_memory_mb);
        println!("  Realistic memory: {} MB", aggregation.realistic_projection.realistic_memory_mb);
        println!("  Max avatars: {}", aggregation.realistic_projection.max_avatars);
        println!("  Realistic avatars: {}", aggregation.realistic_projection.realistic_avatars);
        println!("  Peak avatars: {}", aggregation.realistic_projection.peak_avatars);
    }

    #[test]
    fn test_cart_port_conflicts() {
        let service = ConfigurationBuilderService::new();
        let mut cart = GridDeploymentCart::new();

        let config1 = RegionGridConfig {
            grid_name: "Area 1".to_string(),
            grid_layout: GridLayout {
                grid_width: 2,
                grid_height: 2,
                base_port: 9000,
                ..Default::default()
            },
            ..Default::default()
        };

        let config2 = RegionGridConfig {
            grid_name: "Area 2".to_string(),
            grid_layout: GridLayout {
                grid_width: 2,
                grid_height: 2,
                base_port: 9002,
                ..Default::default()
            },
            ..Default::default()
        };

        cart.add_item(service.create_cart_item("Area 1".to_string(), "World".to_string(), SimulatorType::Mainland, config1));
        cart.add_item(service.create_cart_item("Area 2".to_string(), "World".to_string(), SimulatorType::Mainland, config2));

        let aggregation = cart.aggregate();
        assert!(!aggregation.port_conflicts.is_empty());
        assert!(!aggregation.warnings.is_empty());
    }
}
