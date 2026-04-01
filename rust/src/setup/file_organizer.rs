use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::setup::{SetupConfiguration, SetupConfig, ScenarioTemplate, ScenarioCategory, DifficultyLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: PathBuf,
    pub metadata: ArchiveMetadata,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub size_bytes: u64,
    pub status: ArchiveStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub description: String,
    pub category: ScenarioCategory,
    pub difficulty: DifficultyLevel,
    pub features: Vec<String>,
    pub tags: Vec<String>,
    pub version: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveStatus {
    Available,
    InUse,
    Archived,
    Corrupted,
}

pub struct FileOrganizer {
    base_path: PathBuf,
    archives: HashMap<String, ArchiveEntry>,
}

impl FileOrganizer {
    pub fn new(base_path: PathBuf) -> Result<Self, std::io::Error> {
        // Create directory structure if it doesn't exist
        fs::create_dir_all(&base_path)?;
        fs::create_dir_all(base_path.join("templates"))?;
        fs::create_dir_all(base_path.join("templates/beginner"))?;
        fs::create_dir_all(base_path.join("templates/intermediate"))?;
        fs::create_dir_all(base_path.join("templates/advanced"))?;
        fs::create_dir_all(base_path.join("saved-configs/by-name"))?;
        fs::create_dir_all(base_path.join("saved-configs/by-category/grids"))?;
        fs::create_dir_all(base_path.join("saved-configs/by-category/standalone"))?;
        fs::create_dir_all(base_path.join("saved-configs/by-category/development"))?;
        fs::create_dir_all(base_path.join("active-instances"))?;

        let mut organizer = Self {
            base_path,
            archives: HashMap::new(),
        };

        organizer.scan_archives()?;
        Ok(organizer)
    }

    pub fn save_configuration(&mut self, config: &SetupConfiguration) -> Result<PathBuf, std::io::Error> {
        let safe_name = sanitize_filename(&config.name);
        let config_path = self.base_path
            .join("saved-configs/by-name")
            .join(&safe_name);

        // Create configuration directory structure
        fs::create_dir_all(&config_path)?;
        fs::create_dir_all(config_path.join("configs"))?;
        fs::create_dir_all(config_path.join("configs/Regions"))?;
        fs::create_dir_all(config_path.join("configs/config-include"))?;
        fs::create_dir_all(config_path.join("documentation"))?;
        fs::create_dir_all(config_path.join("scripts"))?;

        // Save metadata
        let metadata_path = config_path.join("metadata.json");
        let metadata_content = serde_json::to_string_pretty(config)?;
        fs::write(metadata_path, metadata_content)?;

        // Create archive entry
        let archive_entry = ArchiveEntry {
            name: config.name.clone(),
            path: config_path.clone(),
            metadata: ArchiveMetadata {
                description: config.description.clone(),
                category: ScenarioCategory::from_preset(&config.preset),
                difficulty: DifficultyLevel::from_preset(&config.preset),
                features: vec![], // TODO: Extract from config
                tags: vec![], // TODO: Generate from config
                version: "1.0.0".to_string(),
                created_by: "Setup Wizard".to_string(),
            },
            created_at: config.created_at,
            last_accessed: None,
            size_bytes: 0, // TODO: Calculate actual size
            status: ArchiveStatus::Available,
        };

        self.archives.insert(safe_name, archive_entry);
        
        // Create category symlink
        self.create_category_link(config)?;

        Ok(config_path)
    }

    pub fn load_configuration(&mut self, name: &str) -> Result<SetupConfiguration, std::io::Error> {
        let safe_name = sanitize_filename(name);
        let config_path = self.base_path
            .join("saved-configs/by-name")
            .join(&safe_name)
            .join("metadata.json");

        let metadata_content = fs::read_to_string(config_path)?;
        let config: SetupConfiguration = serde_json::from_str(&metadata_content)?;

        // Update last accessed time
        if let Some(entry) = self.archives.get_mut(&safe_name) {
            entry.last_accessed = Some(Utc::now());
        }

        Ok(config)
    }

    pub fn list_templates(&self, difficulty: Option<DifficultyLevel>) -> Result<Vec<ScenarioTemplate>, std::io::Error> {
        let mut templates = Vec::new();

        for difficulty_dir in ["beginner", "intermediate", "advanced"] {
            if let Some(filter_difficulty) = &difficulty {
                let dir_difficulty = match difficulty_dir {
                    "beginner" => DifficultyLevel::Beginner,
                    "intermediate" => DifficultyLevel::Intermediate,
                    "advanced" => DifficultyLevel::Advanced,
                    _ => continue,
                };
                if *filter_difficulty != dir_difficulty {
                    continue;
                }
            }

            let templates_dir = self.base_path.join("templates").join(difficulty_dir);
            if templates_dir.exists() {
                for entry in fs::read_dir(templates_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let metadata_path = entry.path().join("metadata.json");
                        if metadata_path.exists() {
                            if let Ok(template) = self.load_template_metadata(&metadata_path) {
                                templates.push(template);
                            }
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    pub fn list_saved_configurations(&self) -> Vec<&ArchiveEntry> {
        self.archives.values().collect()
    }

    pub fn delete_configuration(&mut self, name: &str) -> Result<(), std::io::Error> {
        let safe_name = sanitize_filename(name);
        let config_path = self.base_path.join("saved-configs/by-name").join(&safe_name);
        
        if config_path.exists() {
            fs::remove_dir_all(config_path)?;
            self.archives.remove(&safe_name);
        }

        Ok(())
    }

    pub fn create_instance(&self, config_name: &str, instance_name: &str) -> Result<PathBuf, std::io::Error> {
        let safe_config_name = sanitize_filename(config_name);
        let safe_instance_name = sanitize_filename(instance_name);
        
        let source_path = self.base_path.join("saved-configs/by-name").join(&safe_config_name);
        let instance_path = self.base_path.join("active-instances").join(&safe_instance_name);

        if !source_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Configuration not found"
            ));
        }

        // Copy configuration to active instance
        copy_dir_recursive(&source_path, &instance_path)?;

        Ok(instance_path)
    }

    fn scan_archives(&mut self) -> Result<(), std::io::Error> {
        let configs_dir = self.base_path.join("saved-configs/by-name");
        if !configs_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(configs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    if let Ok(config) = self.load_configuration_from_path(&metadata_path) {
                        let archive_entry = ArchiveEntry {
                            name: config.name.clone(),
                            path: entry.path(),
                            metadata: ArchiveMetadata {
                                description: config.description,
                                category: ScenarioCategory::from_preset(&config.preset),
                                difficulty: DifficultyLevel::from_preset(&config.preset),
                                features: vec![],
                                tags: vec![],
                                version: "1.0.0".to_string(),
                                created_by: "Setup Wizard".to_string(),
                            },
                            created_at: config.created_at,
                            last_accessed: None,
                            size_bytes: calculate_dir_size(&entry.path()).unwrap_or(0),
                            status: ArchiveStatus::Available,
                        };

                        self.archives.insert(
                            entry.file_name().to_string_lossy().to_string(),
                            archive_entry
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn create_category_link(&self, config: &SetupConfiguration) -> Result<(), std::io::Error> {
        let category = match config.preset {
            crate::setup::SetupPreset::Standalone => "standalone",
            crate::setup::SetupPreset::GridRegion | 
            crate::setup::SetupPreset::GridRobust => "grids",
            crate::setup::SetupPreset::Development => "development",
            _ => "grids",
        };

        let safe_name = sanitize_filename(&config.name);
        let source_path = self.base_path.join("saved-configs/by-name").join(&safe_name);
        let link_path = self.base_path
            .join("saved-configs/by-category")
            .join(category)
            .join(&safe_name);

        // Create symlink (Unix) or directory junction (Windows)
        #[cfg(unix)]
        std::os::unix::fs::symlink(&source_path, &link_path)?;
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&source_path, &link_path)?;

        Ok(())
    }

    fn load_template_metadata(&self, path: &Path) -> Result<ScenarioTemplate, std::io::Error> {
        // Implementation for loading template metadata
        // This would parse the template's metadata.json and create a ScenarioTemplate
        let metadata_content = fs::read_to_string(path)?;
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content)?;
        
        // Create a basic ScenarioTemplate from metadata
        Ok(ScenarioTemplate {
            name: metadata["name"].as_str().unwrap_or("Unknown").to_string(),
            description: metadata["description"].as_str().unwrap_or("").to_string(),
            category: ScenarioCategory::Standalone, // TODO: Parse from metadata
            difficulty: DifficultyLevel::Beginner,   // TODO: Parse from metadata
            config_template: SetupConfig::default(),
            documentation: String::new(),
            startup_script: String::new(),
        })
    }

    fn load_configuration_from_path(&self, path: &Path) -> Result<SetupConfiguration, std::io::Error> {
        let metadata_content = fs::read_to_string(path)?;
        let config: SetupConfiguration = serde_json::from_str(&metadata_content)?;
        Ok(config)
    }
}

impl ScenarioCategory {
    fn from_preset(preset: &crate::setup::SetupPreset) -> Self {
        match preset {
            crate::setup::SetupPreset::Standalone => ScenarioCategory::Standalone,
            crate::setup::SetupPreset::Development => ScenarioCategory::TestingEnvironment,
            crate::setup::SetupPreset::Production => ScenarioCategory::EnterpriseDeployment,
            _ => ScenarioCategory::GridDevelopment,
        }
    }
}

impl DifficultyLevel {
    fn from_preset(preset: &crate::setup::SetupPreset) -> Self {
        match preset {
            crate::setup::SetupPreset::Standalone => DifficultyLevel::Beginner,
            crate::setup::SetupPreset::Development => DifficultyLevel::Intermediate,
            crate::setup::SetupPreset::Production => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Intermediate,
        }
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn calculate_dir_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut size = 0;
    
    if path.is_file() {
        size += fs::metadata(path)?.len();
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            size += calculate_dir_size(&entry.path())?;
        }
    }
    
    Ok(size)
}