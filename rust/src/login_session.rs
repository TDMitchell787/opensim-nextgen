use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared circuit code storage for login authentication
#[derive(Clone)]
pub struct CircuitCodeRegistry {
    codes: Arc<RwLock<HashMap<u32, LoginSession>>>,
    session_manager: Option<Arc<crate::session::manager::SessionManager>>,
}

/// Login session information
#[derive(Clone, Debug)]
pub struct LoginSession {
    pub circuit_code: u32,
    pub session_id: String,
    pub agent_id: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: std::time::Instant,
    pub is_xmlrpc_session: bool, // True for sessions from XMLRPC login, false for temporary UDP sessions
}

impl CircuitCodeRegistry {
    pub fn new() -> Self {
        Self {
            codes: Arc::new(RwLock::new(HashMap::new())),
            session_manager: None,
        }
    }

    pub fn with_session_manager(
        session_manager: Arc<crate::session::manager::SessionManager>,
    ) -> Self {
        Self {
            codes: Arc::new(RwLock::new(HashMap::new())),
            session_manager: Some(session_manager),
        }
    }

    pub async fn register_login(&self, session: LoginSession) {
        self.codes
            .write()
            .await
            .insert(session.circuit_code, session);
    }

    pub async fn validate_circuit_code(&self, circuit_code: u32) -> Option<LoginSession> {
        // Clean up expired sessions (older than 5 minutes)
        let mut codes = self.codes.write().await;
        let now = std::time::Instant::now();
        codes.retain(|_, session| now.duration_since(session.created_at).as_secs() < 300); // 5 minute timeout for login sessions

        codes.get(&circuit_code).cloned()
    }

    pub async fn remove_circuit_code(&self, circuit_code: u32) -> Option<LoginSession> {
        self.codes.write().await.remove(&circuit_code)
    }

    pub async fn get_most_recent_session(&self) -> Option<LoginSession> {
        let codes = self.codes.read().await;
        let now = std::time::Instant::now();

        // First, try to find XMLRPC sessions in our registry
        if let Some(xmlrpc_session) = codes
            .values()
            .filter(|session| {
                now.duration_since(session.created_at).as_secs() < 60 && session.is_xmlrpc_session
            })
            .max_by_key(|session| session.created_at)
            .cloned()
        {
            return Some(xmlrpc_session);
        }

        // If we have access to SessionManager, check for recent sessions there
        if let Some(ref session_manager) = self.session_manager {
            // Get all sessions from SessionManager and find the most recent one
            let sessions = session_manager.get_all_sessions();
            if let Some(recent_session) = sessions
                .into_iter()
                .max_by_key(|session| session.created_at)
            {
                // Convert SessionManager::LoginSession to CircuitCodeRegistry::LoginSession
                return Some(LoginSession {
                    circuit_code: recent_session.circuit_code,
                    session_id: recent_session.session_id.to_string(),
                    agent_id: recent_session.agent_id.to_string(),
                    first_name: recent_session.first_name,
                    last_name: recent_session.last_name,
                    created_at: std::time::Instant::now(),
                    is_xmlrpc_session: true, // SessionManager sessions are from XMLRPC
                });
            }
        }

        // Fall back to any recent session in our registry (temporary UDP sessions)
        codes
            .values()
            .filter(|session| now.duration_since(session.created_at).as_secs() < 60)
            .max_by_key(|session| session.created_at)
            .cloned()
    }
}
