use super::models::*;
use std::collections::HashMap;

pub struct GridGenerator;

impl GridGenerator {
    pub fn generate_region_ini(region: &GeneratedRegion, config: &RegionGridConfig) -> String {
        let mut ini = String::new();

        ini.push_str(&format!("[{}]\n", region.name));
        ini.push_str(&format!("RegionUUID = {}\n", region.uuid));
        ini.push_str(&format!(
            "Location = {},{}\n",
            region.location_x, region.location_y
        ));
        ini.push_str("InternalAddress = 0.0.0.0\n");
        ini.push_str(&format!("InternalPort = {}\n", region.internal_port));
        ini.push_str("AllowAlternatePorts = False\n");
        ini.push_str("ExternalHostName = SYSTEMIP\n");
        ini.push_str(&format!("SizeX = {}\n", region.size_x));
        ini.push_str(&format!("SizeY = {}\n", region.size_y));
        ini.push_str(&format!("MaxPrims = {}\n", config.max_prims_per_region));
        ini.push_str(&format!("MaxAgents = {}\n", config.max_agents_per_region));

        ini
    }

    pub fn generate_all_region_inis(config: &RegionGridConfig) -> HashMap<String, String> {
        let mut files = HashMap::new();

        for region in &config.generated_regions {
            let content = Self::generate_region_ini(region, config);
            files.insert(region.ini_filename.clone(), content);
        }

        files
    }

    pub fn generate_combined_regions_ini(config: &RegionGridConfig) -> String {
        let mut ini = String::new();

        ini.push_str("; OpenSim Multi-Region Configuration\n");
        ini.push_str(&format!(
            "; Grid: {} ({} regions)\n",
            config.grid_name,
            config.generated_regions.len()
        ));
        ini.push_str(&format!(
            "; Layout: {}x{}\n",
            config.grid_layout.grid_width, config.grid_layout.grid_height
        ));
        ini.push_str(&format!(
            "; Base Location: {},{}\n",
            config.grid_layout.base_location_x, config.grid_layout.base_location_y
        ));
        ini.push_str(&format!(
            "; Port Range: {}-{}\n",
            config.grid_layout.base_port,
            config.grid_layout.port_range_end()
        ));
        ini.push_str(&format!(
            "; Terrain Type: {:?} (height: {})\n",
            config.terrain_type, config.terrain_base_height
        ));
        ini.push_str("\n");

        for region in &config.generated_regions {
            ini.push_str(&Self::generate_region_ini(region, config));
            ini.push_str("\n");
        }

        ini
    }

    pub fn generate_grid_summary(config: &RegionGridConfig) -> GridSummary {
        let total_regions = config.grid_layout.total_regions();

        GridSummary {
            grid_name: config.grid_name.clone(),
            total_regions,
            grid_width: config.grid_layout.grid_width,
            grid_height: config.grid_layout.grid_height,
            world_size_x: config.grid_layout.world_size_x(),
            world_size_y: config.grid_layout.world_size_y(),
            base_location: (
                config.grid_layout.base_location_x,
                config.grid_layout.base_location_y,
            ),
            port_range: (
                config.grid_layout.base_port,
                config.grid_layout.port_range_end(),
            ),
            terrain_type: config.terrain_type.clone(),
            terrain_height: config.terrain_base_height,
            total_agent_capacity: config.max_agents_per_region * total_regions,
            total_prim_capacity: config.max_prims_per_region * total_regions,
            regions: config
                .generated_regions
                .iter()
                .map(|r| RegionSummary {
                    index: r.index,
                    name: r.name.clone(),
                    location: (r.location_x, r.location_y),
                    port: r.internal_port,
                    ini_file: r.ini_filename.clone(),
                })
                .collect(),
        }
    }

    pub fn validate_grid_config(config: &RegionGridConfig) -> Vec<GridValidationIssue> {
        let mut issues = Vec::new();

        if config.grid_layout.grid_width < 1 || config.grid_layout.grid_height < 1 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Error,
                field: "grid_layout".to_string(),
                message: "Grid dimensions must be at least 1x1".to_string(),
            });
        }

        if config.grid_layout.grid_width > 16 || config.grid_layout.grid_height > 16 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Warning,
                field: "grid_layout".to_string(),
                message: format!(
                    "Large grid ({}x{}) may require significant server resources",
                    config.grid_layout.grid_width, config.grid_layout.grid_height
                ),
            });
        }

        let total_regions = config.grid_layout.total_regions();
        if total_regions > 100 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Warning,
                field: "total_regions".to_string(),
                message: format!(
                    "{} regions is very large - ensure adequate server capacity",
                    total_regions
                ),
            });
        }

        let port_end = config.grid_layout.port_range_end();
        if port_end > 65535 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Error,
                field: "port_range".to_string(),
                message: format!("Port range exceeds maximum (would need port {})", port_end),
            });
        }

        if config.grid_layout.base_port < 1024 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Warning,
                field: "base_port".to_string(),
                message: "Ports below 1024 require elevated privileges".to_string(),
            });
        }

        let valid_sizes = [256, 512, 768, 1024];
        if !valid_sizes.contains(&config.grid_layout.region_size) {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Warning,
                field: "region_size".to_string(),
                message: format!(
                    "Non-standard region size {}. Standard sizes: 256, 512, 768, 1024",
                    config.grid_layout.region_size
                ),
            });
        }

        if config.grid_layout.region_size % 256 != 0 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Error,
                field: "region_size".to_string(),
                message: "Region size must be a multiple of 256".to_string(),
            });
        }

        if config.grid_layout.base_location_x < 0 || config.grid_layout.base_location_y < 0 {
            issues.push(GridValidationIssue {
                severity: IssueSeverity::Error,
                field: "base_location".to_string(),
                message: "Grid coordinates must be non-negative".to_string(),
            });
        }

        match config.terrain_type {
            TerrainType::Water => {
                if config.terrain_base_height > 0 {
                    issues.push(GridValidationIssue {
                        severity: IssueSeverity::Warning,
                        field: "terrain_height".to_string(),
                        message: format!("Water terrain typically uses negative height (recommended: -30, current: {})",
                            config.terrain_base_height),
                    });
                }
            }
            TerrainType::Land => {
                if config.terrain_base_height < 20 {
                    issues.push(GridValidationIssue {
                        severity: IssueSeverity::Warning,
                        field: "terrain_height".to_string(),
                        message: format!(
                            "Land terrain typically uses height 22+ (current: {})",
                            config.terrain_base_height
                        ),
                    });
                }
            }
            _ => {}
        }

        issues
    }

    pub fn suggest_layout_for_count(region_count: i32) -> Vec<GridLayoutSuggestion> {
        let mut suggestions = Vec::new();

        let sqrt = (region_count as f64).sqrt();
        let square_side = sqrt.ceil() as i32;
        if square_side * square_side >= region_count {
            suggestions.push(GridLayoutSuggestion {
                width: square_side,
                height: (region_count as f64 / square_side as f64).ceil() as i32,
                description: "Square-ish layout".to_string(),
                efficiency: (region_count as f64 / (square_side * square_side) as f64 * 100.0)
                    as i32,
            });
        }

        for (w, h) in [
            (1, region_count),
            (2, (region_count + 1) / 2),
            (4, (region_count + 3) / 4),
            (8, (region_count + 7) / 8),
        ] {
            if w * h >= region_count && w <= 16 && h <= 16 {
                suggestions.push(GridLayoutSuggestion {
                    width: w,
                    height: h,
                    description: format!("{}x{} strip", w, h),
                    efficiency: (region_count as f64 / (w * h) as f64 * 100.0) as i32,
                });
            }
        }

        suggestions.sort_by(|a, b| b.efficiency.cmp(&a.efficiency));
        suggestions.dedup_by(|a, b| a.width == b.width && a.height == b.height);
        suggestions.truncate(5);

        suggestions
    }

    pub fn calculate_server_capacity(requirements: &SystemRequirements) -> ServerCapacityEstimate {
        ServerCapacityEstimate {
            max_regions_8gb: 8192 / requirements.min_memory_mb.max(1),
            max_regions_16gb: 16384 / requirements.min_memory_mb.max(1),
            max_regions_32gb: 32768 / requirements.min_memory_mb.max(1),
            max_regions_64gb: 65536 / requirements.min_memory_mb.max(1),
            memory_per_region_mb: requirements.min_memory_mb,
            recommended_memory_per_region_mb: requirements.recommended_memory_mb,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridSummary {
    pub grid_name: String,
    pub total_regions: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub world_size_x: i32,
    pub world_size_y: i32,
    pub base_location: (i32, i32),
    pub port_range: (u16, u16),
    pub terrain_type: TerrainType,
    pub terrain_height: i32,
    pub total_agent_capacity: i32,
    pub total_prim_capacity: i32,
    pub regions: Vec<RegionSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionSummary {
    pub index: i32,
    pub name: String,
    pub location: (i32, i32),
    pub port: u16,
    pub ini_file: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridValidationIssue {
    pub severity: IssueSeverity,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridLayoutSuggestion {
    pub width: i32,
    pub height: i32,
    pub description: String,
    pub efficiency: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerCapacityEstimate {
    pub max_regions_8gb: i32,
    pub max_regions_16gb: i32,
    pub max_regions_32gb: i32,
    pub max_regions_64gb: i32,
    pub memory_per_region_mb: i32,
    pub recommended_memory_per_region_mb: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_marina_grid() {
        let mut config = RegionGridConfig {
            grid_name: "Marina West".to_string(),
            grid_layout: GridLayout {
                grid_width: 4,
                grid_height: 4,
                base_location_x: 1000,
                base_location_y: 1000,
                base_port: 9000,
                region_size: 256,
                naming_pattern: "{name} {index:02}".to_string(),
            },
            terrain_type: TerrainType::Water,
            terrain_base_height: -30,
            max_agents_per_region: 40,
            max_prims_per_region: 30000,
            estate_name: "Marina Estate".to_string(),
            estate_owner: "Admin".to_string(),
            generated_regions: Vec::new(),
        };

        config.generate_regions();

        assert_eq!(config.generated_regions.len(), 16);
        assert_eq!(config.generated_regions[0].name, "Marina West 01");
        assert_eq!(config.generated_regions[0].internal_port, 9000);
        assert_eq!(config.generated_regions[15].name, "Marina West 16");
        assert_eq!(config.generated_regions[15].internal_port, 9015);
    }

    #[test]
    fn test_validate_grid_config() {
        let config = RegionGridConfig {
            grid_name: "Test".to_string(),
            grid_layout: GridLayout {
                grid_width: 20,
                grid_height: 20,
                base_port: 9000,
                ..Default::default()
            },
            ..Default::default()
        };

        let issues = GridGenerator::validate_grid_config(&config);
        assert!(issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Warning && i.field == "grid_layout"));
    }

    #[test]
    fn test_suggest_layout() {
        let suggestions = GridGenerator::suggest_layout_for_count(16);
        assert!(suggestions.iter().any(|s| s.width == 4 && s.height == 4));
    }
}
