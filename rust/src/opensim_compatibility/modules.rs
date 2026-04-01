//! OpenSim addon module loader
//!
//! Provides compatibility with existing OpenSimulator addon modules
//! and third-party plugins.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::Result;

/// Loaded addon module information
#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub module_type: ModuleType,
    pub enabled: bool,
}

/// Types of OpenSim modules
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    RegionModule,
    SharedModule,
    ScriptEngine,
    PhysicsEngine,
    AssetConnector,
    InventoryConnector,
    UserService,
    GridService,
    Unknown,
}

/// Module loader for OpenSim compatibility
pub struct ModuleLoader {
    addon_modules_path: PathBuf,
    loaded_modules: Vec<LoadedModule>,
    module_configs: HashMap<String, ModuleConfig>,
}

/// Module configuration
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    pub enabled: bool,
    pub priority: i32,
    pub config_section: Option<String>,
    pub initialization_params: HashMap<String, String>,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new(addon_modules_path: PathBuf) -> Result<Self> {
        Ok(Self {
            addon_modules_path,
            loaded_modules: Vec::new(),
            module_configs: HashMap::new(),
        })
    }

    /// Scan for available addon modules
    pub async fn scan_modules(&mut self) -> Result<()> {
        if !self.addon_modules_path.exists() {
            fs::create_dir_all(&self.addon_modules_path)?;
            tracing::info!("Created addon-modules directory: {}", self.addon_modules_path.display());
            return Ok(());
        }

        self.loaded_modules.clear();

        // Scan for .dll files (Windows modules)
        self.scan_dll_modules().await?;
        
        // Scan for .so files (Linux modules)
        self.scan_so_modules().await?;
        
        // Scan for .dylib files (macOS modules)
        self.scan_dylib_modules().await?;
        
        // Scan for .jar files (Java modules)
        self.scan_jar_modules().await?;
        
        // Scan for module metadata files
        self.scan_metadata_files().await?;

        tracing::info!("Scanned {} addon modules from {}", 
                      self.loaded_modules.len(), 
                      self.addon_modules_path.display());
        
        Ok(())
    }

    /// Scan for Windows DLL modules
    async fn scan_dll_modules(&mut self) -> Result<()> {
        let dll_pattern = self.addon_modules_path.join("*.dll");
        self.scan_modules_by_pattern(&dll_pattern, "dll").await
    }

    /// Scan for Linux SO modules
    async fn scan_so_modules(&mut self) -> Result<()> {
        let so_pattern = self.addon_modules_path.join("*.so");
        self.scan_modules_by_pattern(&so_pattern, "so").await
    }

    /// Scan for macOS dylib modules
    async fn scan_dylib_modules(&mut self) -> Result<()> {
        let dylib_pattern = self.addon_modules_path.join("*.dylib");
        self.scan_modules_by_pattern(&dylib_pattern, "dylib").await
    }

    /// Scan for Java JAR modules
    async fn scan_jar_modules(&mut self) -> Result<()> {
        let jar_pattern = self.addon_modules_path.join("*.jar");
        self.scan_modules_by_pattern(&jar_pattern, "jar").await
    }

    /// Scan modules by file pattern
    async fn scan_modules_by_pattern(&mut self, pattern: &Path, extension: &str) -> Result<()> {
        let parent_dir = pattern.parent().unwrap_or(Path::new("."));
        
        if !parent_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(parent_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == extension {
                    let module = self.create_module_from_path(&path)?;
                    self.loaded_modules.push(module);
                }
            }
        }
        
        Ok(())
    }

    /// Scan for module metadata files (.xml, .ini, .json)
    async fn scan_metadata_files(&mut self) -> Result<()> {
        let metadata_extensions = &["xml", "ini", "json", "config"];
        
        for extension in metadata_extensions {
            let entries = fs::read_dir(&self.addon_modules_path)?;
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == *extension {
                        self.parse_metadata_file(&path)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Create module info from file path
    fn create_module_from_path(&self, path: &Path) -> Result<LoadedModule> {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let module_type = self.detect_module_type(&name, path);

        Ok(LoadedModule {
            name: name.clone(),
            path: path.to_path_buf(),
            version: None, // Will be populated from metadata if available
            description: None,
            dependencies: Vec::new(),
            module_type,
            enabled: self.is_module_enabled(&name),
        })
    }

    /// Detect module type from name and path
    fn detect_module_type(&self, name: &str, _path: &Path) -> ModuleType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("region") {
            ModuleType::RegionModule
        } else if name_lower.contains("shared") {
            ModuleType::SharedModule
        } else if name_lower.contains("script") || name_lower.contains("lsl") {
            ModuleType::ScriptEngine
        } else if name_lower.contains("physics") || name_lower.contains("ode") || name_lower.contains("bullet") {
            ModuleType::PhysicsEngine
        } else if name_lower.contains("asset") {
            ModuleType::AssetConnector
        } else if name_lower.contains("inventory") {
            ModuleType::InventoryConnector
        } else if name_lower.contains("user") {
            ModuleType::UserService
        } else if name_lower.contains("grid") {
            ModuleType::GridService
        } else {
            ModuleType::Unknown
        }
    }

    /// Parse module metadata file
    fn parse_metadata_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        
        // Try to find corresponding module
        let metadata_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Apply ELEGANT ARCHIVE SOLUTION: Single-pass extraction to avoid borrow conflicts
        let module_index = self.loaded_modules.iter()
            .position(|m| m.name == metadata_name);
            
        if let Some(index) = module_index {
            // Extract metadata parsing action based on file extension
            let parse_action = match path.extension().and_then(|s| s.to_str()) {
                Some("xml") => Some("xml"),
                Some("ini") => Some("ini"), 
                Some("json") => Some("json"),
                _ => {
                    tracing::warn!("Unknown metadata format: {}", path.display());
                    None
                }
            };
            
            // Now safely get mutable reference and apply the action
            if let Some(module) = self.loaded_modules.get_mut(index) {
                if let Some(action) = parse_action {
                    match action {
                        "xml" => Self::parse_xml_metadata_static(module, &content)?,
                        "ini" => Self::parse_ini_metadata_static(module, &content)?,
                        "json" => Self::parse_json_metadata_static(module, &content)?,
                        _ => unreachable!(),
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Parse XML metadata (OpenSim addin format)
    fn parse_xml_metadata(&self, module: &mut LoadedModule, content: &str) -> Result<()> {
        // Basic XML parsing for OpenSim addin manifest
        // In a real implementation, you'd use a proper XML parser
        
        if let Some(start) = content.find("<Description>") {
            if let Some(end) = content[start..].find("</Description>") {
                let desc_start = start + "<Description>".len();
                let desc_end = start + end;
                module.description = Some(content[desc_start..desc_end].trim().to_string());
            }
        }
        
        if let Some(start) = content.find("<Version>") {
            if let Some(end) = content[start..].find("</Version>") {
                let ver_start = start + "<Version>".len();
                let ver_end = start + end;
                module.version = Some(content[ver_start..ver_end].trim().to_string());
            }
        }
        
        Ok(())
    }

    /// Parse INI metadata
    fn parse_ini_metadata(&self, module: &mut LoadedModule, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Description=") {
                module.description = Some(line[12..].trim_matches('"').to_string());
            } else if line.starts_with("Version=") {
                module.version = Some(line[8..].trim_matches('"').to_string());
            }
        }
        Ok(())
    }

    /// Parse JSON metadata
    fn parse_json_metadata(&self, module: &mut LoadedModule, content: &str) -> Result<()> {
        // Basic JSON parsing - in production, use serde_json
        if let Some(start) = content.find("\"description\"") {
            if let Some(colon) = content[start..].find(':') {
                let after_colon = start + colon + 1;
                if let Some(quote_start) = content[after_colon..].find('"') {
                    let desc_start = after_colon + quote_start + 1;
                    if let Some(quote_end) = content[desc_start..].find('"') {
                        let desc_end = desc_start + quote_end;
                        module.description = Some(content[desc_start..desc_end].to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Static version of parse_xml_metadata to avoid borrow conflicts
    fn parse_xml_metadata_static(module: &mut LoadedModule, content: &str) -> Result<()> {
        // Basic XML parsing for OpenSim addin manifest
        // In a real implementation, you'd use a proper XML parser
        
        if let Some(start) = content.find("<Description>") {
            if let Some(end) = content[start..].find("</Description>") {
                let desc_start = start + "<Description>".len();
                let desc_end = start + end;
                module.description = Some(content[desc_start..desc_end].trim().to_string());
            }
        }
        
        if let Some(start) = content.find("<Version>") {
            if let Some(end) = content[start..].find("</Version>") {
                let ver_start = start + "<Version>".len();
                let ver_end = start + end;
                module.version = Some(content[ver_start..ver_end].trim().to_string());
            }
        }
        
        Ok(())
    }

    /// Static version of parse_ini_metadata to avoid borrow conflicts
    fn parse_ini_metadata_static(module: &mut LoadedModule, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Description=") {
                module.description = Some(line[12..].trim_matches('"').to_string());
            } else if line.starts_with("Version=") {
                module.version = Some(line[8..].trim_matches('"').to_string());
            }
        }
        Ok(())
    }

    /// Static version of parse_json_metadata to avoid borrow conflicts
    fn parse_json_metadata_static(module: &mut LoadedModule, content: &str) -> Result<()> {
        // Basic JSON parsing - in production, use serde_json
        if let Some(start) = content.find("\"description\"") {
            if let Some(colon) = content[start..].find(':') {
                let after_colon = start + colon + 1;
                if let Some(quote_start) = content[after_colon..].find('"') {
                    let desc_start = after_colon + quote_start + 1;
                    if let Some(quote_end) = content[desc_start..].find('"') {
                        let desc_end = desc_start + quote_end;
                        module.description = Some(content[desc_start..desc_end].to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if module is enabled in configuration
    fn is_module_enabled(&self, name: &str) -> bool {
        self.module_configs.get(name)
            .map(|config| config.enabled)
            .unwrap_or(true) // Default to enabled
    }

    /// Get loaded modules
    pub fn get_modules(&self) -> &[LoadedModule] {
        &self.loaded_modules
    }

    /// Get modules by type
    pub fn get_modules_by_type(&self, module_type: ModuleType) -> Vec<&LoadedModule> {
        self.loaded_modules.iter()
            .filter(|m| m.module_type == module_type)
            .collect()
    }

    /// Enable/disable a module
    pub fn set_module_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        if let Some(module) = self.loaded_modules.iter_mut().find(|m| m.name == name) {
            module.enabled = enabled;
            
            // Update config
            self.module_configs.entry(name.to_string())
                .or_insert_with(|| ModuleConfig {
                    enabled: true,
                    priority: 0,
                    config_section: None,
                    initialization_params: HashMap::new(),
                })
                .enabled = enabled;
                
            tracing::info!("Module {} {}", name, if enabled { "enabled" } else { "disabled" });
        }
        Ok(())
    }

    /// Get module configuration
    pub fn get_module_config(&self, name: &str) -> Option<&ModuleConfig> {
        self.module_configs.get(name)
    }

    /// Set module configuration
    pub fn set_module_config(&mut self, name: String, config: ModuleConfig) {
        self.module_configs.insert(name, config);
    }

    /// Load module configuration from INI
    pub fn load_module_configs_from_ini(&mut self, ini_content: &str) -> Result<()> {
        let mut current_section = String::new();
        let mut current_config = ModuleConfig {
            enabled: true,
            priority: 0,
            config_section: None,
            initialization_params: HashMap::new(),
        };
        
        for line in ini_content.lines() {
            let line = line.trim();
            
            if line.starts_with('[') && line.ends_with(']') {
                // Save previous config if we have one
                if !current_section.is_empty() {
                    current_config.config_section = Some(current_section.clone());
                    self.module_configs.insert(current_section.clone(), current_config.clone());
                }
                
                // Start new section
                current_section = line[1..line.len()-1].to_string();
                current_config = ModuleConfig {
                    enabled: true,
                    priority: 0,
                    config_section: Some(current_section.clone()),
                    initialization_params: HashMap::new(),
                };
            } else if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim().trim_matches('"');
                
                match key {
                    "Enabled" => current_config.enabled = value.to_lowercase() == "true",
                    "Priority" => current_config.priority = value.parse().unwrap_or(0),
                    _ => {
                        current_config.initialization_params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        // Save final config
        if !current_section.is_empty() {
            current_config.config_section = Some(current_section.clone());
            self.module_configs.insert(current_section, current_config);
        }
        
        Ok(())
    }
}