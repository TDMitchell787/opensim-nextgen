//! Spatial indexing system for efficient entity queries
//!
//! This module provides spatial partitioning for efficient entity queries
//! and collision detection using a grid-based approach.

use crate::ffi::Vec3;
use crate::region::scene::entity::EntityId;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Spatial query types
#[derive(Debug, Clone)]
pub enum SpatialQuery {
    /// Query entities within a radius
    Radius { center: Vec3, radius: f32 },
    /// Query entities in a box
    Box { min: Vec3, max: Vec3 },
    /// Query entities at a specific position
    Point { position: Vec3 },
}

/// Spatial index for efficient entity queries
pub struct SpatialIndex {
    /// Grid cell size
    cell_size: f32,
    /// Grid dimensions
    width: u32,
    height: u32,
    /// Entity positions
    entity_positions: RwLock<HashMap<EntityId, Vec3>>,
    /// Spatial grid
    grid: RwLock<HashMap<(i32, i32), Vec<EntityId>>>,
}

impl SpatialIndex {
    /// Create a new spatial index
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            cell_size: 10.0, // 10 meter cells
            width,
            height,
            entity_positions: RwLock::new(HashMap::new()),
            grid: RwLock::new(HashMap::new()),
        }
    }

    /// Add an entity to the spatial index
    pub async fn add_entity(&self, entity_id: EntityId, position: Vec3) {
        let mut positions = self.entity_positions.write().await;
        positions.insert(entity_id, position);

        let mut grid = self.grid.write().await;
        let cell = self.position_to_cell(position);
        grid.entry(cell).or_insert_with(Vec::new).push(entity_id);
    }

    /// Remove an entity from the spatial index
    pub async fn remove_entity(&self, entity_id: EntityId) {
        let mut positions = self.entity_positions.write().await;
        if let Some(position) = positions.remove(&entity_id) {
            let mut grid = self.grid.write().await;
            let cell = self.position_to_cell(position);
            if let Some(entities) = grid.get_mut(&cell) {
                entities.retain(|&id| id != entity_id);
                if entities.is_empty() {
                    grid.remove(&cell);
                }
            }
        }
    }

    /// Update entity position in the spatial index
    pub async fn update_entity_position(&self, entity_id: EntityId, new_position: Vec3) {
        let mut positions = self.entity_positions.write().await;
        if let Some(old_position) = positions.get(&entity_id) {
            let old_cell = self.position_to_cell(*old_position);
            let new_cell = self.position_to_cell(new_position);

            if old_cell != new_cell {
                let mut grid = self.grid.write().await;

                // Remove from old cell
                if let Some(entities) = grid.get_mut(&old_cell) {
                    entities.retain(|&id| id != entity_id);
                    if entities.is_empty() {
                        grid.remove(&old_cell);
                    }
                }

                // Add to new cell
                grid.entry(new_cell)
                    .or_insert_with(Vec::new)
                    .push(entity_id);
            }
        }
        positions.insert(entity_id, new_position);
    }

    /// Query entities using a spatial query
    pub async fn query(&self, query: SpatialQuery) -> Vec<EntityId> {
        let positions = self.entity_positions.read().await;
        let grid = self.grid.read().await;

        match query {
            SpatialQuery::Radius { center, radius } => {
                let mut result = Vec::new();
                let cells = self.get_cells_in_radius(center, radius);

                for cell in cells {
                    if let Some(entities) = grid.get(&cell) {
                        for &entity_id in entities {
                            if let Some(position) = positions.get(&entity_id) {
                                let distance = self.distance(center, *position);
                                if distance <= radius {
                                    result.push(entity_id);
                                }
                            }
                        }
                    }
                }
                result
            }
            SpatialQuery::Box { min, max } => {
                let mut result = Vec::new();
                let cells = self.get_cells_in_box(min, max);

                for cell in cells {
                    if let Some(entities) = grid.get(&cell) {
                        for &entity_id in entities {
                            if let Some(position) = positions.get(&entity_id) {
                                if self.is_in_box(*position, min, max) {
                                    result.push(entity_id);
                                }
                            }
                        }
                    }
                }
                result
            }
            SpatialQuery::Point { position } => {
                let cell = self.position_to_cell(position);
                grid.get(&cell).cloned().unwrap_or_default()
            }
        }
    }

    /// Convert position to grid cell coordinates
    fn position_to_cell(&self, position: Vec3) -> (i32, i32) {
        let x = (position.x / self.cell_size).floor() as i32;
        let z = (position.z / self.cell_size).floor() as i32;
        (x, z)
    }

    /// Get cells within a radius
    fn get_cells_in_radius(&self, center: Vec3, radius: f32) -> Vec<(i32, i32)> {
        let center_cell = self.position_to_cell(center);
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let mut cells = Vec::new();

        for x in (center_cell.0 - cell_radius)..=(center_cell.0 + cell_radius) {
            for z in (center_cell.1 - cell_radius)..=(center_cell.1 + cell_radius) {
                cells.push((x, z));
            }
        }
        cells
    }

    /// Get cells within a box
    fn get_cells_in_box(&self, min: Vec3, max: Vec3) -> Vec<(i32, i32)> {
        let min_cell = self.position_to_cell(min);
        let max_cell = self.position_to_cell(max);
        let mut cells = Vec::new();

        for x in min_cell.0..=max_cell.0 {
            for z in min_cell.1..=max_cell.1 {
                cells.push((x, z));
            }
        }
        cells
    }

    /// Check if position is within a box
    fn is_in_box(&self, position: Vec3, min: Vec3, max: Vec3) -> bool {
        position.x >= min.x
            && position.x <= max.x
            && position.y >= min.y
            && position.y <= max.y
            && position.z >= min.z
            && position.z <= max.z
    }

    /// Calculate distance between two positions
    fn distance(&self, a: Vec3, b: Vec3) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Errors that can occur in spatial operations
#[derive(Debug, thiserror::Error)]
pub enum SpatialError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spatial_index_creation() {
        let index = SpatialIndex::new(256, 256);
        assert_eq!(index.cell_size, 10.0);
    }

    #[tokio::test]
    async fn test_entity_add_remove() {
        let index = SpatialIndex::new(256, 256);
        let entity_id = EntityId(1);
        let position = Vec3::new(0.0, 0.0, 0.0);

        index.add_entity(entity_id, position).await;
        assert!(index.entity_positions.read().await.contains_key(&entity_id));

        index.remove_entity(entity_id).await;
        assert!(!index.entity_positions.read().await.contains_key(&entity_id));
    }

    #[tokio::test]
    async fn test_radius_query() {
        let index = SpatialIndex::new(256, 256);
        let entity_id = EntityId(1);
        let position = Vec3::new(0.0, 0.0, 0.0);

        index.add_entity(entity_id, position).await;

        let query = SpatialQuery::Radius {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 5.0,
        };

        let results = index.query(query).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entity_id);
    }
}
