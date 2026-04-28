//! OpenSim Next Script Engine Test Suite
//! Comprehensive testing for script engine setup and configuration

use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "test-script-engines")]
#[command(about = "OpenSim Next Script Engine Test Suite")]
#[command(version = "1.0.0")]
struct Cli {
    /// ScriptEngines directory path
    #[arg(long, default_value = "bin/ScriptEngines")]
    engines_dir: PathBuf,

    /// Configuration file path
    #[arg(long, default_value = "bin/config-include/ScriptEngines.ini")]
    config_file: PathBuf,

    /// Run performance tests
    #[arg(long)]
    performance: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    success: bool,
    message: String,
    duration: Duration,
    details: Option<Value>,
}

impl TestResult {
    fn success(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            success: true,
            message: message.to_string(),
            duration,
            details: None,
        }
    }

    fn failure(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            success: false,
            message: message.to_string(),
            duration,
            details: None,
        }
    }

    fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

struct ScriptEngineTestSuite {
    engines_dir: PathBuf,
    config_file: PathBuf,
    verbose: bool,
    results: Vec<TestResult>,
}

impl ScriptEngineTestSuite {
    fn new(engines_dir: PathBuf, config_file: PathBuf, verbose: bool) -> Self {
        Self {
            engines_dir,
            config_file,
            verbose,
            results: Vec::new(),
        }
    }

    async fn run_all_tests(&mut self, run_performance: bool) {
        println!("🧪 OpenSim Next Script Engine Test Suite");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing ScriptEngines at: {}", self.engines_dir.display());
        println!("Configuration file: {}", self.config_file.display());
        println!();

        self.test_directory_structure().await;
        self.test_configuration_file().await;
        self.test_engine_directories().await;
        self.test_engine_manager().await;
        self.test_configuration_validation().await;

        if run_performance {
            self.test_performance().await;
        }

        self.print_summary();
    }

    async fn test_directory_structure(&mut self) {
        println!("📁 Testing Directory Structure...");
        let start = Instant::now();

        let required_dirs = [
            &self.engines_dir,
            &self.engines_dir.join("Native"),
            &self.engines_dir.join("YEngine"),
            &self.engines_dir.join("XEngine"),
            &self.engines_dir.join("Common"),
        ];

        let mut missing_dirs = Vec::new();
        let mut existing_dirs = Vec::new();

        for dir in &required_dirs {
            if dir.exists() {
                existing_dirs.push(dir.display().to_string());
                if self.verbose {
                    println!("  ✅ Found: {}", dir.display());
                }
            } else {
                missing_dirs.push(dir.display().to_string());
                if self.verbose {
                    println!("  ❌ Missing: {}", dir.display());
                }
            }
        }

        if missing_dirs.is_empty() {
            self.results.push(
                TestResult::success(
                    "Directory Structure",
                    &format!("All {} required directories exist", required_dirs.len()),
                    start.elapsed(),
                )
                .with_details(json!({
                    "existing_directories": existing_dirs,
                    "total_checked": required_dirs.len()
                })),
            );
            println!("  ✅ Directory structure: PASSED");
        } else {
            self.results.push(
                TestResult::failure(
                    "Directory Structure",
                    &format!("{} directories missing", missing_dirs.len()),
                    start.elapsed(),
                )
                .with_details(json!({
                    "missing_directories": missing_dirs,
                    "existing_directories": existing_dirs
                })),
            );
            println!("  ❌ Directory structure: FAILED");
        }
    }

    async fn test_configuration_file(&mut self) {
        println!("\n⚙️  Testing Configuration File...");
        let start = Instant::now();

        if !self.config_file.exists() {
            self.results.push(TestResult::failure(
                "Configuration File",
                "Configuration file does not exist",
                start.elapsed(),
            ));
            println!("  ❌ Configuration file: NOT FOUND");
            return;
        }

        match fs::read_to_string(&self.config_file) {
            Ok(content) => {
                let sections = self.parse_ini_sections(&content);
                let engines_found = sections
                    .keys()
                    .filter(|k| !k.eq_ignore_ascii_case("ScriptEngines"))
                    .count();

                self.results.push(
                    TestResult::success(
                        "Configuration File",
                        &format!("Valid configuration with {} engine sections", engines_found),
                        start.elapsed(),
                    )
                    .with_details(json!({
                        "file_size": content.len(),
                        "sections": sections.keys().collect::<Vec<_>>(),
                        "engine_count": engines_found
                    })),
                );

                println!("  ✅ Configuration file: VALID");
                println!("      Sections found: {}", sections.keys().count());
                println!("      Engine configurations: {}", engines_found);
            }
            Err(e) => {
                self.results.push(TestResult::failure(
                    "Configuration File",
                    &format!("Failed to read configuration: {}", e),
                    start.elapsed(),
                ));
                println!("  ❌ Configuration file: READ ERROR");
            }
        }
    }

    fn parse_ini_sections(&self, content: &str) -> HashMap<String, HashMap<String, String>> {
        let mut sections = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                sections.insert(current_section.clone(), HashMap::new());
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                if let Some(section) = sections.get_mut(&current_section) {
                    section.insert(
                        key.trim().to_string(),
                        value.trim().trim_matches('"').to_string(),
                    );
                }
            }
        }

        sections
    }

    async fn test_engine_directories(&mut self) {
        println!("\n🔧 Testing Engine Directories...");
        let start = Instant::now();

        let engines = ["Native", "YEngine", "XEngine"];
        let mut engine_details = HashMap::new();
        let mut total_score = 0;
        let max_score = engines.len() * 3; // 3 points per engine (dir + readme + config)

        for engine in &engines {
            let engine_dir = self.engines_dir.join(engine);
            let readme_file = engine_dir.join("README.md");
            let mut engine_info = HashMap::new();
            let mut score = 0;

            // Check directory exists
            if engine_dir.exists() {
                score += 1;
                engine_info.insert("directory_exists", json!(true));
                if self.verbose {
                    println!("  ✅ {} directory exists", engine);
                }
            } else {
                engine_info.insert("directory_exists", json!(false));
                if self.verbose {
                    println!("  ❌ {} directory missing", engine);
                }
            }

            // Check README exists
            if readme_file.exists() {
                score += 1;
                engine_info.insert("readme_exists", json!(true));
                if let Ok(content) = fs::read_to_string(&readme_file) {
                    engine_info.insert("readme_size", json!(content.len()));
                }
                if self.verbose {
                    println!("  ✅ {} README exists", engine);
                }
            } else {
                engine_info.insert("readme_exists", json!(false));
                if self.verbose {
                    println!("  ❌ {} README missing", engine);
                }
            }

            // Check for configuration templates or files
            let config_files = vec![
                "template.ini".to_string(),
                "config.ini".to_string(),
                format!("{}.ini", engine.to_lowercase()),
            ];
            let mut config_found = false;
            for config_file in &config_files {
                if engine_dir.join(config_file).exists() {
                    score += 1;
                    config_found = true;
                    engine_info.insert("config_file", json!(config_file));
                    break;
                }
            }

            if !config_found {
                engine_info.insert("config_file", json!(null));
                if self.verbose {
                    println!("  ⚠️  {} configuration file missing", engine);
                }
            }

            engine_info.insert("score", json!(score));
            engine_details.insert(engine.to_string(), engine_info);
            total_score += score;
        }

        let success_rate = (total_score as f64 / max_score as f64) * 100.0;

        if total_score == max_score {
            self.results.push(
                TestResult::success(
                    "Engine Directories",
                    &format!("All engine directories complete ({}%)", success_rate as u8),
                    start.elapsed(),
                )
                .with_details(json!({
                    "engines": engine_details,
                    "total_score": total_score,
                    "max_score": max_score,
                    "success_rate": success_rate
                })),
            );
            println!("  ✅ Engine directories: COMPLETE");
        } else {
            self.results.push(
                TestResult::failure(
                    "Engine Directories",
                    &format!("Engine directories incomplete ({:.1}%)", success_rate),
                    start.elapsed(),
                )
                .with_details(json!({
                    "engines": engine_details,
                    "total_score": total_score,
                    "max_score": max_score,
                    "success_rate": success_rate
                })),
            );
            println!(
                "  ⚠️  Engine directories: INCOMPLETE ({:.1}%)",
                success_rate
            );
        }

        println!("      Score: {}/{}", total_score, max_score);
    }

    async fn test_engine_manager(&mut self) {
        println!("\n🛠️  Testing Engine Manager...");
        let start = Instant::now();

        // Test if the engine manager binary exists or can be built
        let manager_binary = "target/debug/script_engine_manager";
        let manager_source = "rust/src/bin/script_engine_manager.rs";

        let mut details = HashMap::new();

        // Check if source file exists
        if Path::new(manager_source).exists() {
            details.insert("source_exists", json!(true));
            if self.verbose {
                println!("  ✅ Engine manager source found");
            }
        } else {
            details.insert("source_exists", json!(false));
            if self.verbose {
                println!("  ❌ Engine manager source missing");
            }
        }

        // Check if binary exists
        if Path::new(manager_binary).exists() {
            details.insert("binary_exists", json!(true));
            if self.verbose {
                println!("  ✅ Engine manager binary found");
            }
        } else {
            details.insert("binary_exists", json!(false));
            if self.verbose {
                println!("  ⚠️  Engine manager binary not built (run 'cargo build')");
            }
        }

        // Test basic functionality (if available)
        details.insert("functionality_tested", json!(false));

        let has_source = details
            .get("source_exists")
            .unwrap()
            .as_bool()
            .unwrap_or(false);
        if has_source {
            self.results.push(
                TestResult::success(
                    "Engine Manager",
                    "Engine manager implementation available",
                    start.elapsed(),
                )
                .with_details(json!(details)),
            );
            println!("  ✅ Engine manager: AVAILABLE");
        } else {
            self.results.push(
                TestResult::failure(
                    "Engine Manager",
                    "Engine manager implementation missing",
                    start.elapsed(),
                )
                .with_details(json!(details)),
            );
            println!("  ❌ Engine manager: MISSING");
        }
    }

    async fn test_configuration_validation(&mut self) {
        println!("\n🔍 Testing Configuration Validation...");
        let start = Instant::now();

        if !self.config_file.exists() {
            self.results.push(TestResult::failure(
                "Configuration Validation",
                "No configuration file to validate",
                start.elapsed(),
            ));
            println!("  ❌ Configuration validation: NO CONFIG FILE");
            return;
        }

        let content = match fs::read_to_string(&self.config_file) {
            Ok(content) => content,
            Err(e) => {
                self.results.push(TestResult::failure(
                    "Configuration Validation",
                    &format!("Failed to read configuration: {}", e),
                    start.elapsed(),
                ));
                return;
            }
        };

        let sections = self.parse_ini_sections(&content);
        let mut validation_results = HashMap::new();
        let mut issues = Vec::new();

        // Check for required sections
        if !sections.contains_key("ScriptEngines") {
            issues.push("Missing [ScriptEngines] section".to_string());
        }

        // Validate engine sections
        let engines = ["Native", "YEngine", "XEngine"];
        for engine in &engines {
            if let Some(engine_config) = sections.get(*engine) {
                let mut engine_issues = Vec::new();

                // Check required fields
                if !engine_config.contains_key("Enabled") {
                    engine_issues.push("Missing 'Enabled' setting".to_string());
                }
                if !engine_config.contains_key("Class") {
                    engine_issues.push("Missing 'Class' setting".to_string());
                }
                if !engine_config.contains_key("Assembly") {
                    engine_issues.push("Missing 'Assembly' setting".to_string());
                }

                validation_results.insert(format!("{}_issues", engine), json!(engine_issues));
                issues.extend(
                    engine_issues
                        .into_iter()
                        .map(|issue| format!("{}: {}", engine, issue)),
                );
            } else {
                let issue = format!("Missing [{}] section", engine);
                issues.push(issue);
            }
        }

        validation_results.insert("total_issues".to_string(), json!(issues.len()));
        validation_results.insert("issues".to_string(), json!(issues));

        if issues.is_empty() {
            self.results.push(
                TestResult::success(
                    "Configuration Validation",
                    "Configuration is valid",
                    start.elapsed(),
                )
                .with_details(json!(validation_results)),
            );
            println!("  ✅ Configuration validation: PASSED");
        } else {
            self.results.push(
                TestResult::failure(
                    "Configuration Validation",
                    &format!("{} validation issues found", issues.len()),
                    start.elapsed(),
                )
                .with_details(json!(validation_results)),
            );
            println!("  ❌ Configuration validation: {} ISSUES", issues.len());

            if self.verbose {
                for issue in &issues {
                    println!("      • {}", issue);
                }
            }
        }
    }

    async fn test_performance(&mut self) {
        println!("\n⚡ Testing Performance...");
        let start = Instant::now();

        let mut performance_data = HashMap::new();

        // Test configuration file parsing speed
        if self.config_file.exists() {
            let parse_start = Instant::now();
            let iterations = 100;

            for _ in 0..iterations {
                if let Ok(content) = fs::read_to_string(&self.config_file) {
                    let _ = self.parse_ini_sections(&content);
                }
            }

            let parse_time = parse_start.elapsed();
            let avg_parse_time = parse_time / iterations;

            performance_data.insert("config_parse_iterations", json!(iterations));
            performance_data.insert("total_parse_time_ms", json!(parse_time.as_millis()));
            performance_data.insert("avg_parse_time_us", json!(avg_parse_time.as_micros()));

            println!("  📊 Configuration parsing:");
            println!(
                "      {} iterations in {:.2}ms",
                iterations,
                parse_time.as_millis()
            );
            println!(
                "      Average: {:.1}μs per parse",
                avg_parse_time.as_micros()
            );
        }

        // Test directory scanning speed
        let scan_start = Instant::now();
        let _ = fs::read_dir(&self.engines_dir);
        let scan_time = scan_start.elapsed();

        performance_data.insert("directory_scan_time_us", json!(scan_time.as_micros()));

        self.results.push(
            TestResult::success(
                "Performance Tests",
                "Performance benchmarks completed",
                start.elapsed(),
            )
            .with_details(json!(performance_data)),
        );

        println!("  ✅ Performance tests: COMPLETED");
    }

    fn print_summary(&self) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎉 Script Engine Test Suite Complete!");

        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;

        println!("\n📋 Summary:");
        println!("   Total tests: {}", total_tests);
        println!("   Successful: {} ✅", successful_tests);
        println!(
            "   Failed: {} {}",
            failed_tests,
            if failed_tests > 0 { "❌" } else { "" }
        );

        let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
        println!("   Success rate: {:.1}%", success_rate);

        if failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.results {
                if !result.success {
                    println!("   • {}: {}", result.name, result.message);
                }
            }
        }

        let avg_duration: Duration =
            self.results.iter().map(|r| r.duration).sum::<Duration>() / self.results.len() as u32;

        println!(
            "\n⏱️  Average test duration: {:.3}ms",
            avg_duration.as_millis()
        );

        if self.verbose && !self.results.is_empty() {
            println!("\n📊 Detailed Results:");
            for result in &self.results {
                let status = if result.success { "✅" } else { "❌" };
                println!(
                    "   {} {} ({:.2}ms)",
                    status,
                    result.name,
                    result.duration.as_millis()
                );
                if let Some(details) = &result.details {
                    println!(
                        "      Details: {}",
                        serde_json::to_string_pretty(details).unwrap_or_default()
                    );
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut test_suite = ScriptEngineTestSuite::new(cli.engines_dir, cli.config_file, cli.verbose);

    test_suite.run_all_tests(cli.performance).await;

    // Exit with error code if any tests failed
    let failed_tests = test_suite.results.iter().filter(|r| !r.success).count();
    if failed_tests > 0 {
        std::process::exit(1);
    }

    Ok(())
}
