use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, anyhow};
use rand::Rng;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct LoginSession {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub secure_session_id: Uuid,
    pub circuit_code: u32,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub sim_ip: String,
    pub sim_port: u16,
    pub region_x: u32,
    pub region_y: u32,
    pub implicit_login_completed: bool,
    pub last_handshake_time: Option<DateTime<Utc>>,
    pub timer_breakthrough_used: bool,
    pub region_handshake_reply_count: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub local_id: u32,
    pub region_size_x: u32,
    pub region_size_y: u32,
}

impl LoginSession {
    pub fn new(
        agent_id: Uuid,
        first_name: String,
        last_name: String,
        sim_ip: String,
        sim_port: u16,
    ) -> Self {
        Self::new_with_region(agent_id, first_name, last_name, sim_ip, sim_port, 256000, 256000)
    }

    pub fn new_with_region(
        agent_id: Uuid,
        first_name: String,
        last_name: String,
        sim_ip: String,
        sim_port: u16,
        region_x: u32,
        region_y: u32,
    ) -> Self {
        let now = Utc::now();
        let mut rng = rand::thread_rng();

        Self {
            agent_id,
            session_id: Uuid::new_v4(),
            secure_session_id: Uuid::new_v4(),
            circuit_code: rng.gen::<u32>(),
            first_name,
            last_name,
            created_at: now,
            last_activity: now,
            sim_ip,
            sim_port,
            region_x,
            region_y,
            implicit_login_completed: false,
            last_handshake_time: None,
            timer_breakthrough_used: false,
            region_handshake_reply_count: 0,
            position: [128.0, 128.0, 25.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            local_id: 1,
            region_size_x: 256,
            region_size_y: 256,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
    
    pub fn is_expired(&self, timeout_minutes: i64) -> bool {
        let timeout = Duration::minutes(timeout_minutes);
        Utc::now() - self.last_activity > timeout
    }
}

#[derive(Debug)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u32, LoginSession>>>, // circuit_code -> session
    agent_sessions: Arc<RwLock<HashMap<Uuid, u32>>>,   // agent_id -> circuit_code
    session_timeout_minutes: i64,
    region_x_meters: u32,
    region_y_meters: u32,
    region_size_x: u32,
    region_size_y: u32,
    next_avatar_local_id: Arc<std::sync::atomic::AtomicU32>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            agent_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout_minutes: 30,
            region_x_meters: 256000,
            region_y_meters: 256000,
            region_size_x: 256,
            region_size_y: 256,
            next_avatar_local_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
        }
    }

    pub fn new_with_region(region_x_meters: u32, region_y_meters: u32) -> Self {
        Self::new_with_region_size(region_x_meters, region_y_meters, 256, 256)
    }

    pub fn new_with_region_size(region_x_meters: u32, region_y_meters: u32, region_size_x: u32, region_size_y: u32) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            agent_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout_minutes: 30,
            region_x_meters,
            region_y_meters,
            region_size_x,
            region_size_y,
            next_avatar_local_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
        }
    }
    
    pub fn create_session(
        &self,
        agent_id: Uuid,
        first_name: String,
        last_name: String,
        sim_ip: String,
        sim_port: u16,
    ) -> Result<LoginSession> {
        debug!("🔍 SESSION TRACE: Starting create_session for agent: {}", agent_id);
        let mut session = LoginSession::new_with_region(
            agent_id, first_name, last_name, sim_ip, sim_port,
            self.region_x_meters, self.region_y_meters,
        );
        session.local_id = self.next_avatar_local_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        session.region_size_x = self.region_size_x;
        session.region_size_y = self.region_size_y;
        info!("🔍 SESSION: Assigned local_id={} to agent {}", session.local_id, agent_id);
        debug!("🔍 SESSION TRACE: LoginSession::new completed");
        
        // Check for existing session and remove it
        debug!("🔍 SESSION TRACE: Checking for existing session");
        let old_circuit_code = {
            let agent_sessions = self.agent_sessions.read();
            agent_sessions.get(&agent_id).copied()
        }; // Read lock is released here
        
        if let Some(old_circuit_code) = old_circuit_code {
            debug!("🔍 SESSION TRACE: Found existing session with circuit code: {}, removing it", old_circuit_code);
            self.remove_session(old_circuit_code)?;
            debug!("🔍 SESSION TRACE: Successfully removed old session");
        } else {
            debug!("🔍 SESSION TRACE: No existing session found");
        }
        
        let circuit_code = session.circuit_code;
        debug!("🔍 SESSION TRACE: About to store session with circuit code: {}", circuit_code);
        
        // Store the session
        {
            debug!("🔍 SESSION TRACE: About to acquire session locks");
            let mut sessions = self.sessions.write();
            debug!("🔍 SESSION TRACE: Acquired sessions write lock");
            let mut agent_sessions = self.agent_sessions.write();
            debug!("🔍 SESSION TRACE: Acquired agent_sessions write lock");
            
            sessions.insert(circuit_code, session.clone());
            debug!("🔍 SESSION TRACE: Inserted session into sessions map");
            agent_sessions.insert(agent_id, circuit_code);
            debug!("🔍 SESSION TRACE: Inserted agent mapping");
        }
        debug!("🔍 SESSION TRACE: Released session locks");
        
        info!(
            "Created new login session for {} {} with circuit code {}",
            session.first_name, session.last_name, circuit_code
        );
        debug!("🔍 SESSION TRACE: create_session completed successfully");
        
        Ok(session)
    }
    
    pub fn register_external_session(
        &self,
        agent_id: Uuid,
        session_id: Uuid,
        secure_session_id: Uuid,
        circuit_code: u32,
        first_name: String,
        last_name: String,
        sim_ip: String,
        sim_port: u16,
    ) -> LoginSession {
        let now = Utc::now();
        let session = LoginSession {
            agent_id,
            session_id,
            secure_session_id,
            circuit_code,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            created_at: now,
            last_activity: now,
            sim_ip,
            sim_port,
            region_x: self.region_x_meters,
            region_y: self.region_y_meters,
            implicit_login_completed: false,
            last_handshake_time: None,
            timer_breakthrough_used: false,
            region_handshake_reply_count: 0,
            position: [128.0, 128.0, 25.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            local_id: 1,
            region_size_x: self.region_size_x,
            region_size_y: self.region_size_y,
        };
        {
            let mut sessions = self.sessions.write();
            let mut agent_sessions = self.agent_sessions.write();
            sessions.insert(circuit_code, session.clone());
            agent_sessions.insert(agent_id, circuit_code);
        }
        info!(
            "Registered external session for {} {} with circuit code {} (HG inbound)",
            first_name, last_name, circuit_code
        );
        session
    }

    pub fn get_session_by_circuit_code(&self, circuit_code: u32) -> Option<LoginSession> {
        self.sessions.read().get(&circuit_code).cloned()
    }
    
    pub fn get_session_by_agent_id(&self, agent_id: Uuid) -> Option<LoginSession> {
        let agent_sessions = self.agent_sessions.read();
        if let Some(circuit_code) = agent_sessions.get(&agent_id) {
            self.sessions.read().get(circuit_code).cloned()
        } else {
            None
        }
    }
    
    pub fn get_all_sessions(&self) -> Vec<LoginSession> {
        let sessions = self.sessions.read();
        sessions.values().cloned().collect()
    }
    
    pub fn update_session_activity(&self, circuit_code: u32) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.update_activity();
            debug!("Updated activity for circuit code {}", circuit_code);
            Ok(())
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }
    
    pub fn remove_session(&self, circuit_code: u32) -> Result<()> {
        debug!("🔍 REMOVE SESSION TRACE: Starting remove_session for circuit code: {}", circuit_code);
        debug!("🔍 REMOVE SESSION TRACE: About to acquire sessions write lock");
        let mut sessions = self.sessions.write();
        debug!("🔍 REMOVE SESSION TRACE: Acquired sessions write lock");
        debug!("🔍 REMOVE SESSION TRACE: About to acquire agent_sessions write lock");
        let mut agent_sessions = self.agent_sessions.write();
        debug!("🔍 REMOVE SESSION TRACE: Acquired agent_sessions write lock");
        
        if let Some(session) = sessions.remove(&circuit_code) {
            debug!("🔍 REMOVE SESSION TRACE: Found session to remove, removing agent mapping");
            agent_sessions.remove(&session.agent_id);
            info!(
                "Removed session for {} {} (circuit code {})",
                session.first_name, session.last_name, circuit_code
            );
            debug!("🔍 REMOVE SESSION TRACE: remove_session completed successfully");
            Ok(())
        } else {
            debug!("🔍 REMOVE SESSION TRACE: Session not found for circuit code: {}", circuit_code);
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }
    
    pub fn validate_session(&self, agent_id: Uuid, session_id: Uuid, circuit_code: u32) -> bool {
        debug!("🔍 VALIDATE SESSION: circuit={}, agent={}, session={}", circuit_code, agent_id, session_id);

        if let Some(session) = self.get_session_by_circuit_code(circuit_code) {
            debug!("🔍 VALIDATE SESSION: Found session - stored_agent={}, stored_session={}", session.agent_id, session.session_id);

            let agent_match = session.agent_id == agent_id;
            let session_match = session.session_id == session_id;
            let not_expired = !session.is_expired(self.session_timeout_minutes);

            debug!("🔍 VALIDATE SESSION: agent_match={}, session_match={}, not_expired={}", agent_match, session_match, not_expired);

            agent_match && session_match && not_expired
        } else {
            warn!("🔍 VALIDATE SESSION: No session found for circuit_code={}", circuit_code);
            false
        }
    }
    
    pub fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write();
        let mut agent_sessions = self.agent_sessions.write();
        
        let expired_circuits: Vec<u32> = sessions
            .iter()
            .filter_map(|(circuit_code, session)| {
                if session.is_expired(self.session_timeout_minutes) {
                    Some(*circuit_code)
                } else {
                    None
                }
            })
            .collect();
        
        let count = expired_circuits.len();
        
        for circuit_code in expired_circuits {
            if let Some(session) = sessions.remove(&circuit_code) {
                agent_sessions.remove(&session.agent_id);
                warn!(
                    "Cleaned up expired session for {} {} (circuit code {})",
                    session.first_name, session.last_name, circuit_code
                );
            }
        }
        
        count
    }

    pub fn update_handshake_time(&self, circuit_code: u32) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.last_handshake_time = Some(Utc::now());
            debug!("Updated handshake time for circuit code {}", circuit_code);
            Ok(())
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }

    pub fn set_login_completed(&self, circuit_code: u32, completed: bool) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.implicit_login_completed = completed;
            debug!("Set login_completed={} for circuit code {}", completed, circuit_code);
            Ok(())
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }

    pub fn increment_handshake_reply_count(&self, circuit_code: u32) -> Result<u32> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.region_handshake_reply_count += 1;
            let count = session.region_handshake_reply_count;
            debug!("Incremented handshake_reply_count to {} for circuit code {}", count, circuit_code);
            Ok(count)
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }

    pub fn set_timer_breakthrough_used(&self, circuit_code: u32, used: bool) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.timer_breakthrough_used = used;
            debug!("Set timer_breakthrough_used={} for circuit code {}", used, circuit_code);
            Ok(())
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }

    pub fn reset_session_state(&self, circuit_code: u32) -> Result<()> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(&circuit_code) {
            session.implicit_login_completed = false;
            session.last_handshake_time = None;
            session.timer_breakthrough_used = false;
            session.region_handshake_reply_count = 0;
            info!("Reset session state for circuit code {}", circuit_code);
            Ok(())
        } else {
            Err(anyhow!("Session not found for circuit code {}", circuit_code))
        }
    }

    pub fn get_active_session_count(&self) -> usize {
        self.sessions.read().len()
    }

    pub fn update_avatar_position(&self, agent_id: Uuid, position: [f32; 3]) -> Result<()> {
        let circuit_code = {
            let agent_sessions = self.agent_sessions.read();
            agent_sessions.get(&agent_id).copied()
        };
        if let Some(circuit_code) = circuit_code {
            let mut sessions = self.sessions.write();
            if let Some(session) = sessions.get_mut(&circuit_code) {
                session.position = position;
                Ok(())
            } else {
                Err(anyhow!("Session not found"))
            }
        } else {
            Err(anyhow!("Agent session not found"))
        }
    }

    pub fn update_avatar_rotation(&self, agent_id: Uuid, rotation: [f32; 4]) -> Result<()> {
        let circuit_code = {
            let agent_sessions = self.agent_sessions.read();
            agent_sessions.get(&agent_id).copied()
        };
        if let Some(circuit_code) = circuit_code {
            let mut sessions = self.sessions.write();
            if let Some(session) = sessions.get_mut(&circuit_code) {
                session.rotation = rotation;
                Ok(())
            } else {
                Err(anyhow!("Session not found"))
            }
        } else {
            Err(anyhow!("Agent session not found"))
        }
    }

    pub fn get_avatar_position(&self, agent_id: Uuid) -> Option<[f32; 3]> {
        self.get_session_by_agent_id(agent_id).map(|s| s.position)
    }

    pub fn get_avatar_local_id(&self, agent_id: Uuid) -> Option<u32> {
        self.get_session_by_agent_id(agent_id).map(|s| s.local_id)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new();
        let agent_id = Uuid::new_v4();
        
        let session = manager.create_session(
            agent_id,
            "John".to_string(),
            "Doe".to_string(),
            "127.0.0.1".to_string(),
            9000,
        ).unwrap();
        
        assert_eq!(session.agent_id, agent_id);
        assert_eq!(session.first_name, "John");
        assert_eq!(session.last_name, "Doe");
        assert_eq!(manager.get_active_session_count(), 1);
        
        // Test retrieval by circuit code
        let retrieved = manager.get_session_by_circuit_code(session.circuit_code).unwrap();
        assert_eq!(retrieved.agent_id, agent_id);
        
        // Test retrieval by agent ID
        let retrieved = manager.get_session_by_agent_id(agent_id).unwrap();
        assert_eq!(retrieved.circuit_code, session.circuit_code);
    }
    
    #[test]
    fn test_session_validation() {
        let manager = SessionManager::new();
        let agent_id = Uuid::new_v4();
        
        let session = manager.create_session(
            agent_id,
            "Test".to_string(),
            "User".to_string(),
            "127.0.0.1".to_string(),
            9000,
        ).unwrap();
        
        assert!(manager.validate_session(agent_id, session.session_id, session.circuit_code));
        assert!(!manager.validate_session(Uuid::new_v4(), session.session_id, session.circuit_code));
        assert!(!manager.validate_session(agent_id, Uuid::new_v4(), session.circuit_code));
        assert!(!manager.validate_session(agent_id, session.session_id, 999999));
    }
    
    #[test]
    fn test_session_removal() {
        let manager = SessionManager::new();
        let agent_id = Uuid::new_v4();
        
        let session = manager.create_session(
            agent_id,
            "Test".to_string(),
            "User".to_string(),
            "127.0.0.1".to_string(),
            9000,
        ).unwrap();
        
        assert_eq!(manager.get_active_session_count(), 1);
        
        manager.remove_session(session.circuit_code).unwrap();
        assert_eq!(manager.get_active_session_count(), 0);
        assert!(manager.get_session_by_circuit_code(session.circuit_code).is_none());
        assert!(manager.get_session_by_agent_id(agent_id).is_none());
    }
}