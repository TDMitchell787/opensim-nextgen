use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LoginStage {
    XmlRpcSent = 1,
    CircuitCodeReceived = 2,
    CompleteAgentMovementReceived = 3,
    RegionHandshakeSent = 4,
    RegionHandshakeAcked = 5,
    TerrainSent = 6,
    TerrainAcked = 7,
    AvatarUpdateSent = 8,
    MovementCompleteSent = 9,
    AvatarFactoryValidated = 10,
    EventQueueActive = 11,
    AgentUpdateStream = 12,
}

impl LoginStage {
    pub fn name(&self) -> &'static str {
        match self {
            LoginStage::XmlRpcSent => "LOGIN_XMLRPC_SENT",
            LoginStage::CircuitCodeReceived => "CIRCUIT_CODE_RECEIVED",
            LoginStage::CompleteAgentMovementReceived => "CAM_RECEIVED",
            LoginStage::RegionHandshakeSent => "REGION_HANDSHAKE_SENT",
            LoginStage::RegionHandshakeAcked => "REGION_HANDSHAKE_ACK",
            LoginStage::TerrainSent => "TERRAIN_SENT",
            LoginStage::TerrainAcked => "TERRAIN_ACKED",
            LoginStage::AvatarUpdateSent => "AVATAR_UPDATE_SENT",
            LoginStage::MovementCompleteSent => "MOVEMENT_COMPLETE_SENT",
            LoginStage::AvatarFactoryValidated => "AVATAR_FACTORY_VALIDATED",
            LoginStage::EventQueueActive => "EVENTQUEUE_ACTIVE",
            LoginStage::AgentUpdateStream => "AGENT_UPDATE_STREAM",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LoginStage::XmlRpcSent => "Login response sent to viewer",
            LoginStage::CircuitCodeReceived => "UseCircuitCode from viewer",
            LoginStage::CompleteAgentMovementReceived => "CompleteAgentMovement received",
            LoginStage::RegionHandshakeSent => "RegionHandshake sent",
            LoginStage::RegionHandshakeAcked => "RegionHandshakeReply received",
            LoginStage::TerrainSent => "All 64 LayerData packets sent",
            LoginStage::TerrainAcked => "Viewer acknowledged terrain",
            LoginStage::AvatarUpdateSent => "ObjectUpdate (self avatar) sent",
            LoginStage::MovementCompleteSent => "AgentMovementComplete sent",
            LoginStage::AvatarFactoryValidated => "Baked texture cache validated",
            LoginStage::EventQueueActive => "First EventQueue response",
            LoginStage::AgentUpdateStream => "Continuous AgentUpdate (20/sec)",
        }
    }

    pub fn number(&self) -> u8 {
        *self as u8
    }

    pub fn all_stages() -> Vec<LoginStage> {
        vec![
            LoginStage::XmlRpcSent,
            LoginStage::CircuitCodeReceived,
            LoginStage::CompleteAgentMovementReceived,
            LoginStage::RegionHandshakeSent,
            LoginStage::RegionHandshakeAcked,
            LoginStage::TerrainSent,
            LoginStage::TerrainAcked,
            LoginStage::AvatarUpdateSent,
            LoginStage::MovementCompleteSent,
            LoginStage::AvatarFactoryValidated,
            LoginStage::EventQueueActive,
            LoginStage::AgentUpdateStream,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct StageStatus {
    pub passed: bool,
    pub timestamp: Option<Instant>,
    pub details: Option<String>,
    pub packet_count: Option<u32>,
}

impl Default for StageStatus {
    fn default() -> Self {
        Self {
            passed: false,
            timestamp: None,
            details: None,
            packet_count: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionStageTracker {
    pub session_id: String,
    pub circuit_code: u32,
    pub stages: HashMap<LoginStage, StageStatus>,
    pub created_at: Instant,
    pub agent_update_count: u32,
}

impl SessionStageTracker {
    pub fn new(session_id: String, circuit_code: u32) -> Self {
        let mut stages = HashMap::new();
        for stage in LoginStage::all_stages() {
            stages.insert(stage, StageStatus::default());
        }
        Self {
            session_id,
            circuit_code,
            stages,
            created_at: Instant::now(),
            agent_update_count: 0,
        }
    }

    pub fn mark_passed(&mut self, stage: LoginStage, details: Option<String>) {
        let status = self.stages.entry(stage).or_default();
        if !status.passed {
            status.passed = true;
            status.timestamp = Some(Instant::now());
            status.details = details.clone();

            let elapsed = self.created_at.elapsed().as_millis();
            info!(
                "[LOGIN_STAGE] Stage {}: {} - PASSED (+{}ms) {}",
                stage.number(),
                stage.name(),
                elapsed,
                details.unwrap_or_default()
            );
        }
    }

    pub fn mark_passed_with_count(
        &mut self,
        stage: LoginStage,
        count: u32,
        details: Option<String>,
    ) {
        let status = self.stages.entry(stage).or_default();
        if !status.passed {
            status.passed = true;
            status.timestamp = Some(Instant::now());
            status.packet_count = Some(count);
            status.details = details.clone();

            let elapsed = self.created_at.elapsed().as_millis();
            info!(
                "[LOGIN_STAGE] Stage {}: {} - PASSED (+{}ms) [count={}] {}",
                stage.number(),
                stage.name(),
                elapsed,
                count,
                details.unwrap_or_default()
            );
        }
    }

    pub fn mark_failed(&mut self, stage: LoginStage, reason: &str) {
        let status = self.stages.entry(stage).or_default();
        status.passed = false;
        status.timestamp = Some(Instant::now());
        status.details = Some(reason.to_string());

        let elapsed = self.created_at.elapsed().as_millis();
        error!(
            "[LOGIN_STAGE] Stage {}: {} - FAILED (+{}ms) reason: {}",
            stage.number(),
            stage.name(),
            elapsed,
            reason
        );
    }

    pub fn is_passed(&self, stage: LoginStage) -> bool {
        self.stages.get(&stage).map(|s| s.passed).unwrap_or(false)
    }

    pub fn increment_agent_update(&mut self) {
        self.agent_update_count += 1;

        if self.agent_update_count == 1 {
            info!("[LOGIN_STAGE] First AgentUpdate received (count=1)");
        } else if self.agent_update_count == 20 {
            self.mark_passed(
                LoginStage::AgentUpdateStream,
                Some(format!(
                    "Received {} AgentUpdates - stream confirmed",
                    self.agent_update_count
                )),
            );
        } else if self.agent_update_count % 100 == 0 {
            info!(
                "[LOGIN_STAGE] AgentUpdate stream healthy (count={})",
                self.agent_update_count
            );
        }
    }

    pub fn get_summary(&self) -> String {
        let mut summary = format!(
            "\n========== LOGIN STAGE SUMMARY (session: {}, circuit: {}) ==========\n",
            self.session_id, self.circuit_code
        );

        for stage in LoginStage::all_stages() {
            let status = self.stages.get(&stage).unwrap();
            let status_str = if status.passed { "PASS" } else { "----" };
            let time_str = status
                .timestamp
                .map(|t| format!("+{}ms", t.duration_since(self.created_at).as_millis()))
                .unwrap_or_else(|| "     ".to_string());

            summary.push_str(&format!(
                "  Stage {:2}: {} [{}] {} - {}\n",
                stage.number(),
                status_str,
                time_str,
                stage.name(),
                stage.description()
            ));
        }

        summary.push_str(&format!(
            "  AgentUpdate count: {}\n",
            self.agent_update_count
        ));
        summary.push_str(
            "==========================================================================\n",
        );
        summary
    }

    pub fn log_summary(&self) {
        let summary = self.get_summary();
        for line in summary.lines() {
            info!("{}", line);
        }
    }
}

#[derive(Debug)]
pub struct LoginStageTracker {
    sessions: Arc<RwLock<HashMap<String, SessionStageTracker>>>,
    circuits: Arc<RwLock<HashMap<u32, String>>>,
}

impl LoginStageTracker {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            circuits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, session_id: &str, circuit_code: u32) {
        let mut sessions = self.sessions.write().await;
        let mut circuits = self.circuits.write().await;

        let tracker = SessionStageTracker::new(session_id.to_string(), circuit_code);
        sessions.insert(session_id.to_string(), tracker);
        circuits.insert(circuit_code, session_id.to_string());

        info!(
            "[LOGIN_STAGE] Created tracker for session {} (circuit {})",
            session_id, circuit_code
        );
    }

    pub async fn mark_passed(&self, session_id: &str, stage: LoginStage, details: Option<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(tracker) = sessions.get_mut(session_id) {
            tracker.mark_passed(stage, details);
        } else {
            warn!(
                "[LOGIN_STAGE] No tracker found for session {} when marking {} passed",
                session_id,
                stage.name()
            );
        }
    }

    pub async fn mark_passed_by_circuit(
        &self,
        circuit_code: u32,
        stage: LoginStage,
        details: Option<String>,
    ) {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            self.mark_passed(&session_id, stage, details).await;
        } else {
            warn!(
                "[LOGIN_STAGE] No tracker found for circuit {} when marking {} passed",
                circuit_code,
                stage.name()
            );
        }
    }

    pub async fn mark_passed_with_count(
        &self,
        session_id: &str,
        stage: LoginStage,
        count: u32,
        details: Option<String>,
    ) {
        let mut sessions = self.sessions.write().await;
        if let Some(tracker) = sessions.get_mut(session_id) {
            tracker.mark_passed_with_count(stage, count, details);
        }
    }

    pub async fn mark_passed_with_count_by_circuit(
        &self,
        circuit_code: u32,
        stage: LoginStage,
        count: u32,
        details: Option<String>,
    ) {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            self.mark_passed_with_count(&session_id, stage, count, details)
                .await;
        }
    }

    pub async fn mark_failed(&self, session_id: &str, stage: LoginStage, reason: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(tracker) = sessions.get_mut(session_id) {
            tracker.mark_failed(stage, reason);
        }
    }

    pub async fn mark_failed_by_circuit(&self, circuit_code: u32, stage: LoginStage, reason: &str) {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            self.mark_failed(&session_id, stage, reason).await;
        }
    }

    pub async fn increment_agent_update(&self, circuit_code: u32) {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);

            let mut sessions = self.sessions.write().await;
            if let Some(tracker) = sessions.get_mut(&session_id) {
                tracker.increment_agent_update();
            }
        }
    }

    pub async fn is_passed(&self, session_id: &str, stage: LoginStage) -> bool {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|t| t.is_passed(stage))
            .unwrap_or(false)
    }

    pub async fn is_passed_by_circuit(&self, circuit_code: u32, stage: LoginStage) -> bool {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            return self.is_passed(&session_id, stage).await;
        }
        false
    }

    pub async fn get_summary(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|t| t.get_summary())
    }

    pub async fn get_summary_by_circuit(&self, circuit_code: u32) -> Option<String> {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            return self.get_summary(&session_id).await;
        }
        None
    }

    pub async fn log_summary(&self, session_id: &str) {
        let sessions = self.sessions.read().await;
        if let Some(tracker) = sessions.get(session_id) {
            tracker.log_summary();
        }
    }

    pub async fn log_summary_by_circuit(&self, circuit_code: u32) {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);
            self.log_summary(&session_id).await;
        }
    }

    pub async fn get_agent_update_count(&self, circuit_code: u32) -> u32 {
        let circuits = self.circuits.read().await;
        if let Some(session_id) = circuits.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuits);

            let sessions = self.sessions.read().await;
            return sessions
                .get(&session_id)
                .map(|t| t.agent_update_count)
                .unwrap_or(0);
        }
        0
    }

    pub async fn cleanup_old_sessions(&self, max_age_secs: u64) {
        let mut sessions = self.sessions.write().await;
        let mut circuits = self.circuits.write().await;

        let now = Instant::now();
        let old_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, t)| now.duration_since(t.created_at).as_secs() > max_age_secs)
            .map(|(k, _)| k.clone())
            .collect();

        for session_id in old_sessions {
            if let Some(tracker) = sessions.remove(&session_id) {
                circuits.remove(&tracker.circuit_code);
                info!(
                    "[LOGIN_STAGE] Cleaned up old session {} (circuit {})",
                    session_id, tracker.circuit_code
                );
            }
        }
    }
}

impl Default for LoginStageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LoginStageTracker {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            circuits: Arc::clone(&self.circuits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stage_tracker() {
        let tracker = LoginStageTracker::new();

        tracker.create_session("test-session", 12345).await;

        tracker
            .mark_passed("test-session", LoginStage::XmlRpcSent, None)
            .await;
        assert!(
            tracker
                .is_passed("test-session", LoginStage::XmlRpcSent)
                .await
        );
        assert!(
            !tracker
                .is_passed("test-session", LoginStage::CircuitCodeReceived)
                .await
        );

        tracker
            .mark_passed_by_circuit(
                12345,
                LoginStage::CircuitCodeReceived,
                Some("test".to_string()),
            )
            .await;
        assert!(
            tracker
                .is_passed_by_circuit(12345, LoginStage::CircuitCodeReceived)
                .await
        );

        let summary = tracker.get_summary("test-session").await;
        assert!(summary.is_some());
    }
}
