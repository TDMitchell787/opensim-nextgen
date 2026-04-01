//! OpenSim compatibility layer
//!
//! This module provides compatibility with existing OpenSimulator installations,
//! including INI configuration parsing, directory structure management, and
//! legacy protocol support.

pub mod config;
pub mod directory;
pub mod protocols;
pub mod message_templates;
pub mod modules;
pub mod archives;
pub mod asset_cache;
pub mod database_compatibility;
pub mod region_module_compatibility;
pub mod library_assets;
pub mod maptiles;
pub mod avatar_data;
pub mod animations;

use std::path::PathBuf;
use uuid::Uuid;
use anyhow::{Result, anyhow};

/// OpenSim compatibility manager
pub struct OpenSimCompatibility {
    pub bin_directory: PathBuf,
    pub config_parser: config::IniConfigParser,
    pub module_loader: modules::ModuleLoader,
    pub archive_manager: archives::ArchiveManager,
    pub asset_cache: asset_cache::AssetCacheManager,
    pub database_compatibility: Option<database_compatibility::DatabaseCompatibilityManager>,
    pub region_module_compatibility: region_module_compatibility::RegionModuleCompatibility,
    pub library_assets: library_assets::LibraryAssetManager,
    pub maptiles: maptiles::MapTileManager,
}

impl OpenSimCompatibility {
    /// Create a new OpenSim compatibility manager
    pub fn new(bin_directory: PathBuf) -> Result<Self> {
        let asset_cache_config = asset_cache::AssetCacheConfig {
            cache_directory: bin_directory.join("assetcache"),
            ..Default::default()
        };
        
        Ok(Self {
            bin_directory: bin_directory.clone(),
            config_parser: config::IniConfigParser::new().map_err(|e| anyhow!(e.to_string()))?,
            module_loader: modules::ModuleLoader::new(bin_directory.join("addon-modules")).map_err(|e| anyhow!(e.to_string()))?,
            archive_manager: archives::ArchiveManager::new(bin_directory.clone()).map_err(|e| anyhow!(e.to_string()))?,
            asset_cache: asset_cache::AssetCacheManager::new(asset_cache_config).map_err(|e| anyhow!(e.to_string()))?,
            database_compatibility: None, // Initialized when database is available
            region_module_compatibility: region_module_compatibility::RegionModuleCompatibility::new(),
            library_assets: library_assets::LibraryAssetManager::new(&bin_directory).map_err(|e| anyhow!(e.to_string()))?,
            maptiles: maptiles::MapTileManager::new(&bin_directory, None).map_err(|e| anyhow!(e.to_string()))?,
        })
    }

    /// Initialize OpenSim compatibility
    pub async fn initialize(&mut self) -> Result<()> {
        // Ensure directory structure exists
        directory::ensure_opensim_structure(&self.bin_directory)
            .map_err(|e| anyhow!(format!("Directory setup failed: {}", e)))?;
        
        // Load main configuration
        let main_config_path = self.bin_directory.join("OpenSim.ini");
        if main_config_path.exists() {
            self.config_parser.load_main_config(&main_config_path)?;
        }
        
        // Initialize module loader
        self.module_loader.scan_modules().await
            .map_err(|e| anyhow!(format!("Module scan failed: {}", e)))?;
        
        // Initialize library assets
        self.library_assets.initialize().await?;
        
        // Initialize maptiles system
        self.maptiles.initialize().await?;

        // Initialize animations
        if let Err(e) = animations::init_global_animations(&self.bin_directory) {
            tracing::warn!("Failed to load animations: {}", e);
        }

        tracing::info!("OpenSim compatibility layer initialized");
        Ok(())
    }

    /// Get configuration value with OpenSim compatibility
    pub fn get_config_value(&self, section: &str, key: &str) -> Option<String> {
        self.config_parser.get_value(section, key)
    }

    /// Load region configurations
    pub fn load_region_configs(&self) -> Result<Vec<config::RegionConfig>> {
        let regions_path = self.bin_directory.join("Regions/Regions.ini");
        self.config_parser.load_region_configs(&regions_path)
    }

    /// Get loaded modules
    pub fn get_loaded_modules(&self) -> &[modules::LoadedModule] {
        self.module_loader.get_modules()
    }

    /// Generate map tile for region
    pub async fn generate_region_maptile(&mut self, region_info: &crate::opensim_compatibility::archives::RegionInfo) -> Result<()> {
        self.maptiles.generate_region_tile(region_info).await?;
        Ok(())
    }

    /// Get map tile for region
    pub async fn get_region_maptile(&self, region_id: &Uuid, x: u32, y: u32, zoom_level: u8) -> Option<&maptiles::MapTile> {
        self.maptiles.get_tile(region_id, x, y, zoom_level).await
    }
}