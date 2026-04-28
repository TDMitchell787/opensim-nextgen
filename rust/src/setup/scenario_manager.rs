use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::setup::{DifficultyLevel, ScenarioCategory, ScenarioTemplate, SetupConfig, SetupPreset};

pub struct ScenarioManager {
    templates: HashMap<String, ScenarioTemplate>,
    base_path: PathBuf,
}

impl ScenarioManager {
    pub fn new(base_path: PathBuf) -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            base_path,
        };

        manager.load_builtin_scenarios();
        manager
    }

    pub fn get_scenario_prompt(&self, user_level: UserLevel) -> ScenarioPrompt {
        match user_level {
            UserLevel::Beginner => self.create_beginner_prompt(),
            UserLevel::Intermediate => self.create_intermediate_prompt(),
            UserLevel::Advanced => self.create_advanced_prompt(),
            UserLevel::Expert => self.create_expert_prompt(),
        }
    }

    pub fn list_scenarios_by_category(&self, category: ScenarioCategory) -> Vec<&ScenarioTemplate> {
        self.templates
            .values()
            .filter(|template| template.category == category)
            .collect()
    }

    pub fn list_scenarios_by_difficulty(
        &self,
        difficulty: DifficultyLevel,
    ) -> Vec<&ScenarioTemplate> {
        self.templates
            .values()
            .filter(|template| template.difficulty == difficulty)
            .collect()
    }

    pub fn get_template(&self, name: &str) -> Option<&ScenarioTemplate> {
        self.templates.get(name)
    }

    pub fn apply_template(
        &self,
        template_name: &str,
        customizations: HashMap<String, String>,
    ) -> Result<SetupConfig, String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or("Template not found")?;

        let mut config = template.config_template.clone();

        // Apply customizations
        for (key, value) in customizations {
            self.apply_customization(&mut config, &key, &value)?;
        }

        Ok(config)
    }

    fn load_builtin_scenarios(&mut self) {
        // Quick Start Standalone
        self.templates.insert("quickstart-standalone".to_string(), ScenarioTemplate {
            name: "Quick Start Standalone".to_string(),
            description: "Perfect for first-time users - a single region setup that just works".to_string(),
            category: ScenarioCategory::Standalone,
            difficulty: DifficultyLevel::Beginner,
            config_template: self.create_quickstart_config(),
            documentation: "# Quick Start Standalone Setup\n\nPerfect for first-time users - a single region setup that just works.".to_string(),
            startup_script: "#!/bin/bash\necho 'Starting Quick Start setup...'\n# Auto-generated startup script".to_string(),
        });

        // Small Grid 2x2
        self.templates.insert(
            "small-grid-2x2".to_string(),
            ScenarioTemplate {
                name: "Small Grid (2x2)".to_string(),
                description: "Four connected regions perfect for learning grid management"
                    .to_string(),
                category: ScenarioCategory::GridDevelopment,
                difficulty: DifficultyLevel::Beginner,
                config_template: self.create_small_grid_config(),
                documentation: self.create_small_grid_docs(),
                startup_script: self.create_small_grid_script(),
            },
        );

        // Creative Sandbox
        self.templates.insert(
            "creative-sandbox".to_string(),
            ScenarioTemplate {
                name: "Creative Sandbox".to_string(),
                description: "Enhanced environment for artists, builders, and content creators"
                    .to_string(),
                category: ScenarioCategory::EducationalGrid,
                difficulty: DifficultyLevel::Intermediate,
                config_template: self.create_creative_config(),
                documentation: self.create_creative_docs(),
                startup_script: self.create_creative_script(),
            },
        );

        // Production Enterprise
        self.templates.insert(
            "production-enterprise".to_string(),
            ScenarioTemplate {
                name: "Production Enterprise".to_string(),
                description: "Full enterprise deployment with all advanced features".to_string(),
                category: ScenarioCategory::EnterpriseDeployment,
                difficulty: DifficultyLevel::Expert,
                config_template: self.create_enterprise_config(),
                documentation: self.create_enterprise_docs(),
                startup_script: self.create_enterprise_script(),
            },
        );
    }

    fn create_beginner_prompt(&self) -> ScenarioPrompt {
        ScenarioPrompt {
            title: "🌟 Welcome to OpenSim!".to_string(),
            description: "Let's get you started with your first virtual world. Choose a pre-configured template that's perfect for beginners.".to_string(),
            level: UserLevel::Beginner,
            options: vec![
                PromptOption {
                    id: "quickstart".to_string(),
                    title: "🚀 Quick Start (Recommended)".to_string(),
                    description: "Single region, auto-configured, ready in 2 minutes".to_string(),
                    template: "quickstart-standalone".to_string(),
                    difficulty_indicator: "🟢 Beginner".to_string(),
                },
                PromptOption {
                    id: "small-grid".to_string(),
                    title: "🏘️ Small Grid (2x2)".to_string(),
                    description: "Four connected regions for exploring grid features".to_string(),
                    template: "small-grid-2x2".to_string(),
                    difficulty_indicator: "🟢 Beginner".to_string(),
                },
            ],
            customization_allowed: false,
            expert_mode_available: true,
        }
    }

    fn create_intermediate_prompt(&self) -> ScenarioPrompt {
        ScenarioPrompt {
            title: "🎯 Intermediate Setup".to_string(),
            description: "You have some OpenSim experience. Choose a template and customize it to your needs.".to_string(),
            level: UserLevel::Intermediate,
            options: vec![
                PromptOption {
                    id: "creative".to_string(),
                    title: "🎨 Creative Sandbox".to_string(),
                    description: "Enhanced building and scripting environment".to_string(),
                    template: "creative-sandbox".to_string(),
                    difficulty_indicator: "🟡 Intermediate".to_string(),
                },
                PromptOption {
                    id: "community".to_string(),
                    title: "👥 Community Grid".to_string(),
                    description: "Social features and group management tools".to_string(),
                    template: "community-grid".to_string(),
                    difficulty_indicator: "🟡 Intermediate".to_string(),
                },
                PromptOption {
                    id: "hypergrid".to_string(),
                    title: "🌐 Hypergrid Enabled".to_string(),
                    description: "Connect with other OpenSim grids worldwide".to_string(),
                    template: "hypergrid-enabled".to_string(),
                    difficulty_indicator: "🟡 Intermediate".to_string(),
                },
            ],
            customization_allowed: true,
            expert_mode_available: true,
        }
    }

    fn create_advanced_prompt(&self) -> ScenarioPrompt {
        ScenarioPrompt {
            title: "⚡ Advanced Configuration".to_string(),
            description: "You're experienced with OpenSim. Choose from professional-grade templates or create custom configurations.".to_string(),
            level: UserLevel::Advanced,
            options: vec![
                PromptOption {
                    id: "production".to_string(),
                    title: "🏢 Production Enterprise".to_string(),
                    description: "Full enterprise deployment with clustering and monitoring".to_string(),
                    template: "production-enterprise".to_string(),
                    difficulty_indicator: "🔴 Advanced".to_string(),
                },
                PromptOption {
                    id: "multi-physics".to_string(),
                    title: "⚗️ Multi-Physics Demo".to_string(),
                    description: "Showcase different physics engines and capabilities".to_string(),
                    template: "multi-physics-demo".to_string(),
                    difficulty_indicator: "🔴 Advanced".to_string(),
                },
                PromptOption {
                    id: "custom-economy".to_string(),
                    title: "💰 Custom Economy".to_string(),
                    description: "Full economy system with marketplace and currency".to_string(),
                    template: "custom-economy".to_string(),
                    difficulty_indicator: "🔴 Advanced".to_string(),
                },
            ],
            customization_allowed: true,
            expert_mode_available: true,
        }
    }

    fn create_expert_prompt(&self) -> ScenarioPrompt {
        ScenarioPrompt {
            title: "🧙 Expert Mode".to_string(),
            description: "Full control over every aspect of your OpenSim deployment. Use advanced scripting and custom scenarios.".to_string(),
            level: UserLevel::Expert,
            options: vec![
                PromptOption {
                    id: "custom-wizard".to_string(),
                    title: "🔧 Custom Configuration Wizard".to_string(),
                    description: "Step-by-step wizard with full customization options".to_string(),
                    template: "custom-wizard".to_string(),
                    difficulty_indicator: "🟣 Expert".to_string(),
                },
                PromptOption {
                    id: "scenario-script".to_string(),
                    title: "📜 Scenario Scripting".to_string(),
                    description: "Create configurations using advanced scripting interface".to_string(),
                    template: "scenario-script".to_string(),
                    difficulty_indicator: "🟣 Expert".to_string(),
                },
                PromptOption {
                    id: "import-config".to_string(),
                    title: "📥 Import Configuration".to_string(),
                    description: "Import and modify existing OpenSim configurations".to_string(),
                    template: "import-config".to_string(),
                    difficulty_indicator: "🟣 Expert".to_string(),
                },
            ],
            customization_allowed: true,
            expert_mode_available: false, // Already in expert mode
        }
    }

    fn apply_customization(
        &self,
        config: &mut SetupConfig,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        match key {
            "grid_name" => config.grid_name = value.to_string(),
            "admin_first_name" => config.admin_first_name = value.to_string(),
            "admin_last_name" => config.admin_last_name = value.to_string(),
            "region_count" => {
                config.region_count = value.parse().map_err(|_| "Invalid region count")?;
            }
            "enable_hypergrid" => {
                config.enable_hypergrid = value.parse().map_err(|_| "Invalid hypergrid setting")?;
            }
            "enable_ossl" => {
                config.enable_ossl = value.parse().map_err(|_| "Invalid OSSL setting")?;
            }
            _ => return Err(format!("Unknown customization key: {}", key)),
        }
        Ok(())
    }

    // Template creation methods
    fn create_quickstart_config(&self) -> SetupConfig {
        let mut config = SetupConfig::default();
        config.grid_name = "My First OpenSim World".to_string();
        config.admin_first_name = "NewUser".to_string();
        config.admin_last_name = "Admin".to_string();
        config.admin_email = "admin@localhost".to_string();
        config.admin_password = "admin123".to_string();
        config.database_provider = "SQLite".to_string();
        config.database_connection = "URI=file:opensim.db,version=3".to_string();
        config.http_port = 8080;
        config.region_port_start = 9000;
        config.region_count = 1;
        config.enable_hypergrid = false;
        config.enable_ossl = true;
        config.ossl_threat_level = "Moderate".to_string();
        config.external_hostname = "127.0.0.1".to_string();
        config.preset = SetupPreset::Standalone;
        config
    }

    fn create_small_grid_config(&self) -> SetupConfig {
        let mut config = self.create_quickstart_config();
        config.grid_name = "My Small Grid".to_string();
        config.region_count = 4;
        config.enable_hypergrid = true;
        config.ossl_threat_level = "High".to_string();
        config.preset = SetupPreset::GridRegion;
        config
    }

    fn create_creative_config(&self) -> SetupConfig {
        let mut config = self.create_quickstart_config();
        config.grid_name = "Creative Sandbox".to_string();
        config.region_count = 2;
        config.enable_ossl = true;
        config.ossl_threat_level = "VeryHigh".to_string();
        config.database_provider = "PostgreSQL".to_string();
        config.database_connection =
            "Host=localhost;Database=opensim_creative;Username=opensim;Password=secure_password"
                .to_string();
        config.preset = SetupPreset::Development;
        config
    }

    fn create_enterprise_config(&self) -> SetupConfig {
        let mut config = self.create_quickstart_config();
        config.grid_name = "Enterprise Grid".to_string();
        config.region_count = 16;
        config.enable_hypergrid = true;
        config.enable_ossl = true;
        config.ossl_threat_level = "VeryHigh".to_string();
        config.database_provider = "PostgreSQL".to_string();
        config.database_connection = "Host=localhost;Database=opensim_enterprise;Username=opensim;Password=enterprise_password".to_string();
        config.preset = SetupPreset::Production;
        config
    }

    // Documentation creation methods (simplified)
    fn create_small_grid_docs(&self) -> String {
        "# Small Grid (2x2) Setup\n\nFour connected regions perfect for learning grid management."
            .to_string()
    }

    fn create_creative_docs(&self) -> String {
        "# Creative Sandbox Setup\n\nEnhanced environment for artists, builders, and content creators.".to_string()
    }

    fn create_enterprise_docs(&self) -> String {
        "# Enterprise Production Setup\n\nFull enterprise deployment with all advanced features."
            .to_string()
    }

    // Script creation methods (simplified)
    fn create_small_grid_script(&self) -> String {
        "#!/bin/bash\necho 'Starting Small Grid...'\n# Grid startup script".to_string()
    }

    fn create_creative_script(&self) -> String {
        "#!/bin/bash\necho 'Starting Creative Sandbox...'\n# Creative setup script".to_string()
    }

    fn create_enterprise_script(&self) -> String {
        "#!/bin/bash\necho 'Starting Enterprise Grid...'\n# Enterprise startup script".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioPrompt {
    pub title: String,
    pub description: String,
    pub level: UserLevel,
    pub options: Vec<PromptOption>,
    pub customization_allowed: bool,
    pub expert_mode_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOption {
    pub id: String,
    pub title: String,
    pub description: String,
    pub template: String,
    pub difficulty_indicator: String,
}
