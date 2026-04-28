//! OpenSim Next Admin Terminal
//!
//! Standalone terminal interface for OpenSim Robust-style administrative commands.
//! Provides an interactive command-line interface that mirrors the classic OpenSim
//! Robust server console experience with modern REST API integration.

use anyhow::Result;
use opensim_next::network::terminal_commands::TerminalCommandProcessor;
use std::env;
use tokio;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Check for command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            "--version" | "-v" => {
                println!("OpenSim Next Admin Terminal v1.0.0");
                println!("Compatible with OpenSim Robust server commands");
                return Ok(());
            }
            "--test-connection" => {
                return test_connection().await;
            }
            "--batch" => {
                if args.len() > 2 {
                    return execute_batch_command(&args[2..]).await;
                } else {
                    eprintln!("Error: --batch requires a command");
                    print_usage();
                    return Ok(());
                }
            }
            _ => {
                // Execute single command
                return execute_batch_command(&args[1..]).await;
            }
        }
    }

    // Start interactive session
    info!("Starting OpenSim Next Admin Terminal");

    let processor = TerminalCommandProcessor::new();

    match processor.start_interactive_session().await {
        Ok(_) => {
            info!("Admin terminal session ended normally");
        }
        Err(e) => {
            error!("Admin terminal session failed: {}", e);
            eprintln!("❌ Terminal session failed: {}", e);
            eprintln!("💡 Check your API configuration:");
            eprintln!("   - OPENSIM_API_KEY (default: default-key-change-me)");
            eprintln!("   - OPENSIM_ADMIN_API_URL (default: http://localhost:9200)");
            eprintln!("   - Ensure OpenSim Next server is running");
        }
    }

    Ok(())
}

/// Print usage information
fn print_usage() {
    println!(
        r#"
🎯 OpenSim Next Admin Terminal - Usage
=====================================

Interactive Mode (default):
  admin_terminal                    Start interactive terminal session

Single Command Mode:
  admin_terminal <command>          Execute single command and exit
  admin_terminal --batch <command>  Execute command in batch mode

Examples:
  admin_terminal create user John Doe password123 john@example.com
  admin_terminal show users 10
  admin_terminal database stats
  admin_terminal --batch "reset user password John Doe newpass123"

Options:
  -h, --help                       Show this help message
  -v, --version                    Show version information
  --test-connection                Test connection to admin API

Configuration:
  Set these environment variables to configure the terminal:
  
  OPENSIM_API_KEY                  API key for authentication
                                   Default: default-key-change-me
  
  OPENSIM_ADMIN_API_URL            Admin API server URL
                                   Default: http://localhost:9200

Available Commands (Interactive Mode):
  Type 'help' in interactive mode for detailed command list
  All commands mirror classic OpenSim Robust server syntax

📝 Notes:
  - Interactive mode provides command history and auto-completion
  - Batch mode is suitable for scripting and automation
  - All operations require valid API key authentication
  - Commands are logged for audit purposes
"#
    );
}

/// Test connection to admin API
async fn test_connection() -> Result<()> {
    println!("🔍 Testing connection to OpenSim Next Admin API...");

    let processor = TerminalCommandProcessor::new();

    // Try to execute a simple health check command
    match processor
        .execute_command(opensim_next::network::terminal_commands::TerminalCommand::Help)
        .await
    {
        Ok(_) => {
            println!("✅ Connection test successful!");
            println!("🌐 Admin API is accessible and responding");

            // Try to get actual API health
            let api_key =
                env::var("OPENSIM_API_KEY").unwrap_or_else(|_| "default-key-change-me".to_string());
            let api_url = env::var("OPENSIM_ADMIN_API_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string());

            println!(
                "🔑 Using API Key: {}",
                if api_key == "default-key-change-me" {
                    "default-key-change-me (⚠️  change for production)"
                } else {
                    "configured"
                }
            );
            println!("🌐 API URL: {}", api_url);

            // Test actual admin API health endpoint
            let client = reqwest::Client::new();
            match client
                .get(&format!("{}/admin/health", api_url))
                .header("X-API-Key", &api_key)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("✅ Admin API health check passed");
                        if let Ok(text) = response.text().await {
                            println!("📊 Response: {}", text);
                        }
                    } else {
                        println!("⚠️  Admin API returned status: {}", response.status());
                    }
                }
                Err(e) => {
                    println!("❌ Failed to connect to admin API: {}", e);
                    println!("💡 Make sure OpenSim Next server is running");
                }
            }
        }
        Err(e) => {
            println!("❌ Connection test failed: {}", e);
            println!("💡 Check your configuration and ensure the server is running");
        }
    }

    Ok(())
}

/// Execute batch command (single command execution)
async fn execute_batch_command(args: &[String]) -> Result<()> {
    let command_line = args.join(" ");
    println!("🚀 Executing batch command: {}", command_line);

    let processor = TerminalCommandProcessor::new();

    match processor.parse_command(&command_line) {
        Ok(command) => match processor.execute_command(command).await {
            Ok(result) => {
                if result.success {
                    println!("✅ Command executed successfully");
                    println!("📝 {}", result.message);
                    if let Some(data) = result.data {
                        println!("📊 Result:");
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    }
                    std::process::exit(0);
                } else {
                    println!("❌ Command failed: {}", result.message);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                println!("💥 Execution error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("💥 Invalid command: {}", e);
            println!("💡 Use 'admin_terminal --help' for usage information");
            std::process::exit(1);
        }
    }
}
