//! Map tile generation and management for OpenSim compatibility
//!
//! Provides maptiles/ directory support with automatic tile generation,
//! caching, and serving for region map visualization.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use image::{RgbImage, ImageFormat, DynamicImage, ImageBuffer, Rgb};
use anyhow::{Result, anyhow};
use crate::opensim_compatibility::archives::RegionInfo;

#[derive(Debug, Clone, Copy)]
struct Vector3 { x: f32, y: f32, z: f32 }
impl Vector3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn zero() -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
}

/// Map tile size constants
pub const TILE_SIZE: u32 = 256;
pub const WORLD_TILE_SIZE: u32 = 256; // World units per tile
pub const MAX_ZOOM_LEVEL: u8 = 8;

/// Map tile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTile {
    pub region_id: Uuid,
    pub x: u32,
    pub y: u32,
    pub zoom_level: u8,
    pub file_path: PathBuf,
    pub last_updated: u64,
    pub size_bytes: u64,
    pub format: TileFormat,
}

/// Supported tile formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileFormat {
    Jpeg,
    Png,
    WebP,
}

impl TileFormat {
    pub fn extension(&self) -> &str {
        match self {
            TileFormat::Jpeg => "jpg",
            TileFormat::Png => "png", 
            TileFormat::WebP => "webp",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            TileFormat::Jpeg => "image/jpeg",
            TileFormat::Png => "image/png",
            TileFormat::WebP => "image/webp",
        }
    }
}

/// Map tile generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTileConfig {
    pub tile_size: u32,
    pub zoom_levels: Vec<u8>,
    pub format: TileFormat,
    pub quality: u8, // 1-100 for JPEG
    pub compression: u8, // 1-9 for PNG
    pub enable_caching: bool,
    pub cache_duration_hours: u32,
    pub enable_auto_generation: bool,
    pub water_color: [u8; 3],
    pub land_color: [u8; 3],
    pub background_color: [u8; 3],
}

impl Default for MapTileConfig {
    fn default() -> Self {
        Self {
            tile_size: TILE_SIZE,
            zoom_levels: vec![0, 1, 2, 3, 4, 5],
            format: TileFormat::Jpeg,
            quality: 85,
            compression: 6,
            enable_caching: true,
            cache_duration_hours: 24,
            enable_auto_generation: true,
            water_color: [72, 116, 166], // Blue water
            land_color: [162, 154, 141], // Sandy land
            background_color: [0, 0, 0], // Black background
        }
    }
}

/// Map tile manager
pub struct MapTileManager {
    maptiles_path: PathBuf,
    cache_path: PathBuf,
    config: MapTileConfig,
    tile_cache: HashMap<String, MapTile>,
    region_tiles: HashMap<Uuid, Vec<MapTile>>,
    is_initialized: bool,
}

impl MapTileManager {
    /// Create a new map tile manager
    pub fn new(bin_directory: &Path, config: Option<MapTileConfig>) -> Result<Self> {
        let maptiles_path = bin_directory.join("maptiles");
        let cache_path = maptiles_path.join(".cache");

        Ok(Self {
            maptiles_path,
            cache_path,
            config: config.unwrap_or_default(),
            tile_cache: HashMap::new(),
            region_tiles: HashMap::new(),
            is_initialized: false,
        })
    }

    /// Initialize the map tile system
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        // Ensure directories exist
        self.ensure_directories()?;
        
        // Load existing tiles
        self.load_existing_tiles().await?;
        
        // Clean up old cached tiles
        self.cleanup_old_tiles().await?;
        
        self.is_initialized = true;
        tracing::info!("Map tile manager initialized with {} cached tiles", 
                      self.tile_cache.len());
        Ok(())
    }

    /// Ensure required directories exist
    fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            &self.maptiles_path,
            &self.cache_path,
            &self.maptiles_path.join("regions"),
            &self.maptiles_path.join("world"),
            &self.maptiles_path.join("tmp"),
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .map_err(|e| anyhow!(
                        format!("Failed to create maptiles directory {}: {}", dir.display(), e)
                    ))?;
                tracing::debug!("Created maptiles directory: {}", dir.display());
            }
        }

        Ok(())
    }

    /// Load existing tiles from disk
    async fn load_existing_tiles(&mut self) -> Result<()> {
        let regions_dir = self.maptiles_path.join("regions");
        
        if !regions_dir.exists() {
            return Ok(());
        }

        // Scan for existing tile files
        let entries = fs::read_dir(&regions_dir)
            .map_err(|e| anyhow!(
                format!("Failed to read regions directory: {}", e)
            ))?;

        for entry in entries {
            let entry = entry.map_err(|e| anyhow!(
                format!("Failed to read directory entry: {}", e)
            ))?;

            let path = entry.path();
            if path.is_file() {
                if let Some(tile) = self.parse_tile_from_path(&path).await? {
                    let cache_key = self.make_cache_key(&tile.region_id, tile.x, tile.y, tile.zoom_level);
                    self.tile_cache.insert(cache_key, tile.clone());
                    
                    self.region_tiles
                        .entry(tile.region_id)
                        .or_insert_with(Vec::new)
                        .push(tile);
                }
            }
        }

        tracing::debug!("Loaded {} existing tiles", self.tile_cache.len());
        Ok(())
    }

    /// Parse tile information from file path
    async fn parse_tile_from_path(&self, path: &Path) -> Result<Option<MapTile>> {
        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!(
                format!("Invalid tile filename: {}", path.display())
            ))?;

        // Expected format: regionid_x_y_zoom.ext
        let parts: Vec<&str> = file_name.split('_').collect();
        if parts.len() != 4 {
            return Ok(None);
        }

        let region_id = Uuid::parse_str(parts[0])
            .map_err(|_| anyhow!(
                format!("Invalid region ID in filename: {}", parts[0])
            ))?;

        let x: u32 = parts[1].parse()
            .map_err(|_| anyhow!(
                format!("Invalid X coordinate in filename: {}", parts[1])
            ))?;

        let y: u32 = parts[2].parse()
            .map_err(|_| anyhow!(
                format!("Invalid Y coordinate in filename: {}", parts[2])
            ))?;

        let zoom_level: u8 = parts[3].parse()
            .map_err(|_| anyhow!(
                format!("Invalid zoom level in filename: {}", parts[3])
            ))?;

        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");

        let format = match extension {
            "jpg" | "jpeg" => TileFormat::Jpeg,
            "png" => TileFormat::Png,
            "webp" => TileFormat::WebP,
            _ => TileFormat::Jpeg,
        };

        let metadata = fs::metadata(path)
            .map_err(|e| anyhow!(
                format!("Failed to get file metadata: {}", e)
            ))?;

        Ok(Some(MapTile {
            region_id,
            x,
            y,
            zoom_level,
            file_path: path.to_path_buf(),
            last_updated: metadata.modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size_bytes: metadata.len(),
            format,
        }))
    }

    /// Generate map tile for region
    pub async fn generate_region_tile(&mut self, region_info: &RegionInfo) -> Result<MapTile> {
        let tile_x = region_info.location_x / WORLD_TILE_SIZE;
        let tile_y = region_info.location_y / WORLD_TILE_SIZE;
        
        // Generate base zoom level (0) first
        let base_tile = self.generate_tile(region_info, tile_x, tile_y, 0).await?;
        
        // Generate additional zoom levels if configured
        let zoom_levels: Vec<u8> = self.config.zoom_levels.clone();
        for zoom_level in zoom_levels {
            if zoom_level > 0 {
                self.generate_tile(region_info, tile_x, tile_y, zoom_level).await?;
            }
        }
        
        Ok(base_tile)
    }

    /// Generate a specific tile
    async fn generate_tile(&mut self, region_info: &RegionInfo, x: u32, y: u32, zoom_level: u8) -> Result<MapTile> {
        let region_uuid = uuid::Uuid::parse_str(&region_info.uuid)
            .map_err(|e| anyhow!(format!("Invalid region UUID: {}", e)))?;
        let cache_key = self.make_cache_key(&region_uuid, x, y, zoom_level);
        
        // Check if tile exists and is recent enough
        if let Some(existing_tile) = self.tile_cache.get(&cache_key) {
            if self.is_tile_fresh(existing_tile) {
                return Ok(existing_tile.clone());
            }
        }

        // Generate new tile
        let tile_data = self.render_tile(region_info, x, y, zoom_level).await?;
        let file_name = format!("{}_{}_{}_{}.{}", 
                               region_info.uuid, x, y, zoom_level, 
                               self.config.format.extension());
        let file_path = self.maptiles_path.join("regions").join(&file_name);

        // Write tile data to disk
        fs::write(&file_path, &tile_data)
            .map_err(|e| anyhow!(
                format!("Failed to write tile file {}: {}", file_path.display(), e)
            ))?;

        let tile = MapTile {
            region_id: region_uuid,
            x,
            y,
            zoom_level,
            file_path: file_path.clone(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size_bytes: tile_data.len() as u64,
            format: self.config.format.clone(),
        };

        // Update cache
        self.tile_cache.insert(cache_key, tile.clone());
        self.region_tiles
            .entry(region_uuid)
            .or_insert_with(Vec::new)
            .push(tile.clone());

        tracing::debug!("Generated tile for region {} at {},{} zoom {}", 
                       region_info.name, x, y, zoom_level);
        Ok(tile)
    }

    /// Render tile image data
    async fn render_tile(&self, region_info: &RegionInfo, x: u32, y: u32, zoom_level: u8) -> Result<Vec<u8>> {
        let scale = 2_u32.pow(zoom_level as u32);
        let tile_size = self.config.tile_size / scale;
        
        // Create a simple terrain-based tile
        // In a real implementation, this would render the actual terrain heightmap,
        // objects, water bodies, etc.
        let mut image_data = Vec::with_capacity((tile_size * tile_size * 3) as usize);
        
        for py in 0..tile_size {
            for px in 0..tile_size {
                // Simple procedural terrain generation based on region position
                let world_x = (x * WORLD_TILE_SIZE) + (px * WORLD_TILE_SIZE / tile_size);
                let world_y = (y * WORLD_TILE_SIZE) + (py * WORLD_TILE_SIZE / tile_size);
                
                let height = self.get_terrain_height(region_info, world_x as f32, world_y as f32);
                let color = self.get_terrain_color(height);
                
                image_data.extend_from_slice(&color);
            }
        }

        // Convert raw RGB data to the configured format
        self.encode_image_data(image_data, tile_size, tile_size).await
    }

    /// Get terrain height at world coordinates (simplified)
    fn get_terrain_height(&self, region_info: &RegionInfo, x: f32, y: f32) -> f32 {
        // Simplified terrain height calculation
        // In a real implementation, this would query the actual terrain heightmap
        let region_x = x - region_info.location_x as f32;
        let region_y = y - region_info.location_y as f32;
        
        // Simple noise-based height
        let noise = ((region_x * 0.01).sin() + (region_y * 0.01).sin()) * 10.0;
        20.0 + noise // Base height + variation
    }

    /// Get terrain color based on height
    fn get_terrain_color(&self, height: f32) -> [u8; 3] {
        match height {
            h if h < 19.0 => self.config.water_color, // Water
            h if h < 25.0 => [92, 51, 23], // Beach/sand
            h if h < 40.0 => [34, 139, 34], // Grass
            h if h < 60.0 => [107, 142, 35], // Hills  
            _ => [169, 169, 169], // Mountains/rocks
        }
    }

    async fn encode_image_data(&self, rgb_data: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>> {
        let expected_size = (width * height * 3) as usize;
        if rgb_data.len() != expected_size {
            return Err(anyhow!(
                format!("RGB data size mismatch: expected {} bytes, got {}", expected_size, rgb_data.len())
            ));
        }

        let img: RgbImage = ImageBuffer::from_raw(width, height, rgb_data)
            .ok_or_else(|| anyhow!(
                "Failed to create image buffer from RGB data".to_string()
            ))?;

        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut output = Cursor::new(Vec::new());

        match self.config.format {
            TileFormat::Jpeg => {
                dynamic_img.write_to(&mut output, ImageFormat::Jpeg)
                    .map_err(|e| anyhow!(
                        format!("Failed to encode JPEG: {}", e)
                    ))?;
            },
            TileFormat::Png => {
                dynamic_img.write_to(&mut output, ImageFormat::Png)
                    .map_err(|e| anyhow!(
                        format!("Failed to encode PNG: {}", e)
                    ))?;
            },
            TileFormat::WebP => {
                dynamic_img.write_to(&mut output, ImageFormat::WebP)
                    .map_err(|e| anyhow!(
                        format!("Failed to encode WebP: {}", e)
                    ))?;
            },
        }

        Ok(output.into_inner())
    }

    /// Check if tile is fresh enough to avoid regeneration
    fn is_tile_fresh(&self, tile: &MapTile) -> bool {
        if !self.config.enable_caching {
            return false;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_hours = (now - tile.last_updated) / 3600;
        age_hours < self.config.cache_duration_hours as u64
    }

    /// Make cache key for tile
    fn make_cache_key(&self, region_id: &Uuid, x: u32, y: u32, zoom_level: u8) -> String {
        format!("{}_{}_{}_{}", region_id, x, y, zoom_level)
    }

    /// Get tile for region at coordinates
    pub async fn get_tile(&self, region_id: &Uuid, x: u32, y: u32, zoom_level: u8) -> Option<&MapTile> {
        let cache_key = self.make_cache_key(region_id, x, y, zoom_level);
        self.tile_cache.get(&cache_key)
    }

    /// Get all tiles for a region
    pub fn get_region_tiles(&self, region_id: &Uuid) -> Option<&Vec<MapTile>> {
        self.region_tiles.get(region_id)
    }

    /// Delete tiles for a region
    pub async fn delete_region_tiles(&mut self, region_id: &Uuid) -> Result<()> {
        if let Some(tiles) = self.region_tiles.remove(region_id) {
            for tile in tiles {
                // Remove from cache
                let cache_key = self.make_cache_key(&tile.region_id, tile.x, tile.y, tile.zoom_level);
                self.tile_cache.remove(&cache_key);

                // Delete file
                if tile.file_path.exists() {
                    fs::remove_file(&tile.file_path)
                        .map_err(|e| anyhow!(
                            format!("Failed to delete tile file {}: {}", tile.file_path.display(), e)
                        ))?;
                }
            }
            tracing::debug!("Deleted all tiles for region {}", region_id);
        }

        Ok(())
    }

    /// Clean up old cached tiles
    async fn cleanup_old_tiles(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut tiles_to_remove = Vec::new();

        for (cache_key, tile) in &self.tile_cache {
            let age_hours = (now - tile.last_updated) / 3600;
            if age_hours > (self.config.cache_duration_hours as u64 * 2) {
                tiles_to_remove.push(cache_key.clone());
            }
        }

        for cache_key in tiles_to_remove {
            if let Some(tile) = self.tile_cache.remove(&cache_key) {
                if tile.file_path.exists() {
                    fs::remove_file(&tile.file_path)
                        .map_err(|e| anyhow!(
                            format!("Failed to delete old tile {}: {}", tile.file_path.display(), e)
                        ))?;
                }

                // Remove from region tiles
                if let Some(region_tiles) = self.region_tiles.get_mut(&tile.region_id) {
                    region_tiles.retain(|t| t.file_path != tile.file_path);
                }
            }
        }

        Ok(())
    }

    /// Get tile data as bytes
    pub async fn get_tile_data(&self, tile: &MapTile) -> Result<Vec<u8>> {
        fs::read(&tile.file_path)
            .map_err(|e| anyhow!(
                format!("Failed to read tile file {}: {}", tile.file_path.display(), e)
            ))
    }

    /// Get configuration
    pub fn get_config(&self) -> &MapTileConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: MapTileConfig) {
        self.config = config;
    }

    /// Get statistics
    pub fn get_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_tiles".to_string(), self.tile_cache.len() as u64);
        stats.insert("regions_with_tiles".to_string(), self.region_tiles.len() as u64);
        
        let total_size: u64 = self.tile_cache.values().map(|t| t.size_bytes).sum();
        stats.insert("total_size_bytes".to_string(), total_size);
        
        // Count by zoom level
        for zoom_level in 0..=MAX_ZOOM_LEVEL {
            let count = self.tile_cache.values()
                .filter(|t| t.zoom_level == zoom_level)
                .count() as u64;
            stats.insert(format!("zoom_{}_tiles", zoom_level), count);
        }
        
        stats
    }

    /// Get tile paths
    pub fn get_tile_paths(&self) -> HashMap<String, PathBuf> {
        let mut paths = HashMap::new();
        
        paths.insert("maptiles".to_string(), self.maptiles_path.clone());
        paths.insert("cache".to_string(), self.cache_path.clone());
        paths.insert("regions".to_string(), self.maptiles_path.join("regions"));
        paths.insert("world".to_string(), self.maptiles_path.join("world"));
        
        paths
    }

    /// Generate world map tile from multiple regions
    pub async fn generate_world_tile(&mut self, regions: &[RegionInfo], world_x: u32, world_y: u32, zoom_level: u8) -> Result<MapTile> {
        // This would combine multiple region tiles into a world tile
        // For now, create a placeholder implementation
        let tile_data = self.render_world_tile(regions, world_x, world_y, zoom_level).await?;
        
        let file_name = format!("world_{}_{}_{}.{}", 
                               world_x, world_y, zoom_level, 
                               self.config.format.extension());
        let file_path = self.maptiles_path.join("world").join(&file_name);

        fs::write(&file_path, &tile_data)
            .map_err(|e| anyhow!(
                format!("Failed to write world tile: {}", e)
            ))?;

        let tile = MapTile {
            region_id: Uuid::nil(), // World tiles don't belong to a specific region
            x: world_x,
            y: world_y,
            zoom_level,
            file_path,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size_bytes: tile_data.len() as u64,
            format: self.config.format.clone(),
        };

        Ok(tile)
    }

    /// Render world tile combining multiple regions
    async fn render_world_tile(&self, regions: &[RegionInfo], world_x: u32, world_y: u32, zoom_level: u8) -> Result<Vec<u8>> {
        let scale = 2_u32.pow(zoom_level as u32);
        let tile_size = self.config.tile_size / scale;
        
        let mut image_data = Vec::with_capacity((tile_size * tile_size * 3) as usize);
        
        for py in 0..tile_size {
            for px in 0..tile_size {
                let world_coord_x = (world_x * WORLD_TILE_SIZE) + (px * WORLD_TILE_SIZE / tile_size);
                let world_coord_y = (world_y * WORLD_TILE_SIZE) + (py * WORLD_TILE_SIZE / tile_size);
                
                // Find region containing this coordinate
                let mut color = self.config.background_color;
                for region in regions {
                    if world_coord_x >= region.location_x && 
                       world_coord_x < (region.location_x + region.size_x) &&
                       world_coord_y >= region.location_y && 
                       world_coord_y < (region.location_y + region.size_y) {
                        let height = self.get_terrain_height(region, world_coord_x as f32, world_coord_y as f32);
                        color = self.get_terrain_color(height);
                        break;
                    }
                }
                
                image_data.extend_from_slice(&color);
            }
        }

        self.encode_image_data(image_data, tile_size, tile_size).await
    }
}