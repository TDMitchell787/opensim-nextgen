//! Terminal command interface for OpenSim Robust-style admin commands
//!
//! Provides a command-line interface that mirrors the classic OpenSim Robust console
//! commands, implemented as a modern REST API client with comprehensive validation.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use tracing::{error, info, warn};

/// Terminal command processor for admin operations
pub struct TerminalCommandProcessor {
    client: Client,
    api_base_url: String,
    api_key: String,
}

/// Command execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Terminal command types matching OpenSim Robust commands
#[derive(Debug, Clone)]
pub enum TerminalCommand {
    CreateUser {
        firstname: String,
        lastname: String,
        password: String,
        email: String,
        user_level: Option<i32>,
    },
    ResetUserPassword {
        firstname: String,
        lastname: String,
        new_password: String,
    },
    ResetUserEmail {
        firstname: String,
        lastname: String,
        new_email: String,
    },
    SetUserLevel {
        firstname: String,
        lastname: String,
        user_level: i32,
    },
    ShowAccount {
        firstname: String,
        lastname: String,
    },
    ShowUsers {
        limit: Option<i32>,
    },
    DeleteUser {
        firstname: String,
        lastname: String,
    },
    DatabaseStats,
    DatabaseBackup {
        backup_name: String,
        include_users: bool,
        include_regions: bool,
        include_assets: bool,
        include_inventory: bool,
    },
    DatabaseRestore {
        backup_file: String,
        overwrite_existing: bool,
    },
    DatabaseMaintenance {
        vacuum: bool,
        reindex: bool,
        analyze: bool,
        cleanup: bool,
    },
    DatabaseMigration {
        target_version: String,
        dry_run: bool,
        backup_before: bool,
    },
    DatabaseHealth,
    DatabaseListBackups {
        directory: Option<String>,
    },
    LoadIar {
        firstname: String,
        lastname: String,
        file_path: String,
        merge: bool,
    },
    SaveIar {
        firstname: String,
        lastname: String,
        file_path: String,
        include_assets: bool,
    },
    LoadOar {
        region_name: String,
        file_path: String,
        merge: bool,
        force_terrain: bool,
        force_parcels: bool,
    },
    SaveOar {
        region_name: String,
        file_path: String,
        include_assets: bool,
        include_terrain: bool,
        include_objects: bool,
        include_parcels: bool,
    },
    ShowArchiveJobs {
        limit: Option<i32>,
    },
    ShowArchiveJob {
        job_id: String,
    },
    CancelArchiveJob {
        job_id: String,
    },
    Help,
    Exit,
}

impl TerminalCommandProcessor {
    /// Create new terminal command processor
    pub fn new() -> Self {
        let api_base_url = env::var("OPENSIM_ADMIN_API_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string());
        let api_key =
            env::var("OPENSIM_API_KEY").unwrap_or_else(|_| "default-key-change-me".to_string());

        Self {
            client: Client::new(),
            api_base_url,
            api_key,
        }
    }

    /// Start interactive terminal session
    pub async fn start_interactive_session(&self) -> Result<()> {
        println!("🎯 OpenSim Next - Robust-Style Admin Terminal");
        println!("===============================================");
        println!("Type 'help' for available commands or 'exit' to quit.");
        println!("All commands mirror classic OpenSim Robust server syntax.\n");

        loop {
            print!("OpenSimNext> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match self.parse_command(input) {
                Ok(TerminalCommand::Exit) => {
                    println!("👋 Goodbye!");
                    break;
                }
                Ok(command) => match self.execute_command(command).await {
                    Ok(result) => {
                        if result.success {
                            println!("✅ {}", result.message);
                            if let Some(data) = result.data {
                                println!("{}", serde_json::to_string_pretty(&data)?);
                            }
                        } else {
                            println!("❌ {}", result.message);
                        }
                    }
                    Err(e) => {
                        println!("🔥 Command failed: {}", e);
                    }
                },
                Err(e) => {
                    println!("💥 Invalid command: {}", e);
                    println!("Type 'help' for available commands.");
                }
            }
            println!();
        }

        Ok(())
    }

    /// Parse user input into terminal command
    pub fn parse_command(&self, input: &str) -> Result<TerminalCommand> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        match parts[0].to_lowercase().as_str() {
            "help" => Ok(TerminalCommand::Help),
            "exit" | "quit" => Ok(TerminalCommand::Exit),
            "create" => self.parse_create_command(&parts),
            "reset" => self.parse_reset_command(&parts),
            "set" => self.parse_set_command(&parts),
            "show" => self.parse_show_command(&parts),
            "delete" => self.parse_delete_command(&parts),
            "database" => self.parse_database_command(&parts),
            "load" => self.parse_load_command(&parts),
            "save" => self.parse_save_command(&parts),
            "archive" => self.parse_archive_command(&parts),
            "cancel" => self.parse_cancel_command(&parts),
            _ => Err(anyhow::anyhow!("Unknown command: {}", parts[0])),
        }
    }

    /// Parse 'create user' command
    fn parse_create_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 || parts[1] != "user" {
            return Err(anyhow::anyhow!(
                "Expected 'create user firstname lastname password email [user_level]'"
            ));
        }

        if parts.len() < 6 {
            return Err(anyhow::anyhow!(
                "create user requires: firstname lastname password email [user_level]"
            ));
        }

        let user_level = if parts.len() > 6 {
            Some(parts[6].parse::<i32>()?)
        } else {
            None
        };

        Ok(TerminalCommand::CreateUser {
            firstname: parts[2].to_string(),
            lastname: parts[3].to_string(),
            password: parts[4].to_string(),
            email: parts[5].to_string(),
            user_level,
        })
    }

    /// Parse 'reset user' commands
    fn parse_reset_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 || parts[1] != "user" {
            return Err(anyhow::anyhow!(
                "Expected 'reset user password|email firstname lastname value'"
            ));
        }

        match parts[2] {
            "password" => {
                if parts.len() < 6 {
                    return Err(anyhow::anyhow!(
                        "reset user password requires: firstname lastname new_password"
                    ));
                }
                Ok(TerminalCommand::ResetUserPassword {
                    firstname: parts[3].to_string(),
                    lastname: parts[4].to_string(),
                    new_password: parts[5].to_string(),
                })
            }
            "email" => {
                if parts.len() < 6 {
                    return Err(anyhow::anyhow!(
                        "reset user email requires: firstname lastname new_email"
                    ));
                }
                Ok(TerminalCommand::ResetUserEmail {
                    firstname: parts[3].to_string(),
                    lastname: parts[4].to_string(),
                    new_email: parts[5].to_string(),
                })
            }
            _ => Err(anyhow::anyhow!("reset user accepts: password or email")),
        }
    }

    /// Parse 'set user' commands
    fn parse_set_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 || parts[1] != "user" {
            return Err(anyhow::anyhow!(
                "Expected 'set user level firstname lastname level'"
            ));
        }

        if parts[2] != "level" {
            return Err(anyhow::anyhow!("set user currently supports: level"));
        }

        if parts.len() < 6 {
            return Err(anyhow::anyhow!(
                "set user level requires: firstname lastname level"
            ));
        }

        Ok(TerminalCommand::SetUserLevel {
            firstname: parts[3].to_string(),
            lastname: parts[4].to_string(),
            user_level: parts[5].parse::<i32>()?,
        })
    }

    /// Parse 'show' commands
    fn parse_show_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Expected 'show account|users'"));
        }

        match parts[1] {
            "account" => {
                if parts.len() < 4 {
                    return Err(anyhow::anyhow!("show account requires: firstname lastname"));
                }
                Ok(TerminalCommand::ShowAccount {
                    firstname: parts[2].to_string(),
                    lastname: parts[3].to_string(),
                })
            }
            "users" => {
                let limit = if parts.len() > 2 {
                    Some(parts[2].parse::<i32>()?)
                } else {
                    None
                };
                Ok(TerminalCommand::ShowUsers { limit })
            }
            _ => Err(anyhow::anyhow!("show accepts: account or users")),
        }
    }

    /// Parse 'delete user' command
    fn parse_delete_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 || parts[1] != "user" {
            return Err(anyhow::anyhow!("Expected 'delete user firstname lastname'"));
        }

        if parts.len() < 4 {
            return Err(anyhow::anyhow!("delete user requires: firstname lastname"));
        }

        Ok(TerminalCommand::DeleteUser {
            firstname: parts[2].to_string(),
            lastname: parts[3].to_string(),
        })
    }

    /// Parse 'database' commands
    fn parse_database_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Expected: database <command>"));
        }

        match parts[1] {
            "stats" => Ok(TerminalCommand::DatabaseStats),
            "health" => Ok(TerminalCommand::DatabaseHealth),
            "backup" => self.parse_database_backup_command(&parts),
            "restore" => self.parse_database_restore_command(&parts),
            "maintenance" => self.parse_database_maintenance_command(&parts),
            "migration" => self.parse_database_migration_command(&parts),
            "list-backups" => self.parse_database_list_backups_command(&parts),
            _ => Err(anyhow::anyhow!("database accepts: stats, health, backup, restore, maintenance, migration, list-backups")),
        }
    }

    /// Parse 'database backup' command
    fn parse_database_backup_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 {
            return Err(anyhow::anyhow!(
                "Expected: database backup <backup_name> [options]"
            ));
        }

        let backup_name = parts[2].to_string();
        let mut include_users = true;
        let mut include_regions = true;
        let mut include_assets = false;
        let mut include_inventory = false;

        // Parse optional flags
        for part in parts.iter().skip(3) {
            match *part {
                "--no-users" => include_users = false,
                "--no-regions" => include_regions = false,
                "--include-assets" => include_assets = true,
                "--include-inventory" => include_inventory = true,
                _ => return Err(anyhow::anyhow!("Unknown backup option: {}", part)),
            }
        }

        Ok(TerminalCommand::DatabaseBackup {
            backup_name,
            include_users,
            include_regions,
            include_assets,
            include_inventory,
        })
    }

    /// Parse 'database restore' command
    fn parse_database_restore_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 {
            return Err(anyhow::anyhow!(
                "Expected: database restore <backup_file> [--overwrite]"
            ));
        }

        let backup_file = parts[2].to_string();
        let overwrite_existing = parts.len() > 3 && parts[3] == "--overwrite";

        Ok(TerminalCommand::DatabaseRestore {
            backup_file,
            overwrite_existing,
        })
    }

    /// Parse 'database maintenance' command
    fn parse_database_maintenance_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        let mut vacuum = false;
        let mut reindex = false;
        let mut analyze = false;
        let mut cleanup = false;

        if parts.len() == 2 {
            // Default: run all maintenance operations
            vacuum = true;
            reindex = true;
            analyze = true;
            cleanup = true;
        } else {
            // Parse specific operations
            for part in parts.iter().skip(2) {
                match *part {
                    "--vacuum" => vacuum = true,
                    "--reindex" => reindex = true,
                    "--analyze" => analyze = true,
                    "--cleanup" => cleanup = true,
                    _ => return Err(anyhow::anyhow!("Unknown maintenance option: {}", part)),
                }
            }
        }

        Ok(TerminalCommand::DatabaseMaintenance {
            vacuum,
            reindex,
            analyze,
            cleanup,
        })
    }

    /// Parse 'database migration' command
    fn parse_database_migration_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 {
            return Err(anyhow::anyhow!(
                "Expected: database migration <version> [--dry-run] [--backup]"
            ));
        }

        let target_version = parts[2].to_string();
        let mut dry_run = false;
        let mut backup_before = false;

        for part in parts.iter().skip(3) {
            match *part {
                "--dry-run" => dry_run = true,
                "--backup" => backup_before = true,
                _ => return Err(anyhow::anyhow!("Unknown migration option: {}", part)),
            }
        }

        Ok(TerminalCommand::DatabaseMigration {
            target_version,
            dry_run,
            backup_before,
        })
    }

    /// Parse 'database list-backups' command
    fn parse_database_list_backups_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        let directory = if parts.len() > 2 {
            Some(parts[2].to_string())
        } else {
            None
        };

        Ok(TerminalCommand::DatabaseListBackups { directory })
    }

    /// Parse 'load iar|oar' command
    fn parse_load_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Expected 'load iar|oar'"));
        }

        match parts[1].to_lowercase().as_str() {
            "iar" => {
                if parts.len() < 5 {
                    return Err(anyhow::anyhow!(
                        "load iar requires: firstname lastname filepath [--merge]"
                    ));
                }
                let merge = parts.iter().any(|p| *p == "--merge");
                Ok(TerminalCommand::LoadIar {
                    firstname: parts[2].to_string(),
                    lastname: parts[3].to_string(),
                    file_path: parts[4].to_string(),
                    merge,
                })
            }
            "oar" => {
                if parts.len() < 4 {
                    return Err(anyhow::anyhow!("load oar requires: regionname filepath [--merge] [--force-terrain] [--force-parcels]"));
                }
                let merge = parts.iter().any(|p| *p == "--merge");
                let force_terrain = parts.iter().any(|p| *p == "--force-terrain");
                let force_parcels = parts.iter().any(|p| *p == "--force-parcels");
                Ok(TerminalCommand::LoadOar {
                    region_name: parts[2].to_string(),
                    file_path: parts[3].to_string(),
                    merge,
                    force_terrain,
                    force_parcels,
                })
            }
            _ => Err(anyhow::anyhow!("load accepts: iar or oar")),
        }
    }

    /// Parse 'save iar|oar' command
    fn parse_save_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Expected 'save iar|oar'"));
        }

        match parts[1].to_lowercase().as_str() {
            "iar" => {
                if parts.len() < 5 {
                    return Err(anyhow::anyhow!(
                        "save iar requires: firstname lastname filepath [--no-assets]"
                    ));
                }
                let include_assets = !parts.iter().any(|p| *p == "--no-assets");
                Ok(TerminalCommand::SaveIar {
                    firstname: parts[2].to_string(),
                    lastname: parts[3].to_string(),
                    file_path: parts[4].to_string(),
                    include_assets,
                })
            }
            "oar" => {
                if parts.len() < 4 {
                    return Err(anyhow::anyhow!("save oar requires: regionname filepath [--no-assets] [--no-terrain] [--no-objects] [--no-parcels]"));
                }
                let include_assets = !parts.iter().any(|p| *p == "--no-assets");
                let include_terrain = !parts.iter().any(|p| *p == "--no-terrain");
                let include_objects = !parts.iter().any(|p| *p == "--no-objects");
                let include_parcels = !parts.iter().any(|p| *p == "--no-parcels");
                Ok(TerminalCommand::SaveOar {
                    region_name: parts[2].to_string(),
                    file_path: parts[3].to_string(),
                    include_assets,
                    include_terrain,
                    include_objects,
                    include_parcels,
                })
            }
            _ => Err(anyhow::anyhow!("save accepts: iar or oar")),
        }
    }

    /// Parse 'archive jobs|job' command
    fn parse_archive_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Expected 'archive jobs|job'"));
        }

        match parts[1].to_lowercase().as_str() {
            "jobs" => {
                let limit = if parts.len() > 2 {
                    Some(parts[2].parse::<i32>()?)
                } else {
                    None
                };
                Ok(TerminalCommand::ShowArchiveJobs { limit })
            }
            "job" => {
                if parts.len() < 3 {
                    return Err(anyhow::anyhow!("archive job requires: job_id"));
                }
                Ok(TerminalCommand::ShowArchiveJob {
                    job_id: parts[2].to_string(),
                })
            }
            _ => Err(anyhow::anyhow!("archive accepts: jobs or job")),
        }
    }

    /// Parse 'cancel archive' command
    fn parse_cancel_command(&self, parts: &[&str]) -> Result<TerminalCommand> {
        if parts.len() < 3 || parts[1] != "archive" {
            return Err(anyhow::anyhow!("Expected 'cancel archive job_id'"));
        }

        Ok(TerminalCommand::CancelArchiveJob {
            job_id: parts[2].to_string(),
        })
    }

    /// Execute terminal command via REST API
    pub async fn execute_command(&self, command: TerminalCommand) -> Result<CommandResult> {
        match command {
            TerminalCommand::Help => Ok(CommandResult {
                success: true,
                message: self.get_help_text(),
                data: None,
            }),
            TerminalCommand::CreateUser {
                firstname,
                lastname,
                password,
                email,
                user_level,
            } => {
                self.create_user_api(firstname, lastname, password, email, user_level)
                    .await
            }
            TerminalCommand::ResetUserPassword {
                firstname,
                lastname,
                new_password,
            } => {
                self.reset_password_api(firstname, lastname, new_password)
                    .await
            }
            TerminalCommand::ResetUserEmail {
                firstname,
                lastname,
                new_email,
            } => self.reset_email_api(firstname, lastname, new_email).await,
            TerminalCommand::SetUserLevel {
                firstname,
                lastname,
                user_level,
            } => {
                self.set_user_level_api(firstname, lastname, user_level)
                    .await
            }
            TerminalCommand::ShowAccount {
                firstname,
                lastname,
            } => self.show_account_api(firstname, lastname).await,
            TerminalCommand::ShowUsers { limit } => self.show_users_api(limit).await,
            TerminalCommand::DeleteUser {
                firstname,
                lastname,
            } => self.delete_user_api(firstname, lastname).await,
            TerminalCommand::DatabaseStats => self.database_stats_api().await,
            TerminalCommand::DatabaseHealth => self.database_health_api().await,
            TerminalCommand::DatabaseBackup {
                backup_name,
                include_users,
                include_regions,
                include_assets,
                include_inventory,
            } => {
                self.database_backup_api(
                    backup_name,
                    include_users,
                    include_regions,
                    include_assets,
                    include_inventory,
                )
                .await
            }
            TerminalCommand::DatabaseRestore {
                backup_file,
                overwrite_existing,
            } => {
                self.database_restore_api(backup_file, overwrite_existing)
                    .await
            }
            TerminalCommand::DatabaseMaintenance {
                vacuum,
                reindex,
                analyze,
                cleanup,
            } => {
                self.database_maintenance_api(vacuum, reindex, analyze, cleanup)
                    .await
            }
            TerminalCommand::DatabaseMigration {
                target_version,
                dry_run,
                backup_before,
            } => {
                self.database_migration_api(target_version, dry_run, backup_before)
                    .await
            }
            TerminalCommand::DatabaseListBackups { directory } => {
                self.database_list_backups_api(directory).await
            }
            TerminalCommand::LoadIar {
                firstname,
                lastname,
                file_path,
                merge,
            } => {
                self.load_iar_api(firstname, lastname, file_path, merge)
                    .await
            }
            TerminalCommand::SaveIar {
                firstname,
                lastname,
                file_path,
                include_assets,
            } => {
                self.save_iar_api(firstname, lastname, file_path, include_assets)
                    .await
            }
            TerminalCommand::LoadOar {
                region_name,
                file_path,
                merge,
                force_terrain,
                force_parcels,
            } => {
                self.load_oar_api(region_name, file_path, merge, force_terrain, force_parcels)
                    .await
            }
            TerminalCommand::SaveOar {
                region_name,
                file_path,
                include_assets,
                include_terrain,
                include_objects,
                include_parcels,
            } => {
                self.save_oar_api(
                    region_name,
                    file_path,
                    include_assets,
                    include_terrain,
                    include_objects,
                    include_parcels,
                )
                .await
            }
            TerminalCommand::ShowArchiveJobs { limit } => self.show_archive_jobs_api(limit).await,
            TerminalCommand::ShowArchiveJob { job_id } => self.show_archive_job_api(job_id).await,
            TerminalCommand::CancelArchiveJob { job_id } => {
                self.cancel_archive_job_api(job_id).await
            }
            TerminalCommand::Exit => unreachable!(),
        }
    }

    /// Create user via API
    async fn create_user_api(
        &self,
        firstname: String,
        lastname: String,
        password: String,
        email: String,
        user_level: Option<i32>,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "password": password,
            "email": email,
            "user_level": user_level
        });

        info!("Creating user: {} {}", firstname, lastname);

        let response = self
            .client
            .post(&format!("{}/admin/users", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Reset user password via API
    async fn reset_password_api(
        &self,
        firstname: String,
        lastname: String,
        new_password: String,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "new_password": new_password
        });

        warn!("Resetting password for user: {} {}", firstname, lastname);

        let response = self
            .client
            .put(&format!("{}/admin/users/password", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Reset user email via API
    async fn reset_email_api(
        &self,
        firstname: String,
        lastname: String,
        new_email: String,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "new_email": new_email
        });

        info!("Resetting email for user: {} {}", firstname, lastname);

        let response = self
            .client
            .put(&format!("{}/admin/users/email", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Set user level via API
    async fn set_user_level_api(
        &self,
        firstname: String,
        lastname: String,
        user_level: i32,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "user_level": user_level
        });

        warn!(
            "Setting user level for {} {} to {}",
            firstname, lastname, user_level
        );

        let response = self
            .client
            .put(&format!("{}/admin/users/level", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Show user account via API
    async fn show_account_api(&self, firstname: String, lastname: String) -> Result<CommandResult> {
        let url = format!(
            "{}/admin/users/account?firstname={}&lastname={}",
            self.api_base_url,
            urlencoding::encode(&firstname),
            urlencoding::encode(&lastname)
        );

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Show users via API
    async fn show_users_api(&self, limit: Option<i32>) -> Result<CommandResult> {
        let mut url = format!("{}/admin/users", self.api_base_url);
        if let Some(limit) = limit {
            url = format!("{}?limit={}", url, limit);
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Delete user via API
    async fn delete_user_api(&self, firstname: String, lastname: String) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "firstname": firstname,
            "lastname": lastname
        });

        error!("DESTRUCTIVE: Deleting user {} {}", firstname, lastname);

        let response = self
            .client
            .delete(&format!("{}/admin/users/delete", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Get database statistics via API
    async fn database_stats_api(&self) -> Result<CommandResult> {
        let response = self
            .client
            .get(&format!("{}/admin/database/stats", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Database health check via API
    async fn database_health_api(&self) -> Result<CommandResult> {
        let response = self
            .client
            .get(&format!("{}/admin/database/health", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Create database backup via API
    async fn database_backup_api(
        &self,
        backup_name: String,
        include_users: bool,
        include_regions: bool,
        include_assets: bool,
        include_inventory: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "backup_name": backup_name,
            "include_user_data": include_users,
            "include_region_data": include_regions,
            "include_asset_data": include_assets,
            "include_inventory_data": include_inventory,
            "compression": true,
            "backup_path": null
        });

        info!("Creating database backup: {}", backup_name);

        let response = self
            .client
            .post(&format!("{}/admin/database/backup", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Restore database from backup via API
    async fn database_restore_api(
        &self,
        backup_file: String,
        overwrite_existing: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "backup_file": backup_file,
            "restore_users": true,
            "restore_regions": true,
            "restore_assets": true,
            "restore_inventory": true,
            "overwrite_existing": overwrite_existing
        });

        info!("Restoring database from backup: {}", backup_file);

        let response = self
            .client
            .post(&format!("{}/admin/database/restore", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Perform database maintenance via API
    async fn database_maintenance_api(
        &self,
        vacuum: bool,
        reindex: bool,
        analyze: bool,
        cleanup: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "vacuum_tables": vacuum,
            "reindex_tables": reindex,
            "analyze_tables": analyze,
            "cleanup_orphaned": cleanup,
            "compress_tables": false
        });

        info!("Performing database maintenance");

        let response = self
            .client
            .post(&format!("{}/admin/database/maintenance", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Run database migration via API
    async fn database_migration_api(
        &self,
        target_version: String,
        dry_run: bool,
        backup_before: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "target_version": target_version,
            "dry_run": dry_run,
            "backup_before": backup_before
        });

        info!("Running database migration to version: {}", target_version);

        let response = self
            .client
            .post(&format!("{}/admin/database/migration", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// List available backups via API
    async fn database_list_backups_api(&self, directory: Option<String>) -> Result<CommandResult> {
        let mut url = format!("{}/admin/database/backups", self.api_base_url);
        if let Some(dir) = directory {
            url.push_str(&format!("?directory={}", urlencoding::encode(&dir)));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Load IAR via API
    async fn load_iar_api(
        &self,
        firstname: String,
        lastname: String,
        file_path: String,
        merge: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "file_path": file_path,
            "user_firstname": firstname,
            "user_lastname": lastname,
            "merge": merge
        });

        info!("Loading IAR for user: {} {}", firstname, lastname);

        let response = self
            .client
            .post(&format!("{}/admin/archives/iar/load", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result
                .get("job_id")
                .map(|id| serde_json::json!({"job_id": id})),
        })
    }

    /// Save IAR via API
    async fn save_iar_api(
        &self,
        firstname: String,
        lastname: String,
        file_path: String,
        include_assets: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "output_path": file_path,
            "user_firstname": firstname,
            "user_lastname": lastname,
            "include_assets": include_assets
        });

        info!("Saving IAR for user: {} {}", firstname, lastname);

        let response = self
            .client
            .post(&format!("{}/admin/archives/iar/save", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result
                .get("job_id")
                .map(|id| serde_json::json!({"job_id": id})),
        })
    }

    /// Load OAR via API
    async fn load_oar_api(
        &self,
        region_name: String,
        file_path: String,
        merge: bool,
        force_terrain: bool,
        force_parcels: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "file_path": file_path,
            "region_name": region_name,
            "merge": merge,
            "force_terrain": force_terrain,
            "force_parcels": force_parcels
        });

        info!("Loading OAR for region: {}", region_name);

        let response = self
            .client
            .post(&format!("{}/admin/archives/oar/load", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result
                .get("job_id")
                .map(|id| serde_json::json!({"job_id": id})),
        })
    }

    /// Save OAR via API
    async fn save_oar_api(
        &self,
        region_name: String,
        file_path: String,
        include_assets: bool,
        include_terrain: bool,
        include_objects: bool,
        include_parcels: bool,
    ) -> Result<CommandResult> {
        let payload = serde_json::json!({
            "output_path": file_path,
            "region_name": region_name,
            "include_assets": include_assets,
            "include_terrain": include_terrain,
            "include_objects": include_objects,
            "include_parcels": include_parcels
        });

        info!("Saving OAR for region: {}", region_name);

        let response = self
            .client
            .post(&format!("{}/admin/archives/oar/save", self.api_base_url))
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result
                .get("job_id")
                .map(|id| serde_json::json!({"job_id": id})),
        })
    }

    /// Show archive jobs via API
    async fn show_archive_jobs_api(&self, limit: Option<i32>) -> Result<CommandResult> {
        let mut url = format!("{}/admin/archives/jobs", self.api_base_url);
        if let Some(l) = limit {
            url.push_str(&format!("?limit={}", l));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Show single archive job via API
    async fn show_archive_job_api(&self, job_id: String) -> Result<CommandResult> {
        let response = self
            .client
            .get(&format!(
                "{}/admin/archives/jobs/{}",
                self.api_base_url, job_id
            ))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: result.get("data").cloned(),
        })
    }

    /// Cancel archive job via API
    async fn cancel_archive_job_api(&self, job_id: String) -> Result<CommandResult> {
        let response = self
            .client
            .post(&format!(
                "{}/admin/archives/jobs/{}/cancel",
                self.api_base_url, job_id
            ))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        Ok(CommandResult {
            success: result["success"].as_bool().unwrap_or(false),
            message: result["message"]
                .as_str()
                .unwrap_or("Unknown response")
                .to_string(),
            data: None,
        })
    }

    /// Get help text for available commands
    fn get_help_text(&self) -> String {
        r#"
🎯 OpenSim Next - Available Admin Commands (Robust Compatible)
============================================================

User Management:
  create user <firstname> <lastname> <password> <email> [user_level]
    - Create a new user account
    - user_level: 0=User, 100=Admin, 200=God (optional, defaults to 0)
    - Example: create user John Doe password123 john@example.com 0

  reset user password <firstname> <lastname> <new_password>
    - Reset a user's password
    - Example: reset user password John Doe newpassword123

  reset user email <firstname> <lastname> <new_email>
    - Change a user's email address
    - Example: reset user email John Doe newemail@example.com

  set user level <firstname> <lastname> <level>
    - Set user permission level (0-255)
    - Example: set user level John Doe 200

  show account <firstname> <lastname>
    - Display detailed user account information
    - Example: show account John Doe

  show users [limit]
    - List all users (optional limit, max 1000)
    - Example: show users 50

  delete user <firstname> <lastname>
    - ⚠️  DESTRUCTIVE: Permanently delete user account
    - Example: delete user John Doe

Database:
  database stats
    - Display database statistics and health
  
  database health
    - Check database connectivity and health status
  
  database backup <backup_name> [--no-users] [--no-regions] [--include-assets] [--include-inventory]
    - Create database backup with specified name
    - Options:
      --no-users        : Exclude user accounts from backup
      --no-regions      : Exclude regions from backup
      --include-assets  : Include asset data in backup
      --include-inventory : Include inventory data in backup
    - Example: database backup daily_backup --include-assets
  
  database restore <backup_file> [--overwrite]
    - Restore database from backup file
    - --overwrite     : Allow overwriting existing data
    - Example: database restore ./backups/daily_backup.sql --overwrite
  
  database maintenance [--vacuum] [--reindex] [--analyze] [--cleanup]
    - Perform database maintenance operations
    - No options = run all maintenance operations
    - --vacuum        : Vacuum database tables
    - --reindex       : Rebuild table indexes
    - --analyze       : Update table statistics
    - --cleanup       : Clean orphaned records
    - Example: database maintenance --vacuum --analyze
  
  database migration <version> [--dry-run] [--backup]
    - Run database schema migration to target version
    - --dry-run       : Test migration without applying changes
    - --backup        : Create backup before migration
    - Example: database migration 1.2.0 --backup --dry-run
  
  database list-backups [directory]
    - List available database backup files
    - Optional directory path (defaults to ./backups)
    - Example: database list-backups /opt/opensim/backups

Archives (IAR/OAR):
  load iar <firstname> <lastname> <filepath> [--merge]
    - Load inventory archive (IAR) for user
    - --merge         : Merge with existing inventory (default: replace)
    - Example: load iar John Doe /backups/john_inventory.iar

  save iar <firstname> <lastname> <filepath> [--no-assets]
    - Save user inventory to IAR file
    - --no-assets     : Exclude asset data from archive
    - Example: save iar John Doe /backups/john_inventory.iar

  load oar <regionname> <filepath> [--merge] [--force-terrain] [--force-parcels]
    - Load region archive (OAR) into region
    - --merge         : Merge with existing objects (default: replace)
    - --force-terrain : Overwrite existing terrain
    - --force-parcels : Overwrite existing parcels
    - Example: load oar "My Region" /backups/my_region.oar --merge

  save oar <regionname> <filepath> [--no-assets] [--no-terrain] [--no-objects] [--no-parcels]
    - Save region to OAR file
    - --no-assets     : Exclude asset data
    - --no-terrain    : Exclude terrain data
    - --no-objects    : Exclude objects/prims
    - --no-parcels    : Exclude parcel data
    - Example: save oar "My Region" /backups/my_region.oar

  archive jobs [limit]
    - List active archive jobs
    - Example: archive jobs 10

  archive job <job_id>
    - Show status of specific archive job
    - Example: archive job 550e8400-e29b-41d4-a716-446655440000

  cancel archive <job_id>
    - Cancel a running archive job
    - Example: cancel archive 550e8400-e29b-41d4-a716-446655440000

General:
  help          - Show this help message
  exit          - Exit the terminal

🔑 Authentication:
  Set OPENSIM_API_KEY environment variable for authentication
  Default: OPENSIM_API_KEY=default-key-change-me

🌐 API URL:
  Set OPENSIM_ADMIN_API_URL to change server endpoint
  Default: http://localhost:9200

📝 Notes:
  - All commands mirror classic OpenSim Robust server syntax
  - Commands are logged for audit purposes
  - API key authentication required for all operations
  - Invalid operations return detailed error messages
        "#
        .to_string()
    }
}

/// Utility function to validate user input
pub fn validate_user_input(firstname: &str, lastname: &str, email: Option<&str>) -> Result<()> {
    if firstname.trim().is_empty() {
        return Err(anyhow::anyhow!("First name cannot be empty"));
    }

    if lastname.trim().is_empty() {
        return Err(anyhow::anyhow!("Last name cannot be empty"));
    }

    if let Some(email) = email {
        if email.trim().is_empty() || !email.contains('@') {
            return Err(anyhow::anyhow!("Valid email address required"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_user_command() {
        let processor = TerminalCommandProcessor::new();

        // Valid create user command
        let result = processor.parse_command("create user John Doe password123 john@example.com 0");
        assert!(result.is_ok());

        if let Ok(TerminalCommand::CreateUser {
            firstname,
            lastname,
            password,
            email,
            user_level,
        }) = result
        {
            assert_eq!(firstname, "John");
            assert_eq!(lastname, "Doe");
            assert_eq!(password, "password123");
            assert_eq!(email, "john@example.com");
            assert_eq!(user_level, Some(0));
        }

        // Invalid create user command
        let result = processor.parse_command("create user John");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_reset_commands() {
        let processor = TerminalCommandProcessor::new();

        // Valid reset password command
        let result = processor.parse_command("reset user password John Doe newpass123");
        assert!(result.is_ok());

        // Valid reset email command
        let result = processor.parse_command("reset user email John Doe newemail@example.com");
        assert!(result.is_ok());

        // Invalid reset command
        let result = processor.parse_command("reset user invalid John Doe value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_show_commands() {
        let processor = TerminalCommandProcessor::new();

        // Valid show account command
        let result = processor.parse_command("show account John Doe");
        assert!(result.is_ok());

        // Valid show users command
        let result = processor.parse_command("show users 50");
        assert!(result.is_ok());

        // Valid show users without limit
        let result = processor.parse_command("show users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_user_input() {
        // Valid input
        assert!(validate_user_input("John", "Doe", Some("john@example.com")).is_ok());

        // Invalid inputs
        assert!(validate_user_input("", "Doe", Some("john@example.com")).is_err());
        assert!(validate_user_input("John", "", Some("john@example.com")).is_err());
        assert!(validate_user_input("John", "Doe", Some("invalid-email")).is_err());
        assert!(validate_user_input("John", "Doe", Some("")).is_err());
    }

    #[test]
    fn test_help_command() {
        let processor = TerminalCommandProcessor::new();
        let result = processor.parse_command("help");
        assert!(result.is_ok());

        if let Ok(TerminalCommand::Help) = result {
            // Help command parsed correctly
        } else {
            panic!("Expected Help command");
        }
    }
}
