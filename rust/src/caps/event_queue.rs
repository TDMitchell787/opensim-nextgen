use crate::login_stage_tracker::LoginStage;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQueueEvent {
    pub message: String,
    pub body: Value,
}

#[derive(Debug)]
struct SessionEventQueue {
    events: VecDeque<EventQueueEvent>,
    pending_request: Option<oneshot::Sender<EventQueueResponse>>,
    last_ack: u64,
    next_id: u64,
    last_activity: Instant,
}

impl SessionEventQueue {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            pending_request: None,
            last_ack: 0,
            next_id: 1,
            last_activity: Instant::now(),
        }
    }

    fn add_event(&mut self, event: EventQueueEvent) {
        self.events.push_back(event);
        self.last_activity = Instant::now();

        if let Some(sender) = self.pending_request.take() {
            let response = self.create_response();
            if let Err(failed_response) = sender.send(response) {
                warn!(
                    "Failed to send EQ response to waiting client, re-queuing {} events",
                    failed_response.events.len()
                );
                for evt in failed_response.events {
                    self.events.push_back(evt);
                }
                self.next_id = failed_response.id;
            }
        }
    }

    fn add_events_batch(&mut self, events: Vec<EventQueueEvent>) {
        for event in events {
            self.events.push_back(event);
        }
        self.last_activity = Instant::now();

        if let Some(sender) = self.pending_request.take() {
            let response = self.create_response();
            if let Err(failed_response) = sender.send(response) {
                warn!(
                    "Failed to send EQ batch response to waiting client, re-queuing {} events",
                    failed_response.events.len()
                );
                for evt in failed_response.events {
                    self.events.push_back(evt);
                }
                self.next_id = failed_response.id;
            }
        }
    }

    fn set_pending_request(&mut self, sender: oneshot::Sender<EventQueueResponse>) {
        if let Some(_old_sender) = self.pending_request.take() {
            // Drop old sender — old receiver gets Err → handler returns 502
            // DO NOT send empty 200 response — that causes viewer to rapid-retry,
            // creating 409K+ EQG requests/session flooding loop
        }
        self.pending_request = Some(sender);
        self.last_activity = Instant::now();
    }

    fn create_response(&mut self) -> EventQueueResponse {
        let events: Vec<EventQueueEvent> = self.events.drain(..).collect();
        let response_id = self.next_id;
        self.next_id += 1;

        EventQueueResponse {
            events,
            id: response_id,
        }
    }

    fn acknowledge(&mut self, ack_id: u64) {
        if ack_id > self.last_ack {
            self.last_ack = ack_id;
            self.last_activity = Instant::now();
        }
    }

    fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventQueueResponse {
    pub events: Vec<EventQueueEvent>,
    pub id: u64,
}

impl EventQueueResponse {
    fn empty(id: u64) -> Self {
        Self {
            events: Vec::new(),
            id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EventQueueQuery {
    pub ack: Option<u64>,
}

#[derive(Debug)]
pub struct EventQueueManager {
    sessions: Arc<RwLock<HashMap<String, SessionEventQueue>>>,
}

impl EventQueueManager {
    pub fn new() -> Self {
        let manager = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start cleanup task
        let sessions_clone = manager.sessions.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                Self::cleanup_expired_sessions(&sessions_clone).await;
            }
        });

        manager
    }

    pub async fn create_session(&self, session_id: String) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), SessionEventQueue::new());
        debug!("Created event queue for session: {}", session_id);
    }

    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(mut queue) = sessions.remove(session_id) {
            // Cancel any pending request
            if let Some(sender) = queue.pending_request.take() {
                let _ = sender.send(EventQueueResponse::empty(queue.next_id));
            }
            debug!("Removed event queue for session: {}", session_id);
        }
    }

    pub async fn send_event(&self, session_id: &str, event: EventQueueEvent) {
        let mut sessions = self.sessions.write().await;
        if let Some(queue) = sessions.get_mut(session_id) {
            let has_pending = queue.pending_request.is_some();
            info!(
                "📡 [EQ] Queueing event '{}' to session: {} (pending_request: {})",
                event.message, session_id, has_pending
            );
            queue.add_event(event);
            info!(
                "📡 [EQ] Event queued successfully, queue now has {} events",
                queue.events.len()
            );
        } else {
            warn!(
                "📡 [EQ] ❌ Attempted to send event to non-existent session: {}",
                session_id
            );
        }
    }

    pub async fn send_events_batch(&self, session_id: &str, events: Vec<EventQueueEvent>) {
        let mut sessions = self.sessions.write().await;
        if let Some(queue) = sessions.get_mut(session_id) {
            debug!(
                "Sending {} events as batch to session: {}",
                events.len(),
                session_id
            );
            queue.add_events_batch(events);
        } else {
            warn!(
                "Attempted to send events to non-existent session: {}",
                session_id
            );
        }
    }

    pub async fn notify_udp_circuit_ready(&self, session_id: &str, agent_id: &str) {
        // Check if viewer is waiting on EventQueueGet and UDP circuit just became ready
        // If so, send login events immediately
        let mut sessions = self.sessions.write().await;
        if let Some(queue) = sessions.get_mut(session_id) {
            // Only send if there's a pending request waiting and no events yet
            if queue.pending_request.is_some() && queue.events.is_empty() && queue.next_id == 1 {
                info!("🎯 UDP circuit ready, sending login events to waiting EventQueueGet for session: {}", session_id);
                drop(sessions); // Release lock before async call
                self.send_delayed_login_events(session_id, agent_id).await;
            }
        }
    }

    pub async fn get_events(
        &self,
        session_id: &str,
        ack: Option<u64>,
    ) -> Result<EventQueueResponse, StatusCode> {
        // Handle acknowledgment if provided
        if let Some(ack_id) = ack {
            let mut sessions = self.sessions.write().await;
            if let Some(queue) = sessions.get_mut(session_id) {
                queue.acknowledge(ack_id);
            }
        }

        // Check if we have events ready
        {
            let mut sessions = self.sessions.write().await;
            if let Some(queue) = sessions.get_mut(session_id) {
                if !queue.events.is_empty() {
                    queue.last_activity = Instant::now(); // CRITICAL: Update activity timestamp!
                    let response = queue.create_response();
                    debug!(
                        "Returning {} events for session: {}",
                        response.events.len(),
                        session_id
                    );
                    return Ok(response);
                }
            } else {
                warn!(
                    "Event queue request for non-existent session: {}",
                    session_id
                );
                return Err(StatusCode::NOT_FOUND);
            }
        }

        // No events ready, set up long-polling
        let (sender, receiver) = oneshot::channel();

        {
            let mut sessions = self.sessions.write().await;
            if let Some(queue) = sessions.get_mut(session_id) {
                if queue.pending_request.is_some() {
                    // Already have a pending long-poll for this session.
                    // Reject the NEW request immediately with 502 to prevent flooding.
                    // The existing pending request will continue to wait for events.
                    debug!(
                        "📡 [EQ] Rejecting duplicate EQG request for session {} (already pending)",
                        session_id
                    );
                    return Err(StatusCode::BAD_GATEWAY);
                }
                queue.last_activity = Instant::now();
                queue.set_pending_request(sender);
                debug!("Set up long-polling for session: {}", session_id);
            } else {
                return Err(StatusCode::NOT_FOUND);
            }
        }

        // Wait for events or timeout (30 seconds - typical for EventQueueGet)
        match timeout(Duration::from_secs(30), receiver).await {
            Ok(Ok(response)) => {
                info!(
                    "📡 [EQ] Long-poll fulfilled with {} events for session: {}",
                    response.events.len(),
                    session_id
                );
                Ok(response)
            }
            Ok(Err(_)) => {
                info!(
                    "📡 [EQ] Long-poll receiver dropped (replaced) for session: {}",
                    session_id
                );
                Err(StatusCode::BAD_GATEWAY)
            }
            Err(_) => {
                info!("📡 [EQ] Long-poll 30s timeout for session: {}", session_id);
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }

    pub async fn send_delayed_login_events(&self, session_id: &str, agent_id: &str) {
        // Phase 68.13: Do NOT send EstablishAgentCommunication during fresh login!
        // Per OpenSim EventQueueGetHandlers.cs, this event is ONLY sent during:
        // - Teleport to another region
        // - Child agent on neighbor region
        // - Informing about neighbor region
        // The login response already contains seed_capability - no need to send again.
        // Sending it during fresh login may confuse the viewer state machine.
        info!("🎯 [Phase 68.13] Fresh login - NOT sending EstablishAgentCommunication");
        info!(
            "🎯 [Phase 68.13] Session: {}, Agent: {} (login response has seed_capability)",
            session_id, agent_id
        );

        // EventQueue is ready but empty for fresh login - viewer doesn't need events here
        // The viewer already has all necessary info from login response
    }

    pub async fn send_login_events_on_first_request(
        &self,
        session_id: &str,
        agent_id: &str,
        udp_circuit_ready: bool,
    ) {
        // Check if this session needs initial login events
        {
            let mut sessions = self.sessions.write().await;
            if let Some(queue) = sessions.get_mut(session_id) {
                // Only send events if no events have been sent yet
                if queue.events.is_empty() && queue.next_id == 1 {
                    if udp_circuit_ready {
                        info!("🎯 UDP circuit ready, sending login events on first EventQueueGet for session: {}", session_id);
                        drop(sessions); // Release lock before async call

                        // Send the login events
                        self.send_delayed_login_events(session_id, agent_id).await;
                    } else {
                        info!("⏳ EventQueueGet arrived but UDP circuit not ready yet for session: {}", session_id);
                        // Don't send events yet - will be sent when UDP circuit becomes ready via notify_udp_circuit_ready()
                    }
                }
            }
        }
    }

    async fn cleanup_expired_sessions(sessions: &Arc<RwLock<HashMap<String, SessionEventQueue>>>) {
        let timeout_duration = Duration::from_secs(300); // 5 minutes
        let mut sessions_guard = sessions.write().await;

        let expired_sessions: Vec<String> = sessions_guard
            .iter()
            .filter(|(_, queue)| queue.is_expired(timeout_duration))
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            if let Some(mut queue) = sessions_guard.remove(&session_id) {
                // Cancel any pending request
                if let Some(sender) = queue.pending_request.take() {
                    let _ = sender.send(EventQueueResponse::empty(queue.next_id));
                }
                debug!("Removed expired event queue session: {}", session_id);
            }
        }
    }
}

