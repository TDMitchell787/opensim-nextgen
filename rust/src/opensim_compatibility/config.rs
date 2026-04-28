//! OpenSim INI configuration parser
//!
//! Provides compatibility with OpenSimulator's cascading INI configuration system.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// OpenSim INI configuration parser
pub struct IniConfigParser {
    sections: HashMap<String, HashMap<String, String>>,
    include_stack: Vec<String>,
}

/// Region configuration parsed from INI files
#[derive(Debug, Clone)]
pub struct RegionConfig {
    pub region_uuid: String,
    pub region_name: String,
    pub location_x: u32,
    pub location_y: u32,
    pub internal_address: String,
    pub internal_port: u16,
    pub external_host_name: String,
    pub physics_engine: String,
    pub max_prims: u32,
    pub max_agents: u32,
}

impl IniConfigParser {
    /// Create a new INI configuration parser
    pub fn new() -> Result<Self> {
        Ok(Self {
            sections: HashMap::new(),
            include_stack: Vec::new(),
        })
    }

    /// Load main OpenSim.ini configuration with includes
    pub fn load_main_config(&mut self, config_path: &Path) -> Result<()> {
        self.include_stack.clear();
        self.load_config_file(config_path)?;
        Ok(())
    }

    /// Load a configuration file and process includes
    fn load_config_file(&mut self, config_path: &Path) -> Result<()> {
        let config_path_str = config_path.to_string_lossy().to_string();

        // Prevent circular includes
        if self.include_stack.contains(&config_path_str) {
            return Err(anyhow!(format!(
                "Circular include detected: {}",
                config_path_str
            )));
        }

        self.include_stack.push(config_path_str);

        let content = fs::read_to_string(config_path).map_err(|e| {
            anyhow!(format!(
                "Failed to read config file {}: {}",
                config_path.display(),
                e
            ))
        })?;

        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Handle section headers
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                self.sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new);
                continue;
            }

            // Handle includes
            if line.starts_with("Include-") {
                if let Some(eq_pos) = line.find('=') {
                    let include_path = line[eq_pos + 1..].trim().trim_matches('"');
                    let full_include_path = if include_path.starts_with('/') {
                        Path::new(include_path).to_path_buf()
                    } else {
                        config_path
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(include_path)
                    };

                    if full_include_path.exists() {
                        self.load_config_file(&full_include_path)?;
                    } else {
                        tracing::warn!("Include file not found: {}", full_include_path.display());
                    }
                }
                continue;
            }

            // Handle key-value pairs
            if let Some(eq_pos) = line.find('=') {
                if current_section.is_empty() {
                    return Err(anyhow!(format!(
                        "Key-value pair outside section at line {}: {}",
                        line_num + 1,
                        line
                    )));
                }

                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().trim_matches('"').to_string();

                self.sections
                    .get_mut(&current_section)
                    .unwrap()
                    .insert(key, value);
            }
        }

        self.include_stack.pop();
        Ok(())
    }

    /// Get a configuration value
    pub fn get_value(&self, section: &str, key: &str) -> Option<String> {
        self.sections.get(section)?.get(key).cloned()
    }

    /// Get all values in a section
    pub fn get_section(&self, section: &str) -> Option<&HashMap<String, String>> {
        self.sections.get(section)
    }

    /// Load region configurations from Regions.ini
    pub fn load_region_configs(&self, regions_path: &Path) -> Result<Vec<RegionConfig>> {
        let mut region_parser = IniConfigParser::new()?;
        region_parser.load_config_file(regions_path)?;

        let mut regions = Vec::new();

        for (section_name, section_data) in &region_parser.sections {
            // Skip non-region sections
            if section_name == "Regions" || section_name.is_empty() {
                continue;
            }

            let region = RegionConfig {
                region_uuid: section_data
                    .get("RegionUUID")
                    .unwrap_or(&"00000000-0000-0000-0000-000000000000".to_string())
                    .clone(),
                region_name: section_data
                    .get("RegionName")
                    .unwrap_or(section_name)
                    .clone(),
                location_x: section_data
                    .get("Location")
                    .and_then(|loc| loc.split(',').next())
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(1000),
                location_y: section_data
                    .get("Location")
                    .and_then(|loc| loc.split(',').nth(1))
                    .and_then(|y| y.parse().ok())
                    .unwrap_or(1000),
                internal_address: section_data
                    .get("InternalAddress")
                    .unwrap_or(&"0.0.0.0".to_string())
                    .clone(),
                internal_port: section_data
                    .get("InternalPort")
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(9000),
                external_host_name: section_data
                    .get("ExternalHostName")
                    .unwrap_or(&"127.0.0.1".to_string())
                    .clone(),
                physics_engine: section_data
                    .get("PhysicsEngine")
                    .unwrap_or(&"POS".to_string())
                    .clone(),
                max_prims: section_data
                    .get("MaxPrims")
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(100000),
                max_agents: section_data
                    .get("MaxAgents")
                    .and_then(|a| a.parse().ok())
                    .unwrap_or(100),
            };

            regions.push(region);
        }

        tracing::info!("Loaded {} region configurations", regions.len());
        Ok(regions)
    }

    /// Get physics engine configuration
    pub fn get_physics_config(&self) -> String {
        self.get_value("Startup", "physics")
            .unwrap_or_else(|| "POS".to_string())
    }

    /// Get network configuration  
    pub fn get_network_config(&self) -> (String, u16) {
        let address = self
            .get_value("Network", "ExternalHostNameForLSL")
            .or_else(|| self.get_value("Startup", "ExternalHostNameForLSL"))
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = self
            .get_value("Network", "http_listener_port")
            .or_else(|| self.get_value("Startup", "http_listener_port"))
            .and_then(|p| p.parse().ok())
            .unwrap_or(9000);

        (address, port)
    }

    /// Check if hypergrid is enabled
    pub fn is_hypergrid_enabled(&self) -> bool {
        self.get_value("Hypergrid", "Enabled")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    }
}
