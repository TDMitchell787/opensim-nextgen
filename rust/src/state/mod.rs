//! Manages the persistent state of the simulator.

pub mod inventory;

use self::inventory::InventoryManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages all persistent state for the simulator.
#[derive(Clone)]
pub struct StateManager {
    pub inventory_manager: Arc<RwLock<InventoryManager>>,
}

impl StateManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inventory_manager: Arc::new(RwLock::new(InventoryManager::new())),
        })
    }
}