// Axum handler for EventQueueGet
pub async fn handle_event_queue_get(
    Path(eqg_uuid): Path<String>,
    Query(query): Query<EventQueueQuery>,
    State(state): State<crate::caps::CapsHandlerState>,
) -> Result<axum::response::Response, StatusCode> {
    use serde_json::json;

    debug!(
        "📡 EventQueueGet request for EQG UUID: {} (ack: {:?})",
        eqg_uuid, query.ack
    );

    // Look up session by EQG UUID
    let session = state.caps_manager.get_session_by_eqg_uuid(&eqg_uuid).await;
    if session.is_none() {
        warn!(
            "📡 EventQueueGet failed: No session found for EQG UUID: {}",
            eqg_uuid
        );
        return Err(StatusCode::NOT_FOUND);
    }

    let session = session.unwrap();
    let session_id = session.session_id.clone();
    let agent_id = session.agent_id.clone();
    let udp_circuit_ready = session.udp_circuit_ready;
    debug!(
        "📡 EventQueueGet mapped EQG UUID {} to session {} (UDP circuit ready: {})",
        eqg_uuid, session_id, udp_circuit_ready
    );

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Get events from the event queue manager
    let event_queue = state.caps_manager.get_event_queue();

    // CRITICAL: Send login events on first EventQueueGet request ONLY if UDP circuit is ready
    // Viewer must establish UDP connection before receiving EventQueue events
    event_queue
        .send_login_events_on_first_request(&session_id, &agent_id, udp_circuit_ready)
        .await;

    match event_queue.get_events(&session_id, query.ack).await {
        Ok(response) => {
            if !response.events.is_empty() {
                info!(
                    "📡 EventQueueGet response for session: {} with {} events",
                    session_id,
                    response.events.len()
                );
            }

            // Phase 70.27: Mark Stage 10 (EVENTQUEUE_ACTIVE) when first EQ response is sent
            // This confirms the viewer has established EventQueue communication
            state
                .stage_tracker
                .mark_passed_by_circuit(
                    session.circuit_code,
                    LoginStage::EventQueueActive,
                    Some(format!(
                        "First EQ response with {} events",
                        response.events.len()
                    )),
                )
                .await;

            let response_json = json!({
                "events": response.events,
                "id": response.id
            });

            if !response.events.is_empty() {
                for (i, event) in response.events.iter().enumerate() {
                    info!(
                        "📡 [DEBUG] Event {}: message='{}', body={}",
                        i,
                        event.message,
                        serde_json::to_string_pretty(&event.body).unwrap_or_default()
                    );
                }
            }

            crate::caps::handlers::json_response_to_llsd_xml(response_json)
        }
        Err(status) => {
            if status == StatusCode::BAD_GATEWAY {
                debug!(
                    "📡 EventQueueGet 502 for session: {} (normal long-poll timeout/replace)",
                    session_id
                );
            } else {
                warn!(
                    "📡 EventQueueGet failed for session: {} with status: {:?}",
                    session_id, status
                );
            }
            Err(status)
        }
    }
}
