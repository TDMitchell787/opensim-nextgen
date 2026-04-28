//! Main setup wizard orchestrator
//!
//! Handles the interactive setup flow, question presentation,
//! and coordination between different setup components.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{self, Write};
use tracing::{error, info, warn};

use crate::setup::{
    config_generator::ConfigGenerator,
    questions::{get_setup_questions, QuestionType, SetupConfig, SetupQuestion},
    validation::Validator,
    SetupPreset, SetupResult,
};

/// Main setup wizard that orchestrates the configuration process
pub struct SetupWizard {
    questions: Vec<SetupQuestion>,
    answers: HashMap<String, String>,
    config: SetupConfig,
    interactive: bool,
    preset: Option<SetupPreset>,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new() -> Self {
        Self {
            questions: get_setup_questions(),
            answers: HashMap::new(),
            config: SetupConfig::default(),
            interactive: true,
            preset: None,
        }
    }

    /// Create wizard with a specific preset
    pub fn with_preset(preset: SetupPreset) -> Self {
        let mut wizard = Self::new();
        wizard.preset = Some(preset);
        wizard
    }

    /// Run the setup wizard in non-interactive mode
    pub fn non_interactive(mut self) -> Self {
        self.interactive = false;
        self
    }

    /// Run the complete setup wizard
    pub async fn run(mut self) -> Result<SetupResult> {
        info!("Starting OpenSim Next Setup Wizard");

        if self.interactive {
            self.print_welcome();
        }

        // Apply preset defaults if specified
        if let Some(preset) = self.preset.clone() {
            self.apply_preset_defaults(&preset)?;
        }

        // Ask all questions
        if let Err(e) = self.ask_questions().await {
            error!("Setup failed: {}", e);
            return Ok(SetupResult::Error(e.to_string()));
        }

        // Build final configuration
        self.build_config()?;

        // Validate configuration
        let validator = Validator::new();
        if let Err(e) = validator.validate(&self.config) {
            error!("Configuration validation failed: {}", e);
            return Ok(SetupResult::Error(format!("Validation failed: {}", e)));
        }

        // Generate configuration files
        let generator = ConfigGenerator::new();
        if let Err(e) = generator.generate(&self.config).await {
            error!("Failed to generate configuration files: {}", e);
            return Ok(SetupResult::Error(format!(
                "Config generation failed: {}",
                e
            )));
        }

        if self.interactive {
            self.print_success();
        }

        info!("Setup wizard completed successfully");

        // Create SetupConfiguration from config
        let setup_configuration = crate::setup::SetupConfiguration {
            name: self.config.grid_name.clone(),
            description: format!(
                "Setup generated on {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M")
            ),
            preset: self.config.preset.clone(),
            config: self.config.clone(),
            output_directory: std::path::PathBuf::from("./"),
            created_at: chrono::Utc::now(),
            documentation_path: std::path::PathBuf::from("./README.md"),
        };

        Ok(SetupResult::Success(setup_configuration))
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("\n{}", "=".repeat(60));
        println!("🌟 Welcome to OpenSim Next Setup Wizard 🌟");
        println!("{}", "=".repeat(60));
        println!();
        println!("This wizard will guide you through setting up your OpenSim Next instance.");
        println!("You can press Enter to accept default values shown in [brackets].");
        println!("Type 'quit' at any time to exit the setup.");
        println!();

        if let Some(preset) = &self.preset {
            println!(
                "📋 Using preset: {} - {}",
                format!("{:?}", preset),
                preset.description()
            );
            println!();
        }
    }

    /// Apply preset defaults to configuration
    fn apply_preset_defaults(&mut self, preset: &SetupPreset) -> Result<()> {
        match preset {
            SetupPreset::Standalone => {
                self.answers
                    .insert("setup_mode".to_string(), "standalone".to_string());
                self.answers
                    .insert("database_type".to_string(), "sqlite".to_string());
                self.answers
                    .insert("grid_mode".to_string(), "false".to_string());
            }
            SetupPreset::GridRegion => {
                self.answers
                    .insert("setup_mode".to_string(), "grid-region".to_string());
                self.answers
                    .insert("grid_mode".to_string(), "true".to_string());
                self.answers
                    .insert("database_type".to_string(), "postgresql".to_string());
            }
            SetupPreset::GridRobust => {
                self.answers
                    .insert("setup_mode".to_string(), "grid-robust".to_string());
                self.answers
                    .insert("database_type".to_string(), "postgresql".to_string());
            }
            SetupPreset::Development => {
                self.answers
                    .insert("log_level".to_string(), "DEBUG".to_string());
                self.answers
                    .insert("enable_monitoring".to_string(), "true".to_string());
                self.answers
                    .insert("https_enabled".to_string(), "false".to_string());
            }
            SetupPreset::Production => {
                self.answers
                    .insert("log_level".to_string(), "WARN".to_string());
                self.answers
                    .insert("enable_monitoring".to_string(), "true".to_string());
                self.answers
                    .insert("https_enabled".to_string(), "true".to_string());
                self.answers
                    .insert("database_type".to_string(), "postgresql".to_string());
            }
            SetupPreset::CustomScenario(scenario_name) => {
                self.answers
                    .insert("setup_mode".to_string(), "custom".to_string());
                self.answers
                    .insert("preset_name".to_string(), scenario_name.clone());
                self.answers
                    .insert("log_level".to_string(), "INFO".to_string());
                self.answers
                    .insert("enable_monitoring".to_string(), "true".to_string());
            }
        }
        Ok(())
    }

    /// Ask all relevant questions
    async fn ask_questions(&mut self) -> Result<()> {
        for question in self.questions.clone() {
            // Check if question should be asked based on dependencies
            if !self.should_ask_question(&question) {
                continue;
            }

            // Skip if already answered by preset
            if self.answers.contains_key(&question.key) && !self.interactive {
                continue;
            }

            let answer = if self.interactive {
                self.ask_question(&question)?
            } else {
                // In non-interactive mode, use default or existing answer
                self.answers
                    .get(&question.key)
                    .cloned()
                    .unwrap_or_else(|| question.default_value.clone())
            };

            // Validate answer
            if let Some(validator) = question.validator {
                if let Err(e) = validator(&answer) {
                    if self.interactive {
                        println!("❌ Invalid input: {}", e);
                        continue; // Ask again
                    } else {
                        return Err(anyhow!("Invalid configuration for {}: {}", question.key, e));
                    }
                }
            }

            self.answers.insert(question.key.clone(), answer);
        }

        Ok(())
    }

    /// Check if a question should be asked based on dependencies
    fn should_ask_question(&self, question: &SetupQuestion) -> bool {
        if let (Some(depends_on), Some(depends_value)) =
            (&question.depends_on, &question.depends_value)
        {
            if let Some(answer) = self.answers.get(depends_on) {
                return answer == depends_value;
            }
            return false;
        }
        true
    }

    /// Ask a single question interactively
    fn ask_question(&self, question: &SetupQuestion) -> Result<String> {
        loop {
            // Print question prompt
            if let Some(description) = &question.description {
                println!("📝 {}", description);
            }

            let prompt = match &question.question_type {
                QuestionType::Choice(choices) => {
                    println!("   Choices: {}", choices.join(", "));
                    format!("{} [{}]: ", question.prompt, question.default_value)
                }
                QuestionType::Boolean => {
                    format!(
                        "{} (true/false) [{}]: ",
                        question.prompt, question.default_value
                    )
                }
                _ => {
                    format!("{} [{}]: ", question.prompt, question.default_value)
                }
            };

            print!("{}", prompt);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Handle quit command
            if input.to_lowercase() == "quit" {
                return Err(anyhow!("Setup cancelled by user"));
            }

            // Use default if empty
            let answer = if input.is_empty() {
                question.default_value.clone()
            } else {
                input.to_string()
            };

            // Validate based on question type
            match self.validate_answer(&question.question_type, &answer) {
                Ok(_) => {
                    // Additional custom validation
                    if let Some(validator) = question.validator {
                        match validator(&answer) {
                            Ok(_) => return Ok(answer),
                            Err(e) => {
                                println!("❌ {}", e);
                                continue;
                            }
                        }
                    }
                    return Ok(answer);
                }
                Err(e) => {
                    println!("❌ {}", e);
                    continue;
                }
            }
        }
    }

    /// Validate answer based on question type
    fn validate_answer(&self, question_type: &QuestionType, answer: &str) -> Result<()> {
        match question_type {
            QuestionType::StringNotEmpty => {
                if answer.trim().is_empty() {
                    return Err(anyhow!("Value cannot be empty"));
                }
            }
            QuestionType::Integer => {
                answer
                    .parse::<i32>()
                    .map_err(|_| anyhow!("Must be a valid integer"))?;
            }
            QuestionType::Port => {
                let port: u16 = answer
                    .parse()
                    .map_err(|_| anyhow!("Must be a valid port number (1-65535)"))?;
                if port == 0 {
                    return Err(anyhow!("Port cannot be 0"));
                }
            }
            QuestionType::Boolean => {
                answer
                    .to_lowercase()
                    .parse::<bool>()
                    .map_err(|_| anyhow!("Must be 'true' or 'false'"))?;
            }
            QuestionType::IpAddress => {
                if answer != "0.0.0.0" && answer != "SYSTEMIP" {
                    answer
                        .parse::<std::net::IpAddr>()
                        .map_err(|_| anyhow!("Must be a valid IP address"))?;
                }
            }
            QuestionType::Choice(choices) => {
                if !choices.contains(&answer.to_string()) {
                    return Err(anyhow!("Must be one of: {}", choices.join(", ")));
                }
            }
            QuestionType::Uuid => {
                uuid::Uuid::parse_str(answer).map_err(|_| anyhow!("Must be a valid UUID"))?;
            }
            _ => {} // No validation for generic strings and database URLs
        }
        Ok(())
    }

    /// Build final configuration from answers
    fn build_config(&mut self) -> Result<()> {
        // Map answers to configuration structure
        if let Some(value) = self.answers.get("setup_mode") {
            self.config.setup_mode = value.clone();
        }

        if let Some(value) = self.answers.get("region_name") {
            self.config.region_name = value.clone();
        }

        if let Some(value) = self.answers.get("region_location") {
            self.config.region_location = value.clone();
        }

        if let Some(value) = self.answers.get("internal_address") {
            self.config.internal_address = value.clone();
        }

        if let Some(value) = self.answers.get("internal_port") {
            self.config.internal_port =
                value.parse().map_err(|_| anyhow!("Invalid port number"))?;
        }

        if let Some(value) = self.answers.get("external_hostname") {
            self.config.external_hostname = value.clone();
        }

        if let Some(value) = self.answers.get("resolve_address") {
            self.config.resolve_address = value
                .parse()
                .map_err(|_| anyhow!("Invalid boolean value"))?;
        }

        if let Some(value) = self.answers.get("database_type") {
            self.config.database_type = value.clone();
        }

        if let Some(value) = self.answers.get("database_url") {
            self.config.database_url = value.clone();
        }

        if let Some(value) = self.answers.get("grid_uri") {
            self.config.grid_uri = Some(value.clone());
        }

        if let Some(value) = self.answers.get("physics_engine") {
            self.config.physics_engine = value.clone();
        }

        if let Some(value) = self.answers.get("script_engine") {
            self.config.script_engine = value.clone();
        }

        if let Some(value) = self.answers.get("log_level") {
            self.config.log_level = value.clone();
        }

        if let Some(value) = self.answers.get("enable_monitoring") {
            self.config.enable_monitoring = value
                .parse()
                .map_err(|_| anyhow!("Invalid boolean value"))?;
        }

        // Set grid mode based on setup mode
        self.config.grid_mode = self.config.setup_mode != "standalone";

        // Generate new UUID if not provided
        if self.config.region_uuid.is_empty() {
            self.config.region_uuid = uuid::Uuid::new_v4().to_string();
        }

        Ok(())
    }

    /// Print success message
    fn print_success(&self) {
        println!("\n{}", "=".repeat(60));
        println!("🎉 Setup Completed Successfully! 🎉");
        println!("{}", "=".repeat(60));
        println!();
        println!("Your OpenSim Next instance has been configured:");
        println!("• Setup mode: {}", self.config.setup_mode);
        println!("• Region: {}", self.config.region_name);
        println!("• Location: {}", self.config.region_location);
        println!("• Database: {}", self.config.database_type);
        println!("• Physics: {}", self.config.physics_engine);
        println!();
        println!("Configuration files have been generated in the config directory.");
        println!("You can now start your OpenSim Next server!");
        println!();
        println!("To start the server, run:");
        println!("  cargo run --bin opensim-next");
        println!();
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = SetupWizard::new();
        assert!(!wizard.questions.is_empty());
        assert!(wizard.interactive);
    }

    #[test]
    fn test_preset_application() {
        let mut wizard = SetupWizard::with_preset(SetupPreset::Standalone);
        wizard
            .apply_preset_defaults(&SetupPreset::Standalone)
            .unwrap();

        assert_eq!(wizard.answers.get("setup_mode").unwrap(), "standalone");
        assert_eq!(wizard.answers.get("database_type").unwrap(), "sqlite");
    }

    #[test]
    fn test_answer_validation() {
        let wizard = SetupWizard::new();

        // Test port validation
        assert!(wizard.validate_answer(&QuestionType::Port, "9000").is_ok());
        assert!(wizard.validate_answer(&QuestionType::Port, "0").is_err());
        assert!(wizard
            .validate_answer(&QuestionType::Port, "invalid")
            .is_err());

        // Test boolean validation
        assert!(wizard
            .validate_answer(&QuestionType::Boolean, "true")
            .is_ok());
        assert!(wizard
            .validate_answer(&QuestionType::Boolean, "false")
            .is_ok());
        assert!(wizard
            .validate_answer(&QuestionType::Boolean, "invalid")
            .is_err());
    }
}
