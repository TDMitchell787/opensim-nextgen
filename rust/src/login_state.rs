use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, error};

/// Login state machine matching Cool Viewer's systematic approach
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoginState {
    /// Initial startup state
    BrowserInit,
    /// Show login screen
    LoginShow,
    /// Wait for user input at login screen
    LoginWait,
    /// Start login authentication process
    LoginAuthInit,
    /// XML-RPC login request in progress
    XmlrpcLogin,
    /// Processing authentication response
    LoginProcessResponse,
    /// Initialize world connection
    WorldInit,
    /// Wait for seed capability grant
    SeedCapGranted,
    /// Send agent connection request
    AgentSend,
    /// Wait for agent connection acknowledgment
    AgentWait,
    /// Send inventory request
    InventorySend,
    /// Login completed successfully
    Started,
    /// Login failed
    Failed(LoginError),
}

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoginError {
    #[error("XML-RPC authentication failed: {0}")]
    XmlrpcAuthFailed(String),
    #[error("Circuit code validation failed: {0}")]
    CircuitCodeFailed(String),
    #[error("Agent connection timeout")]
    AgentConnectionTimeout,
    #[error("State transition timeout in state: {0}")]
    StateTimeout(String),
    #[error("Network connection failed: {0}")]
    NetworkFailed(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Server maintenance mode")]
    ServerMaintenance,
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
}

/// Login session state machine
#[derive(Debug, Clone)]
pub struct LoginStateMachine {
    /// Current login state
    current_state: LoginState,
    /// Session identifier
    session_id: Uuid,
    /// Agent identifier
    agent_id: Option<Uuid>,
    /// Circuit code for this session
    circuit_code: Option<u32>,
    /// State entry timestamp
    state_entered: Instant,
    /// State timeout duration
    state_timeout: Duration,
    /// Retry count for current operation
    retry_count: u32,
    /// Maximum retries allowed
    max_retries: u32,
    /// State transition history for debugging
    state_history: Vec<(LoginState, Instant)>,
}

impl LoginStateMachine {
    /// Create new login state machine
    pub fn new(session_id: Uuid) -> Self {
        let now = Instant::now();
        let initial_state = LoginState::BrowserInit;
        
        Self {
            current_state: initial_state.clone(),
            session_id,
            agent_id: None,
            circuit_code: None,
            state_entered: now,
            state_timeout: Duration::from_secs(30), // Default 30 second timeout
            retry_count: 0,
            max_retries: 3,
            state_history: vec![(initial_state, now)],
        }
    }

    /// Get current state
    pub fn current_state(&self) -> &LoginState {
        &self.current_state
    }

    /// Get session ID
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Get agent ID if set
    pub fn agent_id(&self) -> Option<Uuid> {
        self.agent_id
    }

    /// Get circuit code if set
    pub fn circuit_code(&self) -> Option<u32> {
        self.circuit_code
    }

    /// Set agent ID
    pub fn set_agent_id(&mut self, agent_id: Uuid) {
        self.agent_id = Some(agent_id);
    }

    /// Set circuit code
    pub fn set_circuit_code(&mut self, circuit_code: u32) {
        self.circuit_code = Some(circuit_code);
    }

    /// Check if state has timed out
    pub fn is_timed_out(&self) -> bool {
        self.state_entered.elapsed() > self.state_timeout
    }

    /// Check if login is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.current_state, LoginState::Started)
    }

    /// Check if login has failed
    pub fn is_failed(&self) -> bool {
        matches!(self.current_state, LoginState::Failed(_))
    }

    /// Get retry count
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Check if max retries exceeded
    pub fn max_retries_exceeded(&self) -> bool {
        self.retry_count >= self.max_retries
    }

    /// Transition to next state
    pub fn transition_to(&mut self, next_state: LoginState) -> Result<(), LoginError> {
        // Validate state transition
        if !self.is_valid_transition(&next_state) {
            return Err(LoginError::StateTimeout(format!(
                "Invalid transition from {:?} to {:?}",
                self.current_state, next_state
            )));
        }

        // Record state transition
        let now = Instant::now();
        self.state_history.push((next_state.clone(), now));

        // Update state
        self.current_state = next_state;
        self.state_entered = now;
        self.retry_count = 0; // Reset retry count on successful transition

        // Set appropriate timeout for new state
        self.state_timeout = self.get_state_timeout(&self.current_state);

        info!(
            "Session {} transitioned to state: {:?}",
            self.session_id,
            self.current_state
        );

        Ok(())
    }

    /// Retry current state operation
    pub fn retry(&mut self) -> Result<(), LoginError> {
        if self.retry_count >= self.max_retries {
            let error = LoginError::MaxRetriesExceeded;
            self.transition_to(LoginState::Failed(error.clone()))?;
            return Err(error);
        }

        self.retry_count += 1;
        self.state_entered = Instant::now();

        warn!(
            "Session {} retrying state {:?}, attempt {}/{}",
            self.session_id,
            self.current_state,
            self.retry_count,
            self.max_retries
        );

        Ok(())
    }

    /// Handle state timeout
    pub fn handle_timeout(&mut self) -> Result<(), LoginError> {
        let error = LoginError::StateTimeout(format!("{:?}", self.current_state));
        
        // Try to retry if not at max retries
        if self.retry_count < self.max_retries {
            self.retry()
        } else {
            self.transition_to(LoginState::Failed(error.clone()))?;
            Err(error)
        }
    }

    /// Get appropriate timeout for state
    fn get_state_timeout(&self, state: &LoginState) -> Duration {
        match state {
            LoginState::BrowserInit => Duration::from_secs(10),
            LoginState::LoginShow => Duration::from_secs(300), // 5 minutes for user input
            LoginState::LoginWait => Duration::from_secs(300),
            LoginState::LoginAuthInit => Duration::from_secs(30),
            LoginState::XmlrpcLogin => Duration::from_secs(60),
            LoginState::LoginProcessResponse => Duration::from_secs(30),
            LoginState::WorldInit => Duration::from_secs(45),
            LoginState::SeedCapGranted => Duration::from_secs(30),
            LoginState::AgentSend => Duration::from_secs(60),
            LoginState::AgentWait => Duration::from_secs(60),
            LoginState::InventorySend => Duration::from_secs(45),
            LoginState::Started => Duration::from_secs(u64::MAX), // No timeout when started
            LoginState::Failed(_) => Duration::from_secs(u64::MAX), // No timeout when failed
        }
    }

    /// Check if transition is valid
    fn is_valid_transition(&self, next_state: &LoginState) -> bool {
        use LoginState::*;

        match (&self.current_state, next_state) {
            // Forward transitions
            (BrowserInit, LoginShow) => true,
            (LoginShow, LoginWait) => true,
            (LoginWait, LoginAuthInit) => true,
            (LoginAuthInit, XmlrpcLogin) => true,
            (XmlrpcLogin, LoginProcessResponse) => true,
            (LoginProcessResponse, WorldInit) => true,
            (WorldInit, SeedCapGranted) => true,
            (SeedCapGranted, AgentSend) => true,
            (AgentSend, AgentWait) => true,
            (AgentWait, InventorySend) => true,
            (InventorySend, Started) => true,

            // Failure transitions from any state
            (_, Failed(_)) => true,

            // Retry transitions (same state)
            (state1, state2) if state1 == state2 => true,

            // Backward transitions for retry scenarios
            (LoginProcessResponse, XmlrpcLogin) => true,
            (AgentWait, AgentSend) => true,
            (WorldInit, LoginAuthInit) => true,

            // Invalid transitions
            _ => false,
        }
    }

    /// Get state history for debugging
    pub fn state_history(&self) -> &[(LoginState, Instant)] {
        &self.state_history
    }

    /// Get time spent in current state
    pub fn time_in_current_state(&self) -> Duration {
        self.state_entered.elapsed()
    }

    /// Get total login time
    pub fn total_login_time(&self) -> Duration {
        if let Some((_, first_time)) = self.state_history.first() {
            Instant::now().duration_since(*first_time)
        } else {
            Duration::from_secs(0)
        }
    }
}

/// Login state machine manager for multiple sessions
#[derive(Debug, Default)]
pub struct LoginStateManager {
    sessions: std::collections::HashMap<Uuid, LoginStateMachine>,
}

impl LoginStateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Create new login session
    pub fn create_session(&mut self, session_id: Uuid) -> &mut LoginStateMachine {
        let state_machine = LoginStateMachine::new(session_id);
        self.sessions.insert(session_id, state_machine);
        self.sessions.get_mut(&session_id).unwrap()
    }

    /// Get session state machine
    pub fn get_session(&self, session_id: &Uuid) -> Option<&LoginStateMachine> {
        self.sessions.get(session_id)
    }

    /// Get mutable session state machine
    pub fn get_session_mut(&mut self, session_id: &Uuid) -> Option<&mut LoginStateMachine> {
        self.sessions.get_mut(session_id)
    }

    /// Remove completed or failed sessions
    pub fn cleanup_session(&mut self, session_id: &Uuid) {
        self.sessions.remove(session_id);
    }

    /// Clean up timed out sessions
    pub fn cleanup_timed_out_sessions(&mut self) {
        let timed_out_sessions: Vec<Uuid> = self
            .sessions
            .iter()
            .filter(|(_, state_machine)| state_machine.is_timed_out())
            .map(|(session_id, _)| *session_id)
            .collect();

        for session_id in timed_out_sessions {
            warn!("Cleaning up timed out session: {}", session_id);
            self.sessions.remove(&session_id);
        }
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> impl Iterator<Item = (&Uuid, &LoginStateMachine)> {
        self.sessions.iter()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_state_machine_creation() {
        let session_id = Uuid::new_v4();
        let state_machine = LoginStateMachine::new(session_id);
        
        assert_eq!(state_machine.session_id(), session_id);
        assert_eq!(state_machine.current_state(), &LoginState::BrowserInit);
        assert!(!state_machine.is_complete());
        assert!(!state_machine.is_failed());
    }

    #[test]
    fn test_state_transitions() {
        let session_id = Uuid::new_v4();
        let mut state_machine = LoginStateMachine::new(session_id);
        
        // Test valid transition
        assert!(state_machine.transition_to(LoginState::LoginShow).is_ok());
        assert_eq!(state_machine.current_state(), &LoginState::LoginShow);
        
        // Test invalid transition
        assert!(state_machine.transition_to(LoginState::Started).is_err());
    }

    #[test]
    fn test_retry_logic() {
        let session_id = Uuid::new_v4();
        let mut state_machine = LoginStateMachine::new(session_id);
        
        // Test retry within limit
        assert!(state_machine.retry().is_ok());
        assert_eq!(state_machine.retry_count(), 1);
        
        // Test max retries exceeded
        state_machine.retry_count = 3;
        assert!(state_machine.retry().is_err());
        assert!(state_machine.is_failed());
    }

    #[test]
    fn test_state_manager() {
        let mut manager = LoginStateManager::new();
        let session_id = Uuid::new_v4();
        
        // Create session
        let state_machine = manager.create_session(session_id);
        assert_eq!(state_machine.session_id(), session_id);
        
        // Get session
        assert!(manager.get_session(&session_id).is_some());
        
        // Cleanup session
        manager.cleanup_session(&session_id);
        assert!(manager.get_session(&session_id).is_none());
    }
}