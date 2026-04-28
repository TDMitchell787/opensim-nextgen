//! Terrain generation and manipulation for regions
//!
//! This module handles the creation, modification, and serialization
//! of terrain data for OpenSim regions.

use crate::ffi::physics::PhysicsBridge;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Terrain configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerrainConfig {
    /// Terrain width in samples
    pub width_samples: usize,
    /// Terrain height in samples
    pub height_samples: usize,
    /// Terrain scale
    pub scale: f32,
    /// Base height
    pub base_height: f32,
    /// Noise parameters
    pub noise: NoiseConfig,
    /// Smoothing parameters
    pub smoothing: SmoothingConfig,
}

/// Terrain generation algorithms
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TerrainGeneration {
    /// Flat terrain
    Flat,
    /// Perlin noise terrain
    Perlin {
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    },
    /// Heightmap from file
    Heightmap { file_path: String },
    /// Custom terrain function
    Custom {
        generator: String, // Function name or script
    },
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            width_samples: 256,
            height_samples: 256,
            scale: 1.0,
            base_height: 0.0,
            noise: NoiseConfig::default(),
            smoothing: SmoothingConfig::default(),
        }
    }
}

/// Noise configuration for terrain generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseConfig {
    /// Noise amplitude
    pub amplitude: f32,
    /// Noise frequency
    pub frequency: f32,
    /// Number of octaves
    pub octaves: u32,
    /// Persistence
    pub persistence: f32,
    /// Lacunarity
    pub lacunarity: f32,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            amplitude: 10.0,
            frequency: 0.01,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

/// Smoothing configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmoothingConfig {
    /// Smoothing radius
    pub radius: f32,
    /// Smoothing strength
    pub strength: f32,
}

impl Default for SmoothingConfig {
    fn default() -> Self {
        Self {
            radius: 5.0,
            strength: 0.5,
        }
    }
}

/// Terrain region for modification
#[derive(Debug, Clone)]
pub struct TerrainRegion {
    /// Region X coordinate
    pub x: u32,
    /// Region Y coordinate
    pub y: u32,
    /// Region width
    pub width: u32,
    /// Region height
    pub height: u32,
}

/// Terrain manager for a region
pub struct TerrainManager {
    /// Terrain configuration
    config: TerrainConfig,
    /// Height data
    heights: RwLock<Vec<f32>>,
    /// Physics bridge for terrain collision
    physics_bridge: Arc<PhysicsBridge>,
    /// Terrain statistics
    stats: RwLock<TerrainStats>,
}

/// Statistics about terrain
#[derive(Debug, Clone)]
pub struct TerrainStats {
    /// Number of height samples
    pub height_samples: usize,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Last modification time
    pub last_modified: std::time::Instant,
}

impl TerrainManager {
    /// Create a new terrain manager
    pub fn new(config: TerrainConfig, physics_bridge: Arc<PhysicsBridge>) -> Self {
        let height_count = config.width_samples * config.height_samples;
        let heights = vec![config.base_height; height_count];

        Self {
            config,
            heights: RwLock::new(heights),
            physics_bridge,
            stats: RwLock::new(TerrainStats {
                height_samples: height_count,
                memory_usage: height_count * std::mem::size_of::<f32>(),
                last_modified: std::time::Instant::now(),
            }),
        }
    }

    /// Generate terrain using noise
    pub async fn generate_terrain(&self) -> Result<(), TerrainError> {
        let mut heights = self.heights.write().await;
        let region = TerrainRegion {
            x: 0,
            y: 0,
            width: self.config.width_samples as u32,
            height: self.config.height_samples as u32,
        };

        // Apply noise generation
        self.apply_noise(
            &mut heights,
            &region,
            self.config.noise.amplitude,
            self.config.noise.frequency,
            self.config.width_samples,
        )?;

        // Apply smoothing
        self.apply_smoothing(
            &mut heights,
            &region,
            self.config.smoothing.radius,
            self.config.smoothing.strength,
            self.config.width_samples,
        )?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.last_modified = std::time::Instant::now();

        Ok(())
    }

    /// Modify terrain in a specific region
    pub async fn modify_terrain(
        &self,
        region: &TerrainRegion,
        operation: TerrainOperation,
    ) -> Result<(), TerrainError> {
        let mut heights = self.heights.write().await;

        match operation {
            TerrainOperation::Raise { amount } => {
                self.apply_raise(&mut heights, region, amount)?;
            }
            TerrainOperation::Lower { amount } => {
                self.apply_lower(&mut heights, region, amount)?;
            }
            TerrainOperation::Smooth { radius, strength } => {
                self.apply_smoothing(
                    &mut heights,
                    region,
                    radius,
                    strength,
                    self.config.width_samples,
                )?;
            }
            TerrainOperation::Noise {
                amplitude,
                frequency,
            } => {
                self.apply_noise(
                    &mut heights,
                    region,
                    amplitude,
                    frequency,
                    self.config.width_samples,
                )?;
            }
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.last_modified = std::time::Instant::now();

        Ok(())
    }

    /// Get height at a specific position
    pub async fn get_height(&self, x: f32, z: f32) -> Result<f32, TerrainError> {
        let heights = self.heights.read().await;

        let x_sample = ((x / self.config.scale) * (self.config.width_samples - 1) as f32) as usize;
        let z_sample = ((z / self.config.scale) * (self.config.height_samples - 1) as f32) as usize;

        if x_sample >= self.config.width_samples || z_sample >= self.config.height_samples {
            return Err(TerrainError::PositionOutOfBounds { x, z });
        }

        let index = z_sample * self.config.width_samples + x_sample;
        Ok(heights[index])
    }

    /// Get terrain statistics
    pub async fn get_stats(&self) -> TerrainStats {
        self.stats.read().await.clone()
    }

    /// Apply raise operation to terrain
    fn apply_raise(
        &self,
        heights: &mut [f32],
        region: &TerrainRegion,
        amount: f32,
    ) -> Result<(), TerrainError> {
        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                let index = (y as usize) * self.config.width_samples + (x as usize);
                if index < heights.len() {
                    heights[index] += amount;
                }
            }
        }
        Ok(())
    }

    /// Apply lower operation to terrain
    fn apply_lower(
        &self,
        heights: &mut [f32],
        region: &TerrainRegion,
        amount: f32,
    ) -> Result<(), TerrainError> {
        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                let index = (y as usize) * self.config.width_samples + (x as usize);
                if index < heights.len() {
                    heights[index] -= amount;
                }
            }
        }
        Ok(())
    }

    /// Apply smoothing to terrain
    fn apply_smoothing(
        &self,
        heights: &mut [f32],
        region: &TerrainRegion,
        radius: f32,
        strength: f32,
        _width_samples: usize,
    ) -> Result<(), TerrainError> {
        let radius_int = radius as i32;
        let mut smoothed = heights.to_vec();

        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                let mut sum = 0.0;
                let mut count = 0;

                for dy in -radius_int..=radius_int {
                    for dx in -radius_int..=radius_int {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0
                            && nx < self.config.width_samples as i32
                            && ny >= 0
                            && ny < self.config.height_samples as i32
                        {
                            let index = (ny as usize) * self.config.width_samples + (nx as usize);
                            sum += heights[index];
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let average = sum / count as f32;
                    let index = (y as usize) * self.config.width_samples + (x as usize);
                    smoothed[index] = heights[index] * (1.0 - strength) + average * strength;
                }
            }
        }

        heights.copy_from_slice(&smoothed);
        Ok(())
    }

    /// Apply noise to terrain
    fn apply_noise(
        &self,
        heights: &mut [f32],
        region: &TerrainRegion,
        amplitude: f32,
        frequency: f32,
        _width_samples: usize,
    ) -> Result<(), TerrainError> {
        for y in region.y..(region.y + region.height) {
            for x in region.x..(region.x + region.width) {
                let x_float = x as f32 * frequency;
                let z_float = y as f32 * frequency;

                let noise_value = self.simple_noise(x_float, z_float);
                let index = (y as usize) * self.config.width_samples + (x as usize);

                if index < heights.len() {
                    heights[index] += noise_value * amplitude;
                }
            }
        }
        Ok(())
    }

    /// Simple noise function
    fn simple_noise(&self, x: f32, z: f32) -> f32 {
        let x_int = x as u32;
        let z_int = z as u32;

        // Fix the bitwise XOR operations by using consistent u32 types
        let h10 = ((x_int + 1) * 73856093_u32 ^ (z_int * 19349663_u32)).wrapping_rem(10000) as f32;
        let h01 =
            ((x_int * 73856093_u32) ^ ((z_int + 1) * 19349663_u32)).wrapping_rem(10000) as f32;
        let h11 = (((x_int + 1) * 73856093_u32) ^ ((z_int + 1) * 19349663_u32)).wrapping_rem(10000)
            as f32;

        let fx = x - x_int as f32;
        let fz = z - z_int as f32;

        let h00 = 0.0;

        // Bilinear interpolation
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;

        h0 * (1.0 - fz) + h1 * fz
    }
}

/// Terrain modification operations
#[derive(Debug, Clone)]
pub enum TerrainOperation {
    /// Raise terrain by amount
    Raise { amount: f32 },
    /// Lower terrain by amount
    Lower { amount: f32 },
    /// Smooth terrain
    Smooth { radius: f32, strength: f32 },
    /// Add noise to terrain
    Noise { amplitude: f32, frequency: f32 },
}

/// Errors that can occur in terrain operations
#[derive(Debug, thiserror::Error)]
pub enum TerrainError {
    #[error("Position out of bounds: x={x}, z={z}")]
    PositionOutOfBounds { x: f32, z: f32 },

    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terrain_creation() {
        let config = TerrainConfig::default();
        let physics_bridge = Arc::new(crate::ffi::physics::PhysicsBridge::new().unwrap());
        let terrain = TerrainManager::new(config, physics_bridge);

        let stats = terrain.get_stats().await;
        assert_eq!(stats.height_samples, 256 * 256);
    }

    #[tokio::test]
    async fn test_terrain_generation() {
        let config = TerrainConfig::default();
        let physics_bridge = Arc::new(crate::ffi::physics::PhysicsBridge::new().unwrap());
        let terrain = TerrainManager::new(config, physics_bridge);

        terrain.generate_terrain().await.unwrap();

        let height = terrain.get_height(0.0, 0.0).await.unwrap();
        assert!(height >= 0.0);
    }

    #[tokio::test]
    async fn test_terrain_modification() {
        let config = TerrainConfig::default();
        let physics_bridge = Arc::new(crate::ffi::physics::PhysicsBridge::new().unwrap());
        let terrain = TerrainManager::new(config, physics_bridge);

        terrain.generate_terrain().await.unwrap();

        let region = TerrainRegion {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };

        let operation = TerrainOperation::Raise { amount: 5.0 };
        terrain.modify_terrain(&region, operation).await.unwrap();

        let height = terrain.get_height(5.0, 5.0).await.unwrap();
        assert!(height > 0.0);
    }
}
