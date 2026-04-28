//! Handles inventory-related messages

use crate::network::llsd::LLSDValue;
use crate::{
    network::{llsd::LLSDMessage, session::Session},
    state::StateManager,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Handles inventory-related messages
#[derive(Default)]
pub struct InventoryHandler;

impl InventoryHandler {
    /// Placeholder for handling inventory requests
    pub async fn handle_fetch_inventory(
        &self,
        session: Arc<RwLock<Session>>,
        state_manager: Arc<StateManager>,
    ) -> Result<Option<LLSDMessage>> {
        let session_guard = session.read().await;
        info!(
            "Handling fetch inventory for session: {:?}",
            session_guard.session_id
        );
        let mut inventory_manager = state_manager.inventory_manager.write().await;
        let inventory = inventory_manager.get_or_create_inventory(&session_guard.agent_id);

        let response_data = LLSDValue::from(inventory.clone());

        let response_message = LLSDMessage {
            message_type: "FetchInventoryReply".to_string(),
            data: response_data,
            session_id: Uuid::parse_str(&session_guard.session_id).ok(),
            sequence: None, // Or handle sequence numbers if needed
        };

        Ok(Some(response_message))
    }
}
