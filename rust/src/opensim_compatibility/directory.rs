//! OpenSim directory structure management
//!
//! Ensures compatibility with OpenSimulator's expected directory layout.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Required OpenSim directories
const REQUIRED_DIRECTORIES: &[&str] = &[
    "config-include",
    "Regions",
    "assetcache",
    "assets",
    "addon-modules",
    "data",
    "inventory",
    "Library",
    "maptiles",
    "Physics",
    "ScriptEngines",
];

/// Optional OpenSim directories
const OPTIONAL_DIRECTORIES: &[&str] = &[
    "crashes",
    "logs",
    "userprofiles",
    "UserProfiles",
    "j2kDecodeCache",
];

/// Ensure OpenSim directory structure exists
pub fn ensure_opensim_structure(bin_directory: &Path) -> Result<()> {
    // Create bin directory if it doesn't exist
    if !bin_directory.exists() {
        fs::create_dir_all(bin_directory).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create bin directory {}: {}",
                bin_directory.display(),
                e
            )
        })?;
    }

    // Create required directories
    for dir_name in REQUIRED_DIRECTORIES {
        let dir_path = bin_directory.join(dir_name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).map_err(|e| {
                anyhow::anyhow!("Failed to create directory {}: {}", dir_path.display(), e)
            })?;
            tracing::debug!("Created directory: {}", dir_path.display());
        }
    }

    // Create config-include subdirectories
    let config_include = bin_directory.join("config-include");
    let config_subdirs = &["storage", "services"];
    for subdir in config_subdirs {
        let subdir_path = config_include.join(subdir);
        if !subdir_path.exists() {
            fs::create_dir_all(&subdir_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create config subdirectory {}: {}",
                    subdir_path.display(),
                    e
                )
            })?;
        }
    }

    // Create Library subdirectories
    let library_dir = bin_directory.join("Library");
    let library_subdirs = &["LibraryAssets", "inventory"];
    for subdir in library_subdirs {
        let subdir_path = library_dir.join(subdir);
        if !subdir_path.exists() {
            fs::create_dir_all(&subdir_path).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create library subdirectory {}: {}",
                    subdir_path.display(),
                    e
                )
            })?;
        }
    }

    tracing::info!(
        "OpenSim directory structure verified/created at: {}",
        bin_directory.display()
    );
    Ok(())
}

/// Get OpenSim directory paths
pub struct OpenSimDirectories {
    pub bin: PathBuf,
    pub config_include: PathBuf,
    pub regions: PathBuf,
    pub asset_cache: PathBuf,
    pub assets: PathBuf,
    pub addon_modules: PathBuf,
    pub data: PathBuf,
    pub inventory: PathBuf,
    pub library: PathBuf,
    pub maptiles: PathBuf,
    pub physics: PathBuf,
    pub script_engines: PathBuf,
}

impl OpenSimDirectories {
    /// Create directory structure from bin path
    pub fn new(bin_directory: PathBuf) -> Self {
        Self {
            config_include: bin_directory.join("config-include"),
            regions: bin_directory.join("Regions"),
            asset_cache: bin_directory.join("assetcache"),
            assets: bin_directory.join("assets"),
            addon_modules: bin_directory.join("addon-modules"),
            data: bin_directory.join("data"),
            inventory: bin_directory.join("inventory"),
            library: bin_directory.join("Library"),
            maptiles: bin_directory.join("maptiles"),
            physics: bin_directory.join("Physics"),
            script_engines: bin_directory.join("ScriptEngines"),
            bin: bin_directory,
        }
    }

    /// Verify all directories exist
    pub fn verify(&self) -> Result<()> {
        let paths = [
            &self.bin,
            &self.config_include,
            &self.regions,
            &self.asset_cache,
            &self.assets,
            &self.addon_modules,
            &self.data,
            &self.inventory,
            &self.library,
            &self.maptiles,
            &self.physics,
            &self.script_engines,
        ];

        for path in &paths {
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Required directory missing: {}",
                    path.display()
                ));
            }
        }

        Ok(())
    }

    /// Create missing directories
    pub fn create_missing(&self) -> Result<()> {
        ensure_opensim_structure(&self.bin)
    }
}

/// Check if directory structure is compatible with OpenSim
pub fn is_opensim_compatible(bin_directory: &Path) -> bool {
    if !bin_directory.is_dir() {
        return false;
    }

    // Check for essential directories
    let essential_dirs = &["config-include", "Regions"];
    for dir_name in essential_dirs {
        if !bin_directory.join(dir_name).exists() {
            return false;
        }
    }

    // Check for OpenSim.ini
    if !bin_directory.join("OpenSim.ini").exists() {
        return false;
    }

    true
}

/// Migrate from old directory structure to OpenSim-compatible structure
pub fn migrate_directory_structure(old_path: &Path, new_path: &Path) -> Result<()> {
    if !old_path.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {}",
            old_path.display()
        ));
    }

    // Create new structure
    ensure_opensim_structure(new_path)?;

    // Copy configuration files if they exist
    let config_files = &[
        "OpenSim.ini",
        "config-include/Standalone.ini",
        "config-include/GridCommon.ini",
        "Regions/Regions.ini",
    ];

    for config_file in config_files {
        let old_file = old_path.join(config_file);
        let new_file = new_path.join(config_file);

        if old_file.exists() {
            if let Some(parent) = new_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&old_file, &new_file)
                .map_err(|e| anyhow::anyhow!("Failed to copy {}: {}", config_file, e))?;
            tracing::info!("Migrated config file: {}", config_file);
        }
    }

    tracing::info!("Directory structure migration completed");
    Ok(())
}
