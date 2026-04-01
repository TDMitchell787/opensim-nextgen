//! Region crossing protocols for seamless avatar movement between regions

use std::{collections::HashMap, sync::Arc};
use anyhow::{anyhow, Result};
use tracing::{info, warn, error};
use uuid::Uuid;
use tokio::sync::RwLock;

use crate::{
    network::{session::Session, llsd::LLSDValue},
    region::{RegionManager, RegionId},
    state::StateManager,
};

/// Represents the state of a region crossing operation
#[derive(Debug, Clone, PartialEq)]
pub enum CrossingState {
    /// Initial state - crossing not started
    Idle,
    /// Crossing initiated, waiting for destination region confirmation
    Initiating,
    /// Destination region confirmed, transferring avatar data
    Transferring,
    /// Avatar successfully transferred to destination
    Completed,
    /// Crossing failed, avatar remains in source region
    Failed(String),
}

/// Information about an ongoing region crossing
#[derive(Debug, Clone)]
pub struct RegionCrossing {
    pub crossing_id: String,
    pub agent_id: Uuid,
    pub source_region: RegionId,
    pub destination_region: RegionId,
    pub destination_position: (f32, f32, f32),
    pub state: CrossingState,
    pub initiated_at: u64,
}

/// Manages region crossing operations
pub struct RegionCrossingManager {
    /// Active region crossings
    active_crossings: RwLock<HashMap<String, RegionCrossing>>,
    /// Region manager for accessing regions
    region_manager: Arc<RegionManager>,
    /// State manager for persisting crossing state
    state_manager: Arc<StateManager>,
}

impl RegionCrossingManager {
    /// Create a new region crossing manager
    pub fn new(
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
    ) -> Self {
        Self {
            active_crossings: RwLock::new(HashMap::new()),
            region_manager,
            state_manager,
        }
    }

    /// Initiate a region crossing for an avatar
    pub async fn initiate_crossing(
        &self,
        session: Arc<RwLock<Session>>,
        destination_region: RegionId,
        destination_position: (f32, f32, f32),
    ) -> Result<String> {
        let session_guard = session.read().await;
        let agent_id = session_guard.agent_id;
        let source_region = session_guard.current_region.unwrap_or(RegionId(1)); // Default region
        drop(session_guard);

        let crossing_id = Uuid::new_v4().to_string();
        info!(
            "Initiating region crossing {} for agent {} from region {:?} to region {:?}",
            crossing_id, agent_id, source_region, destination_region
        );

        // Validate destination region exists
        if self.region_manager.get_region(destination_region).await.is_none() {
            return Err(anyhow!("Destination region {:?} not found", destination_region));
        }

        // Create crossing record
        let crossing = RegionCrossing {
            crossing_id: crossing_id.clone(),
            agent_id,
            source_region,
            destination_region,
            destination_position,
            state: CrossingState::Initiating,
            initiated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store crossing in active crossings
        self.active_crossings.write().await.insert(crossing_id.clone(), crossing.clone());

        // Notify destination region about incoming avatar
        if let Err(e) = self.notify_destination_region(&crossing).await {
            error!("Failed to notify destination region: {}", e);
            self.fail_crossing(&crossing_id, format!("Failed to contact destination region: {}", e)).await?;
            return Err(e);
        }

        Ok(crossing_id)
    }

    /// Complete a region crossing
    pub async fn complete_crossing(
        &self,
        crossing_id: &str,
        session: Arc<RwLock<Session>>,
    ) -> Result<()> {
        let mut crossings = self.active_crossings.write().await;
        
        if let Some(mut crossing) = crossings.remove(crossing_id) {
            info!("Completing region crossing {}", crossing_id);
            
            crossing.state = CrossingState::Completed;
            
            // Update session with new region
            {
                let mut session_guard = session.write().await;
                session_guard.current_region = Some(crossing.destination_region);
                session_guard.position = Some(crossing.destination_position);
            }

            // Notify source region that avatar has left
            if let Err(e) = self.notify_source_region_departure(&crossing).await {
                warn!("Failed to notify source region of departure: {}", e);
            }

            info!(
                "Region crossing {} completed for agent {}",
                crossing_id, crossing.agent_id
            );
            
            Ok(())
        } else {
            Err(anyhow!("Crossing {} not found", crossing_id))
        }
    }

    /// Fail a region crossing
    pub async fn fail_crossing(&self, crossing_id: &str, reason: String) -> Result<()> {
        let mut crossings = self.active_crossings.write().await;
        
        if let Some(crossing) = crossings.get_mut(crossing_id) {
            warn!("Failing region crossing {}: {}", crossing_id, reason);
            crossing.state = CrossingState::Failed(reason);
            
            // Keep the crossing record for a short time for debugging
            // In a production system, you might want to clean this up after some time
            
            Ok(())
        } else {
            Err(anyhow!("Crossing {} not found", crossing_id))
        }
    }

    /// Get the status of a region crossing
    pub async fn get_crossing_status(&self, crossing_id: &str) -> Option<RegionCrossing> {
        self.active_crossings.read().await.get(crossing_id).cloned()
    }

    /// Handle region crossing request from client
    pub async fn handle_crossing_request(
        &self,
        session: Arc<RwLock<Session>>,
        request_data: LLSDValue,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!("Handling region crossing request for session {}", session_id);

        // Parse crossing request
        let map = match request_data {
            LLSDValue::Map(m) => m,
            _ => return Err(anyhow!("Crossing request must be a map")),
        };

        let destination_region = match map.get("destination_region") {
            Some(LLSDValue::Integer(id)) => RegionId(*id as u64),
            _ => return Err(anyhow!("Missing or invalid destination_region")),
        };

        let destination_position = match map.get("destination_position") {
            Some(LLSDValue::Array(arr)) if arr.len() >= 3 => {
                let x = self.extract_float(&arr[0])?;
                let y = self.extract_float(&arr[1])?;
                let z = self.extract_float(&arr[2])?;
                (x, y, z)
            }
            _ => return Err(anyhow!("Missing or invalid destination_position")),
        };

        // Initiate the crossing
        let crossing_id = self.initiate_crossing(session, destination_region, destination_position).await?;

        // Return response
        let mut response = HashMap::new();
        response.insert("success".to_string(), LLSDValue::Boolean(true));
        response.insert("crossing_id".to_string(), LLSDValue::String(crossing_id));
        response.insert("message".to_string(), LLSDValue::String("Region crossing initiated".to_string()));

        Ok(LLSDValue::Map(response))
    }

    /// Notify destination region about incoming avatar
    async fn notify_destination_region(&self, crossing: &RegionCrossing) -> Result<()> {
        info!(
            "Notifying destination region {:?} about incoming avatar {}",
            crossing.destination_region, crossing.agent_id
        );

        // In a distributed system, this would send a message to the destination region server
        // For now, we'll just validate that the destination region exists
        let _destination_region = self.region_manager
            .get_region(crossing.destination_region)
            .await
            .ok_or_else(|| anyhow!("Destination region not found"))?;

        // Update crossing state
        {
            let mut crossings = self.active_crossings.write().await;
            if let Some(crossing_record) = crossings.get_mut(&crossing.crossing_id) {
                crossing_record.state = CrossingState::Transferring;
            }
        }

        info!(
            "Destination region {:?} confirmed for crossing {}",
            crossing.destination_region, crossing.crossing_id
        );

        Ok(())
    }

    /// Notify source region that avatar has departed
    async fn notify_source_region_departure(&self, crossing: &RegionCrossing) -> Result<()> {
        info!(
            "Notifying source region {:?} about avatar {} departure",
            crossing.source_region, crossing.agent_id
        );

        // Remove avatar from source region
        self.region_manager
            .remove_avatar(&crossing.agent_id.to_string())
            .await
            .map_err(|e| anyhow!("Failed to remove avatar from source region: {}", e))?;

        Ok(())
    }

    /// Clean up old completed or failed crossings
    pub async fn cleanup_old_crossings(&self, max_age_seconds: u64) -> Result<usize> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut crossings = self.active_crossings.write().await;
        let initial_count = crossings.len();

        crossings.retain(|_, crossing| {
            let age = current_time.saturating_sub(crossing.initiated_at);
            
            // Keep ongoing crossings
            if matches!(crossing.state, CrossingState::Initiating | CrossingState::Transferring) {
                return true;
            }
            
            // Remove old completed or failed crossings
            age < max_age_seconds
        });

        let cleaned = initial_count - crossings.len();
        if cleaned > 0 {
            info!("Cleaned up {} old region crossings", cleaned);
        }

        Ok(cleaned)
    }

    /// Helper function to extract float from LLSD value
    fn extract_float(&self, value: &LLSDValue) -> Result<f32> {
        match value {
            LLSDValue::Real(f) => Ok(*f as f32),
            LLSDValue::Integer(i) => Ok(*i as f32),
            _ => Err(anyhow!("Expected numeric value")),
        }
    }
}

/// Extension trait to add region crossing functionality to Session
pub trait SessionRegionCrossing {
    /// Current region the session is in
    fn current_region(&self) -> Option<RegionId>;
    /// Current position in the region
    fn position(&self) -> Option<(f32, f32, f32)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ffi::physics::PhysicsBridge,
        network::session::SessionManager,
        region::{RegionConfig, terrain::TerrainConfig},
    };
    use std::time::Duration;

    #[tokio::test]
    async fn test_region_crossing_initiation() -> Result<()> {
        // Setup test environment
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        
        // Create test regions
        let source_region = RegionId(1);
        let dest_region = RegionId(2);
        
        let config = RegionConfig {
            name: "Test Region".to_string(),
            size: (256, 256),
            location: (1000, 1000),
            terrain: TerrainConfig::default(),
            physics: crate::ffi::physics::PhysicsConfig::default(),
            max_entities: 1000,
        };
        
        region_manager.create_region(source_region, config.clone()).await?;
        region_manager.create_region(dest_region, config).await?;
        
        // Create crossing manager
        let crossing_manager = RegionCrossingManager::new(region_manager, state_manager);
        
        // Create test session
        let session_manager = Arc::new(SessionManager::new(Duration::from_secs(300)));
        let agent_id = Uuid::new_v4();
        let session = session_manager.create_session_with_agent(agent_id, "127.0.0.1:12345".parse().unwrap());
        
        // Test crossing initiation
        let crossing_id = crossing_manager
            .initiate_crossing(session, dest_region, (128.0, 128.0, 21.0))
            .await?;
        
        // Verify crossing was created
        let crossing = crossing_manager.get_crossing_status(&crossing_id).await;
        assert!(crossing.is_some());
        
        let crossing = crossing.unwrap();
        assert_eq!(crossing.agent_id, agent_id);
        assert_eq!(crossing.source_region, source_region);
        assert_eq!(crossing.destination_region, dest_region);
        
        Ok(())
    }
}