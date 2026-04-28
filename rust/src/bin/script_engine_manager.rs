//! OpenSim Next Script Engine Manager
//! Rust implementation of script engine management utilities

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "script-engine-manager")]
#[command(about = "OpenSim Next Script Engine Manager")]
#[command(version = "1.0.0")]
struct Cli {
    /// ScriptEngines directory path
    #[arg(long, default_value = "bin/ScriptEngines")]
    engines_dir: PathBuf,

    /// Configuration file path
    #[arg(long, default_value = "bin/config-include/ScriptEngines.ini")]
    config_file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ScriptEngines directory structure
    Init,
    /// List available script engines
    List,
    /// Enable a specific script engine
    Enable {
        /// Engine name (Native, YEngine, XEngine)
        engine: String,
    },
    /// Disable a specific script engine
    Disable {
        /// Engine name (Native, YEngine, XEngine)
        engine: String,
    },
    /// Show status of all script engines
    Status,
    /// Validate configuration and setup
    Validate,
    /// Create engine configuration template
    Template {
        /// Engine name
        engine: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScriptEngine {
    name: String,
    enabled: bool,
    class_name: String,
    assembly: String,
    description: String,
    performance_rating: u8,
    language_support: Vec<String>,
    features: Vec<String>,
    config_section: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScriptEngineConfig {
    engines: HashMap<String, ScriptEngine>,
    global_settings: HashMap<String, String>,
}

impl Default for ScriptEngineConfig {
    fn default() -> Self {
        let mut engines = HashMap::new();

        // Native Rust/Zig Script Engine
        engines.insert(
            "Native".to_string(),
            ScriptEngine {
                name: "Native".to_string(),
                enabled: true,
                class_name: "OpenSim.Region.ScriptEngine.Native.NativeScriptEngine".to_string(),
                assembly: "OpenSim.Region.ScriptEngine.Native.dll".to_string(),
                description: "High-performance native Rust/Zig script engine".to_string(),
                performance_rating: 10,
                language_support: vec!["LSL".to_string(), "C#".to_string(), "Rust".to_string()],
                features: vec![
                    "Async execution".to_string(),
                    "Memory safety".to_string(),
                    "SIMD optimization".to_string(),
                    "Zero-copy messaging".to_string(),
                ],
                config_section: "Native".to_string(),
            },
        );

        // YEngine (Legacy compatibility)
        engines.insert(
            "YEngine".to_string(),
            ScriptEngine {
                name: "YEngine".to_string(),
                enabled: false,
                class_name: "OpenSim.Region.ScriptEngine.YEngine.YEngine".to_string(),
                assembly: "OpenSim.Region.ScriptEngine.YEngine.dll".to_string(),
                description: "YEngine compatibility layer for legacy scripts".to_string(),
                performance_rating: 7,
                language_support: vec!["LSL".to_string(), "C#".to_string()],
                features: vec![
                    "Legacy compatibility".to_string(),
                    "State persistence".to_string(),
                    "Migration support".to_string(),
                ],
                config_section: "YEngine".to_string(),
            },
        );

        // XEngine (Legacy compatibility)
        engines.insert(
            "XEngine".to_string(),
            ScriptEngine {
                name: "XEngine".to_string(),
                enabled: false,
                class_name: "OpenSim.Region.ScriptEngine.XEngine.XEngine".to_string(),
                assembly: "OpenSim.Region.ScriptEngine.XEngine.dll".to_string(),
                description: "XEngine compatibility layer for legacy scripts".to_string(),
                performance_rating: 5,
                language_support: vec!["LSL".to_string()],
                features: vec![
                    "Legacy compatibility".to_string(),
                    "Basic script execution".to_string(),
                ],
                config_section: "XEngine".to_string(),
            },
        );

        let mut global_settings = HashMap::new();
        global_settings.insert("DefaultEngine".to_string(), "Native".to_string());
        global_settings.insert("EnableScriptDebugging".to_string(), "true".to_string());
        global_settings.insert("MaxScriptMemory".to_string(), "65536".to_string());
        global_settings.insert("ScriptTimeout".to_string(), "30".to_string());

        Self {
            engines,
            global_settings,
        }
    }
}

struct ScriptEngineManager {
    engines_dir: PathBuf,
    config_file: PathBuf,
    config: ScriptEngineConfig,
}

impl ScriptEngineManager {
    fn new(engines_dir: PathBuf, config_file: PathBuf) -> Result<Self> {
        let config = if config_file.exists() {
            Self::load_config(&config_file)?
        } else {
            ScriptEngineConfig::default()
        };

        Ok(Self {
            engines_dir,
            config_file,
            config,
        })
    }

    fn load_config(config_file: &Path) -> Result<ScriptEngineConfig> {
        let content = fs::read_to_string(config_file)?;
        let mut config = ScriptEngineConfig::default();

        // Parse INI-style configuration
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                if current_section == "ScriptEngines" {
                    config
                        .global_settings
                        .insert(key.to_string(), value.to_string());
                } else if let Some(engine) = config.engines.get_mut(&current_section) {
                    match key {
                        "Enabled" => engine.enabled = value.parse().unwrap_or(false),
                        "Class" => engine.class_name = value.to_string(),
                        "Assembly" => engine.assembly = value.to_string(),
                        "Description" => engine.description = value.to_string(),
                        _ => {}
                    }
                }
            }
        }

        Ok(config)
    }

    fn save_config(&self) -> Result<()> {
        let mut content = String::new();

        // Write global settings
        content.push_str("[ScriptEngines]\n");
        for (key, value) in &self.config.global_settings {
            content.push_str(&format!("{} = \"{}\"\n", key, value));
        }
        content.push('\n');

        // Write engine configurations
        for engine in self.config.engines.values() {
            content.push_str(&format!("[{}]\n", engine.config_section));
            content.push_str(&format!("Enabled = {}\n", engine.enabled));
            content.push_str(&format!("Class = \"{}\"\n", engine.class_name));
            content.push_str(&format!("Assembly = \"{}\"\n", engine.assembly));
            content.push_str(&format!("Description = \"{}\"\n", engine.description));
            content.push('\n');
        }

        if let Some(parent) = self.config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.config_file, content)?;
        Ok(())
    }

    fn init_directory_structure(&self) -> Result<()> {
        println!("🚀 Initializing ScriptEngines directory structure...");

        // Create main directories
        let dirs = [
            &self.engines_dir,
            &self.engines_dir.join("Native"),
            &self.engines_dir.join("YEngine"),
            &self.engines_dir.join("XEngine"),
            &self.engines_dir.join("Common"),
            &self.engines_dir.join("Tests"),
        ];

        for dir in &dirs {
            fs::create_dir_all(dir)?;
            println!("  ✅ Created directory: {}", dir.display());
        }

        // Create README files
        self.create_readme_files()?;

        // Save initial configuration
        self.save_config()?;
        println!("  ✅ Created configuration: {}", self.config_file.display());

        println!("\n🎉 ScriptEngines directory structure initialized successfully!");
        Ok(())
    }

    fn create_readme_files(&self) -> Result<()> {
        // Main README
        let main_readme = self.engines_dir.join("README.md");
        fs::write(
            &main_readme,
            include_str!("../../../docs/script_engines_readme.md"),
        )?;

        // Engine-specific READMEs
        let native_readme = self.engines_dir.join("Native/README.md");
        fs::write(
            &native_readme,
            "# Native Script Engine\n\nHigh-performance Rust/Zig script engine implementation.\n",
        )?;

        let yengine_readme = self.engines_dir.join("YEngine/README.md");
        fs::write(
            &yengine_readme,
            "# YEngine Compatibility\n\nYEngine compatibility layer for legacy script migration.\n",
        )?;

        let xengine_readme = self.engines_dir.join("XEngine/README.md");
        fs::write(
            &xengine_readme,
            "# XEngine Compatibility\n\nXEngine compatibility layer for legacy script support.\n",
        )?;

        Ok(())
    }

    fn list_engines(&self) {
        println!("📋 Available Script Engines:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for engine in self.config.engines.values() {
            let status = if engine.enabled {
                "🟢 ENABLED"
            } else {
                "🔴 DISABLED"
            };
            let performance = "⭐".repeat(engine.performance_rating as usize);

            println!("  {} {} ({})", status, engine.name, performance);
            println!("    Description: {}", engine.description);
            println!("    Languages: {}", engine.language_support.join(", "));
            println!("    Features: {}", engine.features.join(", "));
            println!();
        }
    }

    fn enable_engine(&mut self, engine_name: &str) -> Result<()> {
        if let Some(engine) = self.config.engines.get_mut(engine_name) {
            engine.enabled = true;
            self.save_config()?;
            println!("✅ Enabled script engine: {}", engine_name);
        } else {
            return Err(anyhow!("Script engine '{}' not found", engine_name));
        }
        Ok(())
    }

    fn disable_engine(&mut self, engine_name: &str) -> Result<()> {
        if let Some(engine) = self.config.engines.get_mut(engine_name) {
            engine.enabled = false;
            self.save_config()?;
            println!("🔴 Disabled script engine: {}", engine_name);
        } else {
            return Err(anyhow!("Script engine '{}' not found", engine_name));
        }
        Ok(())
    }

    fn show_status(&self) {
        println!("📊 Script Engine Status:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let enabled_count = self.config.engines.values().filter(|e| e.enabled).count();
        let total_count = self.config.engines.len();

        println!("  Engines enabled: {}/{}", enabled_count, total_count);
        println!(
            "  Default engine: {}",
            self.config
                .global_settings
                .get("DefaultEngine")
                .unwrap_or(&"None".to_string())
        );
        println!("  ScriptEngines directory: {}", self.engines_dir.display());
        println!("  Configuration file: {}", self.config_file.display());
        println!();

        for engine in self.config.engines.values() {
            let status_icon = if engine.enabled { "🟢" } else { "🔴" };
            let status_text = if engine.enabled {
                "ENABLED"
            } else {
                "DISABLED"
            };

            println!("  {} {} - {}", status_icon, engine.name, status_text);
            if engine.enabled {
                println!("    Assembly: {}", engine.assembly);
                println!("    Class: {}", engine.class_name);
            }
        }
    }

    fn validate_setup(&self) -> Result<()> {
        println!("🔍 Validating Script Engine Setup:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut issues = Vec::new();

        // Check directory structure
        if !self.engines_dir.exists() {
            issues.push(format!(
                "❌ ScriptEngines directory missing: {}",
                self.engines_dir.display()
            ));
        } else {
            println!("  ✅ ScriptEngines directory exists");
        }

        // Check configuration file
        if !self.config_file.exists() {
            issues.push(format!(
                "❌ Configuration file missing: {}",
                self.config_file.display()
            ));
        } else {
            println!("  ✅ Configuration file exists");
        }

        // Check enabled engines
        let enabled_engines: Vec<_> = self.config.engines.values().filter(|e| e.enabled).collect();
        if enabled_engines.is_empty() {
            issues.push("⚠️  No script engines are enabled".to_string());
        } else {
            println!("  ✅ {} script engine(s) enabled", enabled_engines.len());
        }

        // Check engine directories
        for engine in self.config.engines.values() {
            let engine_dir = self.engines_dir.join(&engine.name);
            if !engine_dir.exists() {
                issues.push(format!(
                    "⚠️  Engine directory missing: {}",
                    engine_dir.display()
                ));
            }
        }

        println!();

        if issues.is_empty() {
            println!("🎉 Validation completed successfully! No issues found.");
        } else {
            println!("⚠️  Validation found {} issue(s):", issues.len());
            for issue in issues {
                println!("  {}", issue);
            }
        }

        Ok(())
    }

    fn create_template(&self, engine_name: &str) -> Result<()> {
        let template_content = match engine_name {
            "Native" => include_str!("../../../templates/native_engine_template.txt"),
            "YEngine" => include_str!("../../../templates/yengine_template.txt"),
            "XEngine" => include_str!("../../../templates/xengine_template.txt"),
            _ => return Err(anyhow!("Unknown engine type: {}", engine_name)),
        };

        let template_file = self
            .engines_dir
            .join(format!("{}/template.ini", engine_name));
        fs::create_dir_all(template_file.parent().unwrap())?;
        fs::write(&template_file, template_content)?;

        println!(
            "✅ Created template for {}: {}",
            engine_name,
            template_file.display()
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut manager = ScriptEngineManager::new(cli.engines_dir, cli.config_file)?;

    match cli.command {
        Commands::Init => manager.init_directory_structure(),
        Commands::List => {
            manager.list_engines();
            Ok(())
        }
        Commands::Enable { engine } => manager.enable_engine(&engine),
        Commands::Disable { engine } => manager.disable_engine(&engine),
        Commands::Status => {
            manager.show_status();
            Ok(())
        }
        Commands::Validate => manager.validate_setup(),
        Commands::Template { engine } => manager.create_template(&engine),
    }
}
