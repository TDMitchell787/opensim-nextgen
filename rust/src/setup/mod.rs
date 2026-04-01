//! OpenSim Next Setup Wizard
//! 
//! Interactive setup system for configuring OpenSim Next instances.
//! Provides step-by-step guidance for both standalone and grid configurations.

pub mod wizard;
pub mod questions;
pub mod config_generator;
pub mod validation;
pub mod scenario_manager;
pub mod file_organizer;

pub use wizard::SetupWizard;
pub use questions::{SetupQuestion, QuestionType, SetupConfig};
pub use config_generator::ConfigGenerator;
pub use validation::Validator;
pub use scenario_manager::{ScenarioManager, UserLevel, ScenarioPrompt};
pub use file_organizer::{FileOrganizer, ArchiveEntry};

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Setup wizard result
#[derive(Debug, Clone)]
pub enum SetupResult {
    Success(SetupConfiguration),
    Cancelled,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfiguration {
    pub name: String,
    pub description: String,
    pub preset: SetupPreset,
    pub config: SetupConfig,
    pub output_directory: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub documentation_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTemplate {
    pub name: String,
    pub description: String,
    pub category: ScenarioCategory,
    pub difficulty: DifficultyLevel,
    pub config_template: SetupConfig,
    pub documentation: String,
    pub startup_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScenarioCategory {
    Standalone,
    GridDevelopment,
    ProductionGrid,
    EducationalGrid,
    TestingEnvironment,
    EnterpriseDeployment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Setup presets for quick configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SetupPreset {
    Standalone,
    GridRegion,
    GridRobust,
    Development,
    Production,
    CustomScenario(String),
}

impl SetupPreset {
    pub fn description(&self) -> String {
        match self {
            Self::Standalone => "Complete standalone grid with all services in one instance".to_string(),
            Self::GridRegion => "Region server connecting to an existing grid".to_string(),
            Self::GridRobust => "Grid services server (Robust) for multi-region grids".to_string(),
            Self::Development => "Development setup with debugging enabled and relaxed security".to_string(),
            Self::Production => "Production-ready setup with security hardening and optimization".to_string(),
            Self::CustomScenario(name) => format!("Custom scenario: {}", name),
        }
    }
}