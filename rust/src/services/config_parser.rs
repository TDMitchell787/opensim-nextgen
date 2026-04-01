use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn, debug};

use crate::services::traits::{ServiceConfig, ServiceMode};

pub fn parse_service_config(ini_path: &str) -> Result<ServiceConfig> {
    let content = std::fs::read_to_string(ini_path)
        .map_err(|e| anyhow!("Failed to read config file '{}': {}", ini_path, e))?;

    let sections = parse_ini_sections(&content);
    let mut config = ServiceConfig::default();

    if let Some(startup) = sections.get("Startup") {
        if let Some(mode) = startup.get("ServiceMode") {
            config.mode = match mode.to_lowercase().as_str() {
                "grid" => ServiceMode::Grid,
                "hybrid" => ServiceMode::Hybrid,
                _ => ServiceMode::Standalone,
            };
        }
    }

    if let Some(grid) = sections.get("GridService") {
        if let Some(uri) = grid.get("GridServerURI").or_else(|| grid.get("ServerURI")) {
            config.grid_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(user) = sections.get("UserAccountService") {
        if let Some(uri) = user.get("UserAccountServerURI").or_else(|| user.get("ServerURI")) {
            config.user_account_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(asset) = sections.get("AssetService") {
        if let Some(uri) = asset.get("AssetServerURI").or_else(|| asset.get("ServerURI")) {
            config.asset_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(auth) = sections.get("AuthenticationService") {
        if let Some(uri) = auth.get("AuthenticationServerURI").or_else(|| auth.get("ServerURI")) {
            config.authentication_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(inv) = sections.get("InventoryService") {
        if let Some(uri) = inv.get("InventoryServerURI").or_else(|| inv.get("ServerURI")) {
            config.inventory_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(presence) = sections.get("PresenceService") {
        if let Some(uri) = presence.get("PresenceServerURI").or_else(|| presence.get("ServerURI")) {
            config.presence_server_uri = Some(normalize_uri(uri));
        }
    }

    if let Some(avatar) = sections.get("AvatarService") {
        if let Some(uri) = avatar.get("AvatarServerURI").or_else(|| avatar.get("ServerURI")) {
            config.avatar_server_uri = Some(normalize_uri(uri));
        }
    }

    Ok(config)
}

pub fn service_config_from_env() -> ServiceConfig {
    let mut config = ServiceConfig::default();

    if let Ok(mode) = std::env::var("OPENSIM_SERVICE_MODE") {
        config.mode = match mode.to_lowercase().as_str() {
            "grid" => ServiceMode::Grid,
            "robust" => ServiceMode::Standalone,
            "hybrid" => ServiceMode::Hybrid,
            _ => ServiceMode::Standalone,
        };
    }

    if let Ok(uri) = std::env::var("OPENSIM_GRID_URI") {
        config.grid_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_ASSET_URI") {
        config.asset_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_INVENTORY_URI") {
        config.inventory_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_USER_ACCOUNT_URI") {
        config.user_account_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_PRESENCE_URI") {
        config.presence_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_AVATAR_URI") {
        config.avatar_server_uri = Some(normalize_uri(&uri));
    }
    if let Ok(uri) = std::env::var("OPENSIM_AUTH_URI") {
        config.authentication_server_uri = Some(normalize_uri(&uri));
    }

    config
}

pub fn build_service_config(mode_str: &str) -> ServiceConfig {
    let mut config = service_config_from_env();

    let effective_mode = match mode_str {
        "grid" => ServiceMode::Grid,
        "robust" => ServiceMode::Standalone,
        "hybrid" => ServiceMode::Hybrid,
        _ => config.mode,
    };
    config.mode = effective_mode;

    let grid_common_path = "bin/config-include/GridCommon.ini";
    if Path::new(grid_common_path).exists() {
        debug!("Loading service config from {}", grid_common_path);
        if let Ok(file_config) = parse_service_config(grid_common_path) {
            if config.grid_server_uri.is_none() {
                config.grid_server_uri = file_config.grid_server_uri;
            }
            if config.asset_server_uri.is_none() {
                config.asset_server_uri = file_config.asset_server_uri;
            }
            if config.inventory_server_uri.is_none() {
                config.inventory_server_uri = file_config.inventory_server_uri;
            }
            if config.user_account_server_uri.is_none() {
                config.user_account_server_uri = file_config.user_account_server_uri;
            }
            if config.presence_server_uri.is_none() {
                config.presence_server_uri = file_config.presence_server_uri;
            }
            if config.avatar_server_uri.is_none() {
                config.avatar_server_uri = file_config.avatar_server_uri;
            }
            if config.authentication_server_uri.is_none() {
                config.authentication_server_uri = file_config.authentication_server_uri;
            }
        }
    }

    if effective_mode == ServiceMode::Grid {
        let default_robust = "http://localhost:8003".to_string();
        if config.grid_server_uri.is_none() {
            warn!("Grid mode: no GridServerURI configured, using default {}", default_robust);
            config.grid_server_uri = Some(default_robust.clone());
        }
        if config.asset_server_uri.is_none() {
            config.asset_server_uri = config.grid_server_uri.clone();
        }
        if config.inventory_server_uri.is_none() {
            config.inventory_server_uri = config.grid_server_uri.clone();
        }
        if config.user_account_server_uri.is_none() {
            config.user_account_server_uri = config.grid_server_uri.clone();
        }
        if config.presence_server_uri.is_none() {
            config.presence_server_uri = config.grid_server_uri.clone();
        }
        if config.avatar_server_uri.is_none() {
            config.avatar_server_uri = config.grid_server_uri.clone();
        }
        if config.authentication_server_uri.is_none() {
            config.authentication_server_uri = config.grid_server_uri.clone();
        }
    }

    info!("Service config built: mode={:?}", config.mode);
    config
}

fn parse_ini_sections(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
            continue;
        }

        if !current_section.is_empty() {
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                sections.entry(current_section.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
    }

    sections
}

pub fn build_hypergrid_config() -> crate::network::hypergrid::HypergridConfig {
    let enabled = std::env::var("OPENSIM_HYPERGRID_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let home_uri = std::env::var("OPENSIM_HOME_URI")
        .unwrap_or_else(|_| "http://localhost:8002".to_string());
    let gatekeeper_uri = std::env::var("OPENSIM_GATEKEEPER_URI")
        .unwrap_or_else(|_| home_uri.clone());
    let grid_name = std::env::var("OPENSIM_GRID_NAME")
        .unwrap_or_else(|_| "OpenSim Next".to_string());
    let allow_teleports = std::env::var("OPENSIM_HG_ALLOW_TELEPORTS_TO_ANY_REGION")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let foreign_agents = std::env::var("OPENSIM_HG_FOREIGN_AGENTS_ALLOWED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").ok();
    let grid_common_path = if let Some(ref dir) = instance_dir {
        format!("{}/bin/config-include/GridCommon.ini", dir)
    } else {
        "bin/config-include/GridCommon.ini".to_string()
    };
    let external_uri = std::env::var("OPENSIM_HG_EXTERNAL_URI")
        .unwrap_or_default();

    let robust_port = std::env::var("OPENSIM_ROBUST_PORT")
        .unwrap_or_else(|_| "8003".to_string());
    let external_robust_uri = if !external_uri.is_empty() {
        if let Ok(mut url) = url::Url::parse(&external_uri) {
            let _ = url.set_port(robust_port.parse::<u16>().ok());
            url.as_str().trim_end_matches('/').to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let mut cfg = crate::network::hypergrid::HypergridConfig {
        enabled,
        home_uri,
        gatekeeper_uri,
        external_uri,
        external_robust_uri,
        grid_name,
        allow_teleports_to_any_region: allow_teleports,
        foreign_agents_allowed: foreign_agents,
    };

    if Path::new(&grid_common_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&grid_common_path) {
            let sections = parse_ini_sections(&content);
            if let Some(hg) = sections.get("Hypergrid") {
                if let Some(v) = hg.get("Enabled") {
                    cfg.enabled = v.to_lowercase() == "true";
                }
            }
            if let Some(gk) = sections.get("GatekeeperService") {
                if let Some(v) = gk.get("ExternalName") {
                    cfg.gatekeeper_uri = normalize_uri(v);
                    if cfg.home_uri == "http://localhost:8002" {
                        cfg.home_uri = cfg.gatekeeper_uri.clone();
                    }
                }
            }
        }
    }

    info!("Hypergrid config: enabled={}, home={}, gk={}, ext={}, robust={}, grid='{}'",
          cfg.enabled, cfg.home_uri, cfg.gatekeeper_uri, cfg.external_uri, cfg.external_robust_uri, cfg.grid_name);
    cfg
}

fn normalize_uri(uri: &str) -> String {
    let trimmed = uri.trim().trim_matches('"');
    if trimmed.ends_with('/') {
        trimmed[..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_sections() {
        let content = r#"
[Startup]
ServiceMode = grid

[GridService]
GridServerURI = "http://robust.example.com:8003"

[AssetService]
ServerURI = http://robust.example.com:8003/
"#;
        let sections = parse_ini_sections(content);
        assert_eq!(sections["Startup"]["ServiceMode"], "grid");
        assert_eq!(sections["GridService"]["GridServerURI"], "http://robust.example.com:8003");
        assert_eq!(sections["AssetService"]["ServerURI"], "http://robust.example.com:8003/");
    }

    #[test]
    fn test_normalize_uri() {
        assert_eq!(normalize_uri("http://localhost:8003/"), "http://localhost:8003");
        assert_eq!(normalize_uri("http://localhost:8003"), "http://localhost:8003");
        assert_eq!(normalize_uri("\"http://localhost:8003\""), "http://localhost:8003");
    }
}
