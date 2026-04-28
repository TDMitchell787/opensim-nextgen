use crate::region::RegionId;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Session permissions and access levels
#[derive(Debug, Clone)]
pub struct SessionPermissions {
    pub can_build: bool,
    pub can_script: bool,
    pub can_terraform: bool,
    pub can_teleport: bool,
    pub can_return_objects: bool,
    pub god_level: u32,
    pub access_level: AccessLevel,
}

/// Access level for session
#[derive(Debug, Clone, PartialEq)]
pub enum AccessLevel {
    Guest,    // No building/scripting rights
    Resident, // Normal user rights
    Estate,   // Estate manager rights
    Grid,     // Grid manager rights
    God,      // Administrator rights
}

impl Default for SessionPermissions {
    fn default() -> Self {
        Self {
            can_build: true,
            can_script: true,
            can_terraform: false,
            can_teleport: true,
            can_return_objects: false,
            god_level: 0,
            access_level: AccessLevel::Resident,
        }
    }
}

/// Represents a client session with its associated state and security credentials
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub secure_session_id: Uuid,
    pub agent_id: Uuid,
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_id: Option<Uuid>,
    pub region_id: Option<RegionId>,
    pub current_region: Option<RegionId>,
    pub position: Option<(f32, f32, f32)>,
    pub capabilities: HashMap<String, String>,
    pub last_activity: Instant,
    pub client_addr: SocketAddr,
    pub viewer_info: Option<crate::network::client::ViewerInfo>,
    pub authentication_token: Option<String>,
    pub is_authenticated: bool,
    pub login_time: Instant,
    pub permissions: SessionPermissions,
}

impl Session {
    /// Creates a new session for a user
    pub fn new(user_id: String, addr: SocketAddr) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            secure_session_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            user_id,
            first_name: String::new(),
            last_name: String::new(),
            avatar_id: None,
            region_id: None,
            current_region: None,
            position: None,
            capabilities: HashMap::new(),
            last_activity: Instant::now(),
            client_addr: addr,
            viewer_info: None,
            authentication_token: None,
            is_authenticated: false,
            login_time: Instant::now(),
            permissions: SessionPermissions::default(),
        }
    }

    /// Create a session for a successful login
    pub fn new_authenticated(
        user_id: String,
        agent_id: Uuid,
        first_name: String,
        last_name: String,
        addr: SocketAddr,
        auth_token: String,
        permissions: SessionPermissions,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            secure_session_id: Uuid::new_v4(),
            agent_id,
            user_id,
            first_name,
            last_name,
            avatar_id: Some(agent_id), // Use agent_id as avatar_id for now
            region_id: None,
            current_region: None,
            position: None,
            capabilities: HashMap::new(),
            last_activity: Instant::now(),
            client_addr: addr,
            viewer_info: None,
            authentication_token: Some(auth_token),
            is_authenticated: true,
            login_time: Instant::now(),
            permissions,
        }
    }

    /// Updates the last activity timestamp for the session
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Checks if the session has expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Clones the session data into a new Arc<Session>
    pub fn clone_session(&self) -> Arc<Session> {
        Arc::new(self.clone())
    }
}

/// Manages all active client sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    sessions_by_agent: Arc<RwLock<HashMap<Uuid, String>>>, // Map agent_id to session_id for quick lookup
    timeout: Duration,
}

impl SessionManager {
    /// Creates a new session manager
    pub fn new(timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            sessions_by_agent: Arc::new(RwLock::new(HashMap::new())),
            timeout,
        }
    }

    /// Creates a new session and adds it to the manager
    pub async fn create_session(&self, session: Session) -> Result<String, String> {
        let session_id = session.session_id.clone();
        let agent_id = session.agent_id;

        // Check if agent already has an active session
        if self.has_active_session(&agent_id.to_string()).await {
            return Err("Agent already has an active session".to_string());
        }

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            let mut agent_map = self.sessions_by_agent.write().await;

            sessions.insert(session_id.clone(), session);
            agent_map.insert(agent_id, session_id.clone());
        }

        info!(
            "Created new session: {} for agent: {}",
            session_id, agent_id
        );
        Ok(session_id)
    }

    /// Creates a new session with agent ID and adds it to the manager
    pub fn create_session_with_agent(
        &self,
        agent_id: Uuid,
        addr: SocketAddr,
    ) -> Arc<RwLock<Session>> {
        let mut session = Session::new("unknown".to_string(), addr);
        session.agent_id = agent_id;
        Arc::new(RwLock::new(session))
    }

    /// Check if user/agent has an active session
    pub async fn has_active_session(&self, user_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .any(|s| s.user_id == user_id || s.agent_id.to_string() == user_id)
    }

    /// Retrieves a session by its ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Retrieves a session by agent ID
    pub async fn get_session_by_agent(&self, agent_id: &Uuid) -> Option<Session> {
        let agent_map = self.sessions_by_agent.read().await;
        if let Some(session_id) = agent_map.get(agent_id) {
            self.get_session(session_id).await
        } else {
            None
        }
    }

    /// Updates a session's state
    pub async fn update_session(&self, session: Session) {
        let mut sessions = self.sessions.write().await;
        if let Some(s) = sessions.get_mut(&session.session_id) {
            *s = session;
            debug!("Updated session: {}", s.session_id);
        }
    }

    /// Removes a session from the manager
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        let mut agent_map = self.sessions_by_agent.write().await;

        if let Some(session) = sessions.remove(session_id) {
            agent_map.remove(&session.agent_id);
            info!(
                "Removed session: {} for agent: {}",
                session_id, session.agent_id
            );
        }
    }

    /// Periodically cleans up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let mut agent_map = self.sessions_by_agent.write().await;
        let timeout = self.timeout;

        let mut expired_sessions = Vec::new();

        // Find expired sessions first
        for (session_id, session) in sessions.iter() {
            if session.is_expired(timeout) {
                info!("Session {} expired", session.session_id);
                expired_sessions.push((session_id.clone(), session.agent_id));
            }
        }

        // Remove expired sessions and their agent mappings
        for (session_id, agent_id) in expired_sessions {
            sessions.remove(&session_id);
            agent_map.remove(&agent_id);
        }
    }

    /// Retrieves all active sessions
    pub async fn get_all_sessions(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Get agents (sessions) in a specific region
    pub async fn get_region_agents(&self, region_id: Uuid) -> Result<Vec<Session>, String> {
        let sessions = self.sessions.read().await;
        let region_sessions: Vec<Session> = sessions
            .values()
            .filter(|session| {
                session.region_id == Some(crate::region::RegionId(region_id.as_u128() as u64))
            })
            .cloned()
            .collect();
        Ok(region_sessions)
    }

    /// Get all agents (alias for get_all_sessions)
    pub async fn get_all_agents(&self) -> Vec<Session> {
        self.get_all_sessions().await
    }

    /// Teleport an agent to a new region and position
    pub async fn teleport_agent(
        &self,
        agent_id: Uuid,
        region_id: Uuid,
        position: (f32, f32, f32),
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let mut session_by_agent = self.sessions_by_agent.write().await;

        if let Some(session_id) = session_by_agent.get(&agent_id) {
            if let Some(session) = sessions.get_mut(session_id) {
                session.region_id = Some(crate::region::RegionId(region_id.as_u128() as u64));
                session.current_region = Some(crate::region::RegionId(region_id.as_u128() as u64));
                // Update position would require additional fields in Session struct
                debug!(
                    "Teleported agent {} to region {} at position {:?}",
                    agent_id, region_id, position
                );
                return Ok(());
            }
        }

        Err(format!("Agent {} not found", agent_id))
    }

    /// Broadcast a message to all agents in a specific region
    pub async fn broadcast_to_region(
        &self,
        region_id: Uuid,
        message: &str,
    ) -> Result<usize, String> {
        let sessions = self.sessions.read().await;
        let mut message_count = 0;

        for session in sessions.values() {
            if session.region_id == Some(crate::region::RegionId(region_id.as_u128() as u64)) {
                // In production, this would send the message to the client
                // For now, just log it
                debug!(
                    "Broadcasting to agent {} in region {}: {}",
                    session.agent_id, region_id, message
                );
                message_count += 1;
            }
        }

        info!(
            "Broadcasted message to {} agents in region {}",
            message_count, region_id
        );
        Ok(message_count)
    }

    /// Broadcast a message to all agents across all regions
    pub async fn broadcast_to_all(&self, message: &str) -> Result<usize, String> {
        let sessions = self.sessions.read().await;
        let message_count = sessions.len();

        for session in sessions.values() {
            // In production, this would send the message to the client
            // For now, just log it
            debug!("Broadcasting to agent {}: {}", session.agent_id, message);
        }

        info!(
            "Broadcasted message to {} agents across all regions",
            message_count
        );
        Ok(message_count)
    }

    /// Validate a session token
    pub async fn validate_token(&self, token: &str) -> Result<bool, String> {
        let sessions = self.sessions.read().await;

        // In production, this would validate the token properly
        // For now, check if any session has this token as session_id
        let is_valid = sessions.contains_key(token);

        debug!("Token validation for {}: {}", token, is_valid);
        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_session_creation_and_retrieval() {
        let manager = SessionManager::new(Duration::from_secs(60));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new("test_user".to_string(), addr);
        let session_id = session.session_id.clone();

        manager.create_session(session).await.unwrap();

        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, "test_user");
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let manager = SessionManager::new(Duration::from_millis(10));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new("test_user".to_string(), addr);
        let session_id = session.session_id.clone();

        manager.create_session(session).await.unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;

        manager.cleanup_expired_sessions().await;

        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_none());
    }
}
