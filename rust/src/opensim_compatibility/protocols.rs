//! OpenSim protocol compatibility
//!
//! Provides compatibility with OpenSimulator's communication protocols
//! including LLUDP, ROBUST services, and Hypergrid.

use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
// use crate::login_session::CircuitError; // TODO: Define CircuitError if needed
use super::message_templates::{MessageTemplateManager, MessageTemplate as NewMessageTemplate};

const DEFAULT_REGION_HANDLE: u64 = (256000_u64 << 32) | 256000_u64;

/// LLUDP Client Stack for Second Life viewer compatibility
pub struct LLUDPClientStack {
    socket: Arc<UdpSocket>,
    clients: Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
    message_template_manager: Arc<MessageTemplateManager>,
    running: Arc<RwLock<bool>>,
    circuit_codes: Option<crate::login_session::CircuitCodeRegistry>,
}

/// Individual LLUDP client connection
#[derive(Debug, Clone)]
pub struct LLUDPClient {
    pub address: SocketAddr,
    pub circuit_code: u32,
    pub session_id: String,
    pub agent_id: String,
    pub sequence_number: u32,
    pub last_packet_time: std::time::Instant,
    pub acknowledged_packets: Vec<u32>,
    pub pending_acks: Vec<u32>,
}

/// Message template for LLUDP protocol
pub struct MessageTemplate {
    messages: HashMap<String, MessageDefinition>,
    message_numbers: HashMap<u16, String>,
}

/// Message definition from message_template.msg
#[derive(Debug, Clone)]
pub struct MessageDefinition {
    pub name: String,
    pub number: u16,
    pub frequency: MessageFrequency,
    pub trusted: bool,
    pub blocks: Vec<MessageBlock>,
}

/// Message frequency (High, Medium, Low, Fixed)
#[derive(Debug, Clone, PartialEq)]
pub enum MessageFrequency {
    High,
    Medium, 
    Low,
    Fixed,
}

/// Message block definition
#[derive(Debug, Clone)]
pub struct MessageBlock {
    pub name: String,
    pub block_type: BlockType,
    pub fields: Vec<MessageField>,
}

/// Block type (Single, Multiple, Variable)
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Single,
    Multiple(u8), // Max count
    Variable,
}

/// Message field definition
#[derive(Debug, Clone)]
pub struct MessageField {
    pub name: String,
    pub field_type: FieldType,
    pub size: Option<u8>,
}

/// Field types in LLUDP messages
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
    LLVector3,
    LLVector3d,
    LLQuaternion,
    LLUUID,
    Bool,
    IPAddr,
    IPPort,
    Variable(u8), // Variable length with size prefix
    Fixed(u8),    // Fixed length
}

/// ROBUST service connector for grid services
pub struct ROBUSTConnector {
    base_url: String,
    client: reqwest::Client,
    auth_token: Option<String>,
}

impl LLUDPClientStack {
    /// Create a new LLUDP client stack
    pub async fn new(bind_address: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_address).await
            .map_err(|e| anyhow::anyhow!("Failed to bind UDP socket: {}", e))?;
        
        let message_template_manager = Arc::new(MessageTemplateManager::new());
        
        Ok(Self {
            socket: Arc::new(socket),
            clients: Arc::new(RwLock::new(HashMap::new())),
            message_template_manager,
            running: Arc::new(RwLock::new(false)),
            circuit_codes: None,
        })
    }

    /// Start the LLUDP server
    pub async fn start(&self) -> Result<()> {
        *self.running.write().await = true;
        
        let socket = self.socket.clone();
        let clients = self.clients.clone();
        let running = self.running.clone();
        let circuit_codes = self.circuit_codes.clone();
        let template_manager = self.message_template_manager.clone();
        
        tokio::spawn(async move {
            let mut buffer = [0u8; 1500]; // MTU size
            
            while *running.read().await {
                match socket.recv_from(&mut buffer).await {
                    Ok((size, addr)) => {
                        if let Err(e) = Self::handle_packet(&clients, &circuit_codes, &template_manager, &socket, &buffer[..size], addr).await {
                            tracing::error!("Failed to handle packet from {}: {}", addr, e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("UDP receive error: {}", e);
                        break;
                    }
                }
            }
        });

        tracing::info!("LLUDP ClientStack started on {}", self.socket.local_addr()?);
        Ok(())
    }

    /// Start the LLUDP server with circuit code registry
    pub async fn start_with_circuit_codes(&mut self, circuit_codes: crate::login_session::CircuitCodeRegistry) -> Result<()> {
        self.circuit_codes = Some(circuit_codes);
        self.start().await
    }

    /// Get message template manager
    pub fn get_message_template_manager(&self) -> Arc<MessageTemplateManager> {
        self.message_template_manager.clone()
    }

    /// Validate message using template system
    async fn validate_message_with_template(&self, message_number: u32, data: &[u8]) -> Result<String> {
        match self.message_template_manager.get_template_by_number(message_number) {
            Ok(template) => {
                // Validate message structure
                if let Err(e) = self.message_template_manager.validate_message(&template.name, data) {
                    tracing::warn!("Message validation failed for {}: {}", template.name, e);
                } else {
                    tracing::debug!("Message {} validated successfully", template.name);
                }
                Ok(template.name.clone())
            }
            Err(_) => {
                tracing::warn!("Unknown message number: {}", message_number);
                Ok(format!("UnknownMessage_{}", message_number))
            }
        }
    }

    /// Check if message is critical for login process
    fn is_critical_login_message(&self, message_name: &str) -> bool {
        self.message_template_manager.is_critical_message(message_name)
    }

    /// Stop the LLUDP server
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    /// Handle incoming UDP packet
    async fn handle_packet(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        circuit_codes: &Option<crate::login_session::CircuitCodeRegistry>,
        template_manager: &Arc<MessageTemplateManager>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
    ) -> Result<()> {
        if data.len() < 6 {
            return Err(anyhow::anyhow!("Packet too short".to_string()));
        }

        // Parse packet header
        let flags = data[0];
        let sequence = u32::from_be_bytes([0, 0, data[1], data[2]]);
        let extra_length = data[3];
        
        let mut offset = 4 + extra_length as usize;
        
        // Handle zero-coded packets
        let packet_data = if flags & 0x80 != 0 {
            Self::decode_zero_coded(&data[offset..])?
        } else {
            data[offset..].to_vec()
        };

        // Extract message number
        if packet_data.len() < 2 {
            return Err(anyhow::anyhow!("Invalid message data".to_string()));
        }

        let message_num = if packet_data[0] == 0xFF {
            if packet_data.len() < 4 {
                return Err(anyhow::anyhow!("Invalid extended message".to_string()));
            }
            u16::from_be_bytes([packet_data[1], packet_data[2]])
        } else {
            packet_data[0] as u16
        };

        // Validate message using template system
        let message_name = match template_manager.get_template_by_number(message_num as u32) {
            Ok(template) => {
                // Validate message structure
                if let Err(e) = template_manager.validate_message(&template.name, &packet_data) {
                    tracing::warn!("Message validation failed for {} ({}): {}", template.name, message_num, e);
                } else {
                    tracing::debug!("Message {} ({}) validated successfully", template.name, message_num);
                }
                
                // Log critical messages for debugging
                if template_manager.is_critical_message(&template.name) {
                    tracing::info!("Critical message received: {} from {}", template.name, addr);
                }
                
                template.name.clone()
            }
            Err(_) => {
                tracing::warn!("Unknown message number: {} from {}", message_num, addr);
                format!("UnknownMessage_{}", message_num)
            }
        };

        // Handle acknowledgments
        if flags & 0x10 != 0 {
            // This packet contains acknowledgments
            Self::handle_acknowledgments(&packet_data, addr).await?;
        }

        // Handle the actual message (now with validated message name available)
        Self::handle_message(clients, circuit_codes, socket, message_num, &packet_data, addr, sequence).await?;
        
        Ok(())
    }

    /// Decode zero-coded packet data
    fn decode_zero_coded(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoded = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            if data[i] == 0x00 {
                if i + 1 >= data.len() {
                    break;
                }
                let count = data[i + 1];
                decoded.extend(vec![0u8; count as usize]);
                i += 2;
            } else {
                decoded.push(data[i]);
                i += 1;
            }
        }
        
        Ok(decoded)
    }

    /// Handle message acknowledgments
    async fn handle_acknowledgments(data: &[u8], _addr: SocketAddr) -> Result<()> {
        // Parse acknowledgment data and update client state
        // This is a simplified implementation
        tracing::debug!("Handling acknowledgments from packet");
        Ok(())
    }

    /// Handle specific message
    async fn handle_message(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        circuit_codes: &Option<crate::login_session::CircuitCodeRegistry>,
        socket: &Arc<UdpSocket>,
        message_num: u16,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        match message_num {
            1 => Self::handle_use_circuit_code(clients, circuit_codes, socket, data, addr).await,
            2 => Self::handle_complete_agent_movement(clients, socket, data, addr, sequence).await,
            3 => Self::handle_agent_update(clients, data, addr, sequence).await,
            4 => Self::handle_complete_ping_check(clients, data, addr, sequence).await,
            5 => Self::handle_agent_animation(clients, data, addr, sequence).await,
            6 => Self::handle_agent_request_sit(clients, data, addr, sequence).await,
            7 => Self::handle_agent_throttle(clients, data, addr, sequence).await,
            8 => Self::handle_packet_ack(clients, data, addr, sequence).await,
            9 => Self::handle_packet_ack(clients, data, addr, sequence).await,
            10 => Self::handle_agent_cached_texture(clients, socket, data, addr, sequence).await,
            11 => Self::handle_layer_data(clients, socket, data, addr, sequence).await,
            12 => Self::handle_request_image(clients, data, addr, sequence).await,
            13 => Self::handle_packet_ack(clients, data, addr, sequence).await, // Generic ack handler
            14 => Self::handle_agent_wearables_request(clients, socket, data, addr, sequence).await,
            15 => Self::handle_request_multiple_objects(clients, socket, data, addr, sequence).await,
            16 => Self::handle_object_update_request(clients, data, addr, sequence).await,
            17 => Self::handle_packet_ack(clients, data, addr, sequence).await, // Generic ack handler
            18 => Self::handle_agent_animation(clients, data, addr, sequence).await,
            19 => Self::handle_agent_throttle(clients, data, addr, sequence).await, // AgentThrottle
            20 => Self::handle_agent_pause(clients, data, addr, sequence).await,
            21 => Self::handle_agent_resume(clients, data, addr, sequence).await,
            22 => Self::handle_agent_setappearance(clients, data, addr, sequence).await,
            23 => Self::handle_agent_is_now_wearing(clients, data, addr, sequence).await,
            24 => Self::handle_request_region_info(clients, socket, data, addr, sequence).await,
            25 => Self::handle_estate_covenant_request(clients, socket, data, addr, sequence).await,
            26 => Self::handle_request_godlike_powers(clients, socket, data, addr, sequence).await,
            27 => Self::handle_godlike_message(clients, socket, data, addr, sequence).await,
            28 => Self::handle_economy_data_request(clients, socket, data, addr, sequence).await,
            29 => Self::handle_avatar_properties_request(clients, socket, data, addr, sequence).await,
            30 => Self::handle_avatar_interests_request(clients, socket, data, addr, sequence).await,
            31 => Self::handle_avatar_groups_request(clients, socket, data, addr, sequence).await,
            32 => Self::handle_avatar_picks_request(clients, socket, data, addr, sequence).await,
            33 => Self::handle_avatar_classifieds_request(clients, socket, data, addr, sequence).await,
            149 => Self::handle_region_handshake_reply(clients, socket, data, addr, sequence).await,
            _ => {
                tracing::info!("Unhandled message type: {} from {}", message_num, addr);
                Ok(())
            }
        }
    }

    /// Handle UseCircuitCode message (establish connection)
    async fn handle_use_circuit_code(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        circuit_codes: &Option<crate::login_session::CircuitCodeRegistry>,
        socket: &Arc<tokio::net::UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
    ) -> Result<()> {
        // Parse UseCircuitCode message
        // LLUDP structure: [message_num][block_count][circuit_code][session_id][agent_id]
        if data.len() < 22 {
            return Err(anyhow::anyhow!("Invalid UseCircuitCode message".to_string()));
        }

        // Circuit code starts at byte 2 (after message_num and block_count)
        let circuit_code = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        tracing::info!("UseCircuitCode received from {} with circuit code: {}", addr, circuit_code);

        // Accept any circuit code and try to find a matching session
        if let Some(registry) = circuit_codes {
            // First try to validate the exact circuit code
            match registry.validate_circuit_code(circuit_code).await {
                Some(login_session) => {
                    tracing::info!("Valid circuit code {} for user {} {}", 
                        circuit_code, login_session.first_name, login_session.last_name);

                let client = LLUDPClient {
                    address: addr,
                    circuit_code,
                    session_id: login_session.session_id,
                    agent_id: login_session.agent_id,
                    sequence_number: 0,
                    last_packet_time: std::time::Instant::now(),
                    acknowledged_packets: Vec::new(),
                    pending_acks: Vec::new(),
                };

                clients.write().await.insert(addr, client);
                tracing::info!("LLUDP client authenticated: {} {} (circuit: {})", 
                    login_session.first_name, login_session.last_name, circuit_code);
                
                // Send essential login packets immediately
                if let Err(e) = Self::send_login_sequence(socket, addr, circuit_code).await {
                    tracing::error!("Failed to send login sequence: {}", e);
                }
                
                // Remove the circuit code from registry as it's now in use
                registry.remove_circuit_code(circuit_code).await;
                
                return Ok(());
                }
                None => {
                    // Circuit code doesn't match - try to find any recent login session for this IP
                    // This handles viewers that use their own circuit codes
                    tracing::warn!("Circuit code {} validation failed, checking for recent login sessions from {}", circuit_code, addr);
                    
                    // For now, accept any circuit code from recent logins (within last 60 seconds)
                    // This is a temporary compatibility fix
                    let recent_session = registry.get_most_recent_session().await;
                    if let Some(session) = recent_session {
                        tracing::info!("Accepting circuit code {} for recent login session {} {}", 
                            circuit_code, session.first_name, session.last_name);
                        
                        let client = LLUDPClient {
                            address: addr,
                            circuit_code,
                            session_id: session.session_id.clone(),
                            agent_id: session.agent_id.clone(),
                            sequence_number: 0,
                            last_packet_time: std::time::Instant::now(),
                            acknowledged_packets: Vec::new(),
                            pending_acks: Vec::new(),
                        };

                        clients.write().await.insert(addr, client);
                        tracing::info!("LLUDP client authenticated with fallback: {} {} (circuit: {})", 
                            session.first_name, session.last_name, circuit_code);
                        
                        // Send essential login packets immediately
                        if let Err(e) = Self::send_login_sequence(&*socket, addr, circuit_code).await {
                            tracing::error!("Failed to send login sequence: {}", e);
                        }
                        
                        return Ok(());
                    } else {
                        tracing::error!("No recent login sessions found for circuit code {} from {}", circuit_code, addr);
                        return Err(anyhow::anyhow!("Invalid circuit code: {}", circuit_code));
                    }
                }
            }
        }

        // Fallback for testing without registry
        let client = LLUDPClient {
            address: addr,
            circuit_code,
            session_id: "00000000-0000-0000-0000-000000000000".to_string(),
            agent_id: "00000000-0000-0000-0000-000000000000".to_string(),
            sequence_number: 0,
            last_packet_time: std::time::Instant::now(),
            acknowledged_packets: Vec::new(),
            pending_acks: Vec::new(),
        };

        clients.write().await.insert(addr, client);
        tracing::warn!("LLUDP client connected without validation: {} (circuit: {})", addr, circuit_code);
        
        Ok(())
    }

    /// Handle CompleteAgentMovement message
    async fn handle_complete_agent_movement(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Agent movement completed for client: {} - SENDING FINAL LOGIN CONFIRMATION", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };

        // Send CRITICAL final confirmation packets
        // AgentMovementComplete response
        let movement_complete_response = Self::create_agent_movement_complete_response_packet(circuit_code)?;
        socket.send_to(&movement_complete_response, addr).await?;
        tracing::info!("Sent AgentMovementCompleteResponse to {} - LOGIN SHOULD BE COMPLETE!", addr);

        // Send CoarseLocationUpdate - shows avatar in world
        let coarse_location = Self::create_coarse_location_update_packet(circuit_code)?;
        socket.send_to(&coarse_location, addr).await?;
        tracing::info!("Sent CoarseLocationUpdate to {}", addr);

        // CRITICAL: Send ObjectUpdate for avatar - viewer needs to see its own avatar
        let avatar_object_update = Self::create_avatar_object_update_packet(circuit_code)?;
        socket.send_to(&avatar_object_update, addr).await?;
        tracing::info!("Sent Avatar ObjectUpdate to {} - AVATAR SPAWNED!", addr);

        // TEMPORARILY REMOVED: AgentDataUpdate packet causing Firestorm crash
        // let agent_data_update = Self::create_agent_data_update_packet(circuit_code)?;
        // socket.send_to(&agent_data_update, addr).await?;
        // tracing::info!("Sent AgentDataUpdate to {} - LOGIN FINALIZED!", addr);

        Ok(())
    }

    /// Handle AgentUpdate message
    async fn handle_agent_update(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            // Parse agent position, rotation, etc. from data
        }
        Ok(())
    }

    /// Handle CompletePingCheck message (respond to viewer ping)
    async fn handle_complete_ping_check(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received CompletePingCheck from {}, ping/pong established", addr);
        }
        Ok(())
    }

    /// Handle RegionHandshakeReply message (viewer responding to our handshake)
    async fn handle_region_handshake_reply(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Received RegionHandshakeReply from {}, handshake confirmed!", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };
        
        // Send critical follow-up packets after handshake confirmation
        // Send SimStats packet - provides region statistics
        let sim_stats = Self::create_sim_stats_packet(circuit_code)?;
        socket.send_to(&sim_stats, addr).await?;
        tracing::info!("Sent SimStats packet to {} after handshake confirmation", addr);
        
        // Send UUIDNameReply for region owner
        let uuid_name_reply = Self::create_uuid_name_reply_packet(circuit_code)?;
        socket.send_to(&uuid_name_reply, addr).await?;
        tracing::info!("Sent UUIDNameReply packet to {} after handshake confirmation", addr);
        
        Ok(())
    }

    /// Handle AgentThrottle message (viewer bandwidth settings)
    async fn handle_agent_throttle(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received AgentThrottle from {}, bandwidth settings updated", addr);
        }
        Ok(())
    }

    /// Handle PacketAck message (packet acknowledgment)
    async fn handle_packet_ack(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            // Parse acknowledgment IDs and update client state
            tracing::debug!("Received PacketAck from {}", addr);
        }
        Ok(())
    }

    /// Handle AgentCachedTexture message (texture cache request)
    async fn handle_agent_cached_texture(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Received AgentCachedTexture from {}, responding with texture info", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };

        // Send AgentCachedTextureResponse
        let texture_response = Self::create_agent_cached_texture_response_packet(circuit_code)?;
        socket.send_to(&texture_response, addr).await?;
        tracing::info!("Sent AgentCachedTextureResponse to {}", addr);

        Ok(())
    }

    /// Handle RequestImage message (texture requests)
    async fn handle_request_image(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received RequestImage from {}", addr);
        }
        Ok(())
    }

    /// Handle AgentWearablesRequest message  
    async fn handle_agent_wearables_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Received AgentWearablesRequest from {}, sending basic wearables", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };

        // Send basic AgentWearablesUpdate response
        let wearables_update = Self::create_agent_wearables_update_packet(circuit_code)?;
        socket.send_to(&wearables_update, addr).await?;
        tracing::info!("Sent AgentWearablesUpdate to {}", addr);

        // Send AvatarAppearance packet - critical for avatar spawn
        let avatar_appearance = Self::create_avatar_appearance_packet(circuit_code)?;
        socket.send_to(&avatar_appearance, addr).await?;
        tracing::info!("Sent AvatarAppearance to {}", addr);

        // Send AgentSetAppearance confirmation
        let agent_set_appearance = Self::create_agent_set_appearance_packet(circuit_code)?;
        socket.send_to(&agent_set_appearance, addr).await?;
        tracing::info!("Sent AgentSetAppearance to {}", addr);

        Ok(())
    }

    /// Handle RequestMultipleObjects message
    async fn handle_request_multiple_objects(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Received RequestMultipleObjects from {}, sending basic objects", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };

        // Send basic ObjectUpdate response
        let object_update = Self::create_object_update_packet(circuit_code)?;
        socket.send_to(&object_update, addr).await?;
        tracing::info!("Sent ObjectUpdate to {}", addr);

        Ok(())
    }

    /// Handle ObjectUpdateRequest message
    async fn handle_object_update_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received ObjectUpdateRequest from {}", addr);
        }
        Ok(())
    }

    /// Handle AgentAnimation message
    async fn handle_agent_animation(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received AgentAnimation from {}", addr);
        }
        Ok(())
    }

    /// Handle AgentRequestSit message
    async fn handle_agent_request_sit(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_guard = clients.write().await;
        if let Some(client) = clients_guard.get_mut(&addr) {
            client.sequence_number = sequence;
            client.last_packet_time = std::time::Instant::now();
            tracing::info!("Received AgentRequestSit from {}", addr);
        }
        Ok(())
    }

    /// Handle LayerData message
    async fn handle_layer_data(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        _data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let circuit_code = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.sequence_number = sequence;
                client.last_packet_time = std::time::Instant::now();
                tracing::info!("Received LayerData request from {}", addr);
                client.circuit_code
            } else {
                return Ok(());
            }
        };

        // Send LayerData response packet
        let layer_data_response = Self::create_layer_data_packet(circuit_code)?;
        socket.send_to(&layer_data_response, addr).await?;
        tracing::info!("Sent LayerData response to {}", addr);
        
        Ok(())
    }

    /// Send packet to client
    pub async fn send_packet(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        self.socket.send_to(data, addr).await?;
        Ok(())
    }

    /// Get connected clients
    pub async fn get_clients(&self) -> Vec<LLUDPClient> {
        self.clients.read().await.values().cloned().collect()
    }

    /// Send essential login sequence packets to complete viewer login
    async fn send_login_sequence(socket: &tokio::net::UdpSocket, addr: SocketAddr, circuit_code: u32) -> Result<()> {
        // Send RegionHandshake packet first - this is critical for viewer login
        let region_handshake = Self::create_region_handshake_packet(circuit_code)?;
        socket.send_to(&region_handshake, addr).await?;
        tracing::info!("Sent RegionHandshake packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // NOTE: AgentMovementComplete is NOT sent here - it should only be sent
        // as a response to CompleteAgentMovement from the viewer (handled in udp/server.rs)
        // Sending it too early causes the viewer to never send CompleteAgentMovement

        // Send EnableSimulator packet  
        let enable_simulator = Self::create_enable_simulator_packet(circuit_code)?;
        socket.send_to(&enable_simulator, addr).await?;
        tracing::info!("Sent EnableSimulator packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send TeleportFinish packet last
        let teleport_finish = Self::create_teleport_finish_packet(circuit_code)?;
        socket.send_to(&teleport_finish, addr).await?;
        tracing::info!("Sent TeleportFinish packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send StartPingCheck packet - this is crucial for viewer ping/pong
        let start_ping_check = Self::create_start_ping_check_packet(circuit_code)?;
        socket.send_to(&start_ping_check, addr).await?;
        tracing::info!("Sent StartPingCheck packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send LayerData packet - provides terrain information
        let layer_data = Self::create_layer_data_packet(circuit_code)?;
        socket.send_to(&layer_data, addr).await?;
        tracing::info!("Sent LayerData packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets  
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send RegionHandshakeReply packet - confirms handshake completion
        let handshake_reply = Self::create_region_handshake_reply_packet(circuit_code)?;
        socket.send_to(&handshake_reply, addr).await?;
        tracing::info!("Sent RegionHandshakeReply packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets  
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send critical completion packets
        let sim_stats = Self::create_sim_stats_packet(circuit_code)?;
        socket.send_to(&sim_stats, addr).await?;
        tracing::info!("Sent SimStats packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets  
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send UUIDNameReply packet - provides UUID name mappings
        let uuid_name_reply = Self::create_uuid_name_reply_packet(circuit_code)?;
        socket.send_to(&uuid_name_reply, addr).await?;
        tracing::info!("Sent UUIDNameReply packet to {} (circuit: {})", addr, circuit_code);

        // Small delay between packets  
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send AvatarAppearance packet - critical for avatar spawn
        let avatar_appearance = Self::create_avatar_appearance_packet(circuit_code)?;
        socket.send_to(&avatar_appearance, addr).await?;
        tracing::info!("Sent AvatarAppearance to {}", addr);

        // Small delay between packets  
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send AgentWearablesUpdate packet - critical for avatar appearance
        let agent_wearables = Self::create_agent_wearables_update_packet(circuit_code)?;
        socket.send_to(&agent_wearables, addr).await?;
        tracing::info!("Sent AgentWearablesUpdate to {}", addr);

        Ok(())
    }

    /// Create RegionHandshake LLUDP packet
    fn create_region_handshake_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0xC0); // Packet flags (reliable + zerocoded) - CRITICAL FIX
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x01); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for RegionHandshake (148) - Low frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&148u16.to_be_bytes()); // Message number (big endian)
        
        // RegionHandshake message data
        packet.push(0x01); // Block count for RegionInfo
        
        // RegionInfo block
        packet.extend_from_slice(&[0x80, 0x00, 0x3E, 0x80]); // RegionFlags (uint32)
        packet.push(0x01); // SimAccess (PG=1, Mature=2, Adult=3)
        
        // SimName (variable length string)
        let sim_name = "OpenSim Test Region";
        packet.push(sim_name.len() as u8);
        packet.extend_from_slice(sim_name.as_bytes());
        
        // SimOwner UUID (fixed 16 bytes)
        let sim_owner_uuid = [0u8; 16];
        packet.extend_from_slice(&sim_owner_uuid);
        
        packet.push(0x01); // IsEstateManager (bool)
        packet.extend_from_slice(&16.0f32.to_le_bytes()); // WaterHeight (float32)
        packet.extend_from_slice(&100.0f32.to_le_bytes()); // BillableFactor (float32)
        
        // CacheID (fixed 16 bytes UUID)
        let cache_id = [0u8; 16];
        packet.extend_from_slice(&cache_id);
        
        // CRITICAL MISSING FIELDS: Terrain texture UUIDs (8 total - Base0-3, Detail0-3)
        // TerrainBase0-3 (each 16 bytes UUID)
        packet.extend_from_slice(&[0u8; 16]); // TerrainBase0
        packet.extend_from_slice(&[0u8; 16]); // TerrainBase1
        packet.extend_from_slice(&[0u8; 16]); // TerrainBase2
        packet.extend_from_slice(&[0u8; 16]); // TerrainBase3
        
        // TerrainDetail0-3 (each 16 bytes UUID) 
        packet.extend_from_slice(&[0u8; 16]); // TerrainDetail0
        packet.extend_from_slice(&[0u8; 16]); // TerrainDetail1
        packet.extend_from_slice(&[0u8; 16]); // TerrainDetail2
        packet.extend_from_slice(&[0u8; 16]); // TerrainDetail3
        
        // Terrain height ranges (all float32)
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // TerrainStartHeight00
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // TerrainStartHeight01
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // TerrainStartHeight10
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // TerrainStartHeight11
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // TerrainHeightRange00
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // TerrainHeightRange01
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // TerrainHeightRange10
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // TerrainHeightRange11
        
        // CRITICAL MISSING BLOCK: RegionInfo2 block (Single)
        packet.push(0x01); // Block count for RegionInfo2
        
        // RegionID (LLUUID) - 16 bytes
        let region_id = [0u8; 16]; // Default region ID
        packet.extend_from_slice(&region_id);
        
        // CRITICAL MISSING BLOCK: RegionInfo3 block (Single)
        packet.push(0x01); // Block count for RegionInfo3
        
        // CPUClassID (S32)
        packet.extend_from_slice(&1i32.to_le_bytes()); // Default CPU class
        
        // CPURatio (S32)
        packet.extend_from_slice(&100i32.to_le_bytes()); // Default CPU ratio
        
        // ColoName (Variable 1) - string
        let colo_name = "OpenSim";
        packet.push(colo_name.len() as u8);
        packet.extend_from_slice(colo_name.as_bytes());
        
        // ProductSKU (Variable 1) - string
        let product_sku = "OpenSim";
        packet.push(product_sku.len() as u8);
        packet.extend_from_slice(product_sku.as_bytes());
        
        // ProductName (Variable 1) - string
        let product_name = "OpenSim Next";
        packet.push(product_name.len() as u8);
        packet.extend_from_slice(product_name.as_bytes());
        
        // SURGICAL FIX: RegionInfo4 block (Variable) - use 0 blocks for basic compatibility
        packet.push(0x00); // Block count for RegionInfo4 (0 blocks - minimal spec)

        Ok(packet)
    }

    /// Create EnableSimulator LLUDP packet  
    fn create_enable_simulator_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x02); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for EnableSimulator (151) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&151u16.to_be_bytes()); // Message number (big endian)
        
        // EnableSimulator message data
        packet.push(0x01); // Block count for SimulatorInfo
        
        // SimulatorInfo block
        packet.extend_from_slice(&0x7F000001u32.to_be_bytes()); // IP (127.0.0.1 as big endian uint32)
        packet.extend_from_slice(&9002u16.to_le_bytes()); // Port
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // Handle (region handle)

        Ok(packet)
    }

    /// Create TeleportFinish LLUDP packet
    fn create_teleport_finish_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x03); // Sequence number (low byte) 
        packet.push(0x00); // Extra header length
        
        // Message number for TeleportFinish (68) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&68u16.to_be_bytes()); // Message number (big endian)
        
        // TeleportFinish message data
        packet.push(0x01); // Block count for Info
        
        // Info block
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        
        packet.extend_from_slice(&4u32.to_le_bytes()); // LocationID
        packet.extend_from_slice(&0x7F000001u32.to_be_bytes()); // SimIP (127.0.0.1 as big endian uint32)
        packet.extend_from_slice(&9002u16.to_le_bytes()); // SimPort
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionHandle (256000)
        
        // SeedCapability (variable length string)
        let seed_cap = "http://127.0.0.1:9001/cap/00000000-0000-0000-0000-000000000001";
        packet.push(seed_cap.len() as u8);
        packet.extend_from_slice(seed_cap.as_bytes());
        
        packet.push(0x01); // SimAccess (PG)
        // Phase 73: TeleportFlags - ViaLogin = 1 << 7 = 128 (NOT 4160 which was ViaHome|ViaTelehub)
        packet.extend_from_slice(&128u32.to_le_bytes()); // TeleportFlags (ViaLogin = 128)

        Ok(packet)
    }

    /// Create AgentMovementComplete LLUDP packet (FIXED to match official specification)
    fn create_agent_movement_complete_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x04); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AgentMovementComplete (250) - Low frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&250u16.to_be_bytes()); // Message number (big endian)
        
        // AgentData block (Single)
        packet.push(0x01); // Block count for AgentData
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        let session_id = [0u8; 16]; // SessionID (UUID) 
        packet.extend_from_slice(&session_id);
        
        // Data block (Single) - CRITICAL: Contains Position, LookAt, RegionHandle, Timestamp
        packet.push(0x01); // Block count for Data
        
        // Position (LLVector3) - Default avatar spawn position
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // X position
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Y position  
        packet.extend_from_slice(&21.0f32.to_le_bytes());  // Z position (ground level)
        
        // LookAt (LLVector3) - Default look direction
        packet.extend_from_slice(&1.0f32.to_le_bytes());   // X direction
        packet.extend_from_slice(&0.0f32.to_le_bytes());   // Y direction
        packet.extend_from_slice(&0.0f32.to_le_bytes());   // Z direction
        
        // RegionHandle (U64)
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionHandle
        
        // Timestamp (U32)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        packet.extend_from_slice(&timestamp.to_le_bytes());
        
        // SimData block (Single) - CRITICAL: Contains ChannelVersion
        packet.push(0x01); // Block count for SimData
        
        // ChannelVersion (Variable 2) - Server version string
        let channel_version = "OpenSim Next 1.0";
        packet.push(channel_version.len() as u8); // Length byte
        packet.extend_from_slice(channel_version.as_bytes());

        Ok(packet)
    }

    /// Create StartPingCheck LLUDP packet
    fn create_start_ping_check_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x05); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for StartPingCheck (2) - High frequency message
        packet.push(0x02); // Message number (single byte for high frequency)
        
        // StartPingCheck message data
        packet.push(0x01); // Block count for PingID
        
        // PingID block
        let ping_id = 1u8; // Starting ping ID
        packet.push(ping_id);
        
        // Old Time block
        packet.push(0x01); // Block count for OldTime
        packet.extend_from_slice(&0u32.to_le_bytes()); // OldTime (0 for first ping)

        Ok(packet)
    }

    /// Create LayerData LLUDP packet (terrain information)
    fn create_layer_data_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x06); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for LayerData (102) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&102u16.to_be_bytes()); // Message number (big endian)
        
        // LayerData message data
        packet.push(0x01); // Block count for LayerID
        
        // LayerID block
        packet.push(0x00); // Type (Land = 0)
        
        // LayerData block
        packet.push(0x01); // Block count for Data
        
        // Simple terrain data (minimal viable terrain)
        let terrain_data = vec![0u8; 128]; // Minimal terrain patch
        packet.push(terrain_data.len() as u8); // Data length
        packet.extend_from_slice(&terrain_data);

        Ok(packet)
    }

    /// Create RegionHandshakeReply LLUDP packet
    fn create_region_handshake_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x07); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for RegionHandshakeReply (149) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&149u16.to_be_bytes()); // Message number (big endian)
        
        // RegionHandshakeReply message data
        packet.push(0x01); // Block count for RegionInfo
        
        // RegionInfo block
        packet.extend_from_slice(&[0x80, 0x00, 0x3E, 0x80]); // RegionFlags
        
        // AgentData block
        packet.push(0x01); // Block count for AgentData
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        
        let session_id = [0u8; 16]; // SessionID (UUID)
        packet.extend_from_slice(&session_id);

        Ok(packet)
    }

    /// Create SimStats LLUDP packet (region statistics)
    fn create_sim_stats_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x08); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for SimStats (140) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&140u16.to_be_bytes()); // Message number (big endian)
        
        // SimStats message data
        packet.push(0x01); // Block count for Region
        
        // Region block
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionX (in region units)
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionY (in region units)
        packet.extend_from_slice(&[0x80, 0x00, 0x3E, 0x80]); // RegionFlags
        packet.extend_from_slice(&20u32.to_le_bytes()); // ObjectCapacity
        
        // Stat block (multiple stats)
        packet.push(0x06); // Block count for Stat (6 basic stats)
        
        // Basic simulation statistics
        packet.extend_from_slice(&20u32.to_le_bytes()); // StatID (FPS)
        packet.extend_from_slice(&45.0f32.to_le_bytes()); // StatValue (45 FPS)
        
        packet.extend_from_slice(&21u32.to_le_bytes()); // StatID (Physics FPS) 
        packet.extend_from_slice(&45.0f32.to_le_bytes()); // StatValue (45 FPS)
        
        packet.extend_from_slice(&22u32.to_le_bytes()); // StatID (Agent Updates)
        packet.extend_from_slice(&10.0f32.to_le_bytes()); // StatValue (10 updates/sec)
        
        packet.extend_from_slice(&23u32.to_le_bytes()); // StatID (Root Agents)
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // StatValue (1 agent)
        
        packet.extend_from_slice(&24u32.to_le_bytes()); // StatID (Active Objects)
        packet.extend_from_slice(&5.0f32.to_le_bytes()); // StatValue (5 objects)
        
        packet.extend_from_slice(&25u32.to_le_bytes()); // StatID (Active Scripts)
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // StatValue (0 scripts)

        Ok(packet)
    }

    /// Create UUIDNameReply LLUDP packet (provides names for UUIDs)
    fn create_uuid_name_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x09); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for UUIDNameReply (231) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&231u16.to_be_bytes()); // Message number (big endian)
        
        // UUIDNameReply message data
        packet.push(0x01); // Block count for UUIDNameBlock
        
        // UUIDNameBlock (region owner info)
        let owner_uuid = [0u8; 16]; // Region owner UUID (zeros for now)
        packet.extend_from_slice(&owner_uuid);
        
        // First name (variable length string)
        let first_name = "OpenSim";
        packet.push(first_name.len() as u8);
        packet.extend_from_slice(first_name.as_bytes());
        
        // Last name (variable length string)  
        let last_name = "Administrator";
        packet.push(last_name.len() as u8);
        packet.extend_from_slice(last_name.as_bytes());

        Ok(packet)
    }

    /// Create AgentCachedTextureResponse LLUDP packet
    fn create_agent_cached_texture_response_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0A); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AgentCachedTextureResponse (237) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&237u16.to_be_bytes()); // Message number (big endian)
        
        // AgentCachedTextureResponse message data
        packet.push(0x01); // Block count for AgentData
        
        // AgentData block
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        
        let session_id = [0u8; 16]; // SessionID (UUID)
        packet.extend_from_slice(&session_id);
        
        packet.extend_from_slice(&0u32.to_le_bytes()); // SerialNum
        
        // WearableData block (empty for now - no cached textures)
        packet.push(0x00); // Block count for WearableData (0 = no cached textures)

        Ok(packet)
    }

    /// Create AgentWearablesUpdate LLUDP packet  
    fn create_agent_wearables_update_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0B); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AgentWearablesUpdate (142) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&142u16.to_be_bytes()); // Message number (big endian)
        
        // AgentWearablesUpdate message data
        packet.push(0x01); // Block count for AgentData
        
        // AgentData block
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        
        let session_id = [0u8; 16]; // SessionID (UUID)
        packet.extend_from_slice(&session_id);
        
        packet.extend_from_slice(&0u32.to_le_bytes()); // SerialNum
        
        // WearableData block (basic default wearables)
        packet.push(0x01); // Block count for WearableData (1 basic wearable)
        
        packet.push(0x00); // ItemID (shape wearable type)
        let item_id = [0u8; 16]; // Item UUID
        packet.extend_from_slice(&item_id);
        
        let asset_id = [0u8; 16]; // Asset UUID  
        packet.extend_from_slice(&asset_id);

        Ok(packet)
    }

    /// Create ObjectUpdate LLUDP packet (basic ground plane)
    fn create_object_update_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0C); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for ObjectUpdate (60) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&60u16.to_be_bytes()); // Message number (big endian)
        
        // ObjectUpdate message data
        packet.push(0x01); // Block count for RegionData
        
        // RegionData block
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionHandle
        packet.extend_from_slice(&0u16.to_le_bytes()); // TimeDilation
        
        // ObjectData block (basic ground plane object)
        packet.push(0x01); // Block count for ObjectData
        
        packet.extend_from_slice(&1u32.to_le_bytes()); // ID (object ID)
        packet.push(0x01); // State (active)
        let full_id = [1u8; 16]; // FullID (UUID)
        packet.extend_from_slice(&full_id);
        
        packet.extend_from_slice(&0u32.to_le_bytes()); // CRC
        packet.push(0x01); // PCode (primitive)
        packet.push(0x00); // Material
        packet.push(0x00); // ClickAction
        
        // Basic object properties (position at origin)
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Scale X
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Scale Y  
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // Scale Z
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Position X
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Position Y
        packet.extend_from_slice(&20.0f32.to_le_bytes()); // Position Z
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // Rotation X
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // Rotation Y
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // Rotation Z
        packet.extend_from_slice(&1.0f32.to_le_bytes()); // Rotation W

        Ok(packet)
    }

    /// Create AvatarAppearance LLUDP packet - CRITICAL for avatar spawn
    fn create_avatar_appearance_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0D); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarAppearance (158) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&158u16.to_be_bytes()); // Message number (big endian)
        
        // AvatarAppearance message data
        packet.push(0x01); // Block count for Sender
        
        // Sender block
        let agent_id = [0u8; 16]; // Sender ID (UUID)
        packet.extend_from_slice(&agent_id);
        
        packet.push(0x01); // IsTrial
        
        // ObjectData block
        packet.push(0x01); // Block count for ObjectData
        
        let texture_entry = vec![0u8; 16]; // Basic texture entry
        packet.push(texture_entry.len() as u8);
        packet.extend_from_slice(&texture_entry);
        
        // VisualParam block (basic avatar shape)
        packet.push(0x01); // Block count for VisualParam
        packet.push(150u8); // Basic param value for default avatar
        
        // AppearanceData block
        packet.push(0x01); // Block count for AppearanceData
        let appearance_version = 1u32;
        packet.extend_from_slice(&appearance_version.to_le_bytes());
        packet.extend_from_slice(&0u32.to_le_bytes()); // COFVersion
        
        // AppearanceHover block
        packet.push(0x01); // Block count
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // HoverHeight

        Ok(packet)
    }

    /// Create AgentSetAppearance LLUDP packet
    fn create_agent_set_appearance_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0E); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AgentSetAppearance (159) - Fixed frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&159u16.to_be_bytes()); // Message number (big endian)
        
        // AgentSetAppearance message data
        packet.push(0x01); // Block count for AgentData
        
        // AgentData block
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        
        let session_id = [0u8; 16]; // SessionID (UUID)
        packet.extend_from_slice(&session_id);
        
        packet.extend_from_slice(&0u32.to_le_bytes()); // SerialNum
        packet.extend_from_slice(&128u32.to_le_bytes()); // Size (medium avatar)

        Ok(packet)
    }

    /// Create AgentMovementCompleteResponse LLUDP packet - FINAL LOGIN CONFIRMATION (FIXED)
    fn create_agent_movement_complete_response_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x0F); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for AgentMovementComplete (250) - Low frequency message
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&250u16.to_be_bytes()); // Message number (big endian)
        
        // AgentData block (Single)
        packet.push(0x01); // Block count for AgentData
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);
        let session_id = [0u8; 16]; // SessionID (UUID)
        packet.extend_from_slice(&session_id);
        
        // Data block (Single) - CRITICAL: Contains Position, LookAt, RegionHandle, Timestamp
        packet.push(0x01); // Block count for Data
        
        // Position (LLVector3) - Default avatar spawn position
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // X position
        packet.extend_from_slice(&128.0f32.to_le_bytes()); // Y position  
        packet.extend_from_slice(&21.0f32.to_le_bytes());  // Z position (ground level)
        
        // LookAt (LLVector3) - Default look direction
        packet.extend_from_slice(&1.0f32.to_le_bytes());   // X direction
        packet.extend_from_slice(&0.0f32.to_le_bytes());   // Y direction
        packet.extend_from_slice(&0.0f32.to_le_bytes());   // Z direction
        
        // RegionHandle (U64)
        packet.extend_from_slice(&DEFAULT_REGION_HANDLE.to_le_bytes()); // RegionHandle
        
        // Timestamp (U32)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        packet.extend_from_slice(&timestamp.to_le_bytes());
        
        // SimData block (Single) - CRITICAL: Contains ChannelVersion
        packet.push(0x01); // Block count for SimData
        
        // ChannelVersion (Variable 2) - Server version string
        let channel_version = "OpenSim Next 1.0";
        packet.push(channel_version.len() as u8); // Length byte
        packet.extend_from_slice(channel_version.as_bytes());

        Ok(packet)
    }

    /// Create CoarseLocationUpdate LLUDP packet - Shows avatar in world
    fn create_coarse_location_update_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header (6 bytes)
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high byte)
        packet.push(0x00); // Sequence number (mid byte)
        packet.push(0x10); // Sequence number (low byte)
        packet.push(0x00); // Extra header length
        
        // Message number for CoarseLocationUpdate (6) - High frequency message
        packet.push(0x06); // Message number (single byte for high frequency)
        
        // CoarseLocationUpdate message data
        packet.push(0x01); // Block count for Location
        
        // Location block (avatar position)
        packet.push(128u8); // X position (center of region)
        packet.push(128u8); // Y position (center of region)
        packet.push(20u8); // Z position (ground level)
        
        // AgentData block
        packet.push(0x01); // Block count for AgentData
        let agent_id = [0u8; 16]; // AgentID (UUID)
        packet.extend_from_slice(&agent_id);

        Ok(packet)
    }
}

impl MessageTemplate {
    /// Load default message template (simplified)
    fn load_default() -> Result<Self> {
        let mut messages = HashMap::new();
        let mut message_numbers = HashMap::new();

        // Add some basic message definitions
        let use_circuit_code = MessageDefinition {
            name: "UseCircuitCode".to_string(),
            number: 1,
            frequency: MessageFrequency::Fixed,
            trusted: true,
            blocks: vec![
                MessageBlock {
                    name: "CircuitCode".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "Code".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                        MessageField {
                            name: "SessionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "ID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                    ],
                },
            ],
        };

        messages.insert("UseCircuitCode".to_string(), use_circuit_code.clone());
        message_numbers.insert(1, "UseCircuitCode".to_string());

        Ok(Self {
            messages,
            message_numbers,
        })
    }

    /// Get message definition by name
    pub fn get_message(&self, name: &str) -> Option<&MessageDefinition> {
        self.messages.get(name)
    }

    /// Get message definition by number
    pub fn get_message_by_number(&self, number: u16) -> Option<&MessageDefinition> {
        self.message_numbers.get(&number)
            .and_then(|name| self.messages.get(name))
    }
}

impl ROBUSTConnector {
    /// Create a new ROBUST connector
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            auth_token: None,
        }
    }

    /// Authenticate with ROBUST service
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let auth_url = format!("{}/auth", self.base_url);
        let params = [
            ("METHOD", "login"),
            ("UserID", username),
            ("Password", password),
        ];

        let response = self.client
            .post(&auth_url)
            .form(&params)
            .send()
            .await?;

        let text = response.text().await?;
        
        // Parse ROBUST response (simplified)
        if text.contains("Success=true") {
            // Extract session token from response
            if let Some(start) = text.find("SessionID=") {
                let token_start = start + 10;
                if let Some(end) = text[token_start..].find('\n') {
                    self.auth_token = Some(text[token_start..token_start + end].to_string());
                }
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("ROBUST authentication failed".to_string()))
        }
    }

    /// Make authenticated request to ROBUST service
    pub async fn request(&self, service: &str, method: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = format!("{}/{}", self.base_url, service);
        
        let mut form_params = vec![("METHOD", method)];
        form_params.extend_from_slice(params);
        
        if let Some(token) = &self.auth_token {
            form_params.push(("SessionID", token));
        }

        let response = self.client
            .post(&url)
            .form(&form_params)
            .send()
            .await?;

        Ok(response.text().await?)
    }
}

impl LLUDPClientStack {
    // Additional message handlers for complete login sequence

    /// Handle AgentPause message (20)
    async fn handle_agent_pause(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AgentPause from {}", addr);
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AgentResume message (21)
    async fn handle_agent_resume(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AgentResume from {}", addr);
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AgentSetAppearance message (22)
    async fn handle_agent_setappearance(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AgentSetAppearance from {}", addr);
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AgentIsNowWearing message (23)
    async fn handle_agent_is_now_wearing(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AgentIsNowWearing from {}", addr);
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle RequestRegionInfo message (24)
    async fn handle_request_region_info(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("RequestRegionInfo from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send RegionInfo response
        let region_info = Self::create_region_info_packet(circuit_code)?;
        socket.send_to(&region_info, addr).await?;
        tracing::debug!("Sent RegionInfo to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle EstateCovenantRequest message (25)
    async fn handle_estate_covenant_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("EstateCovenantRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send EstateCovenant response
        let estate_covenant = Self::create_estate_covenant_packet(circuit_code)?;
        socket.send_to(&estate_covenant, addr).await?;
        tracing::debug!("Sent EstateCovenant to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle RequestGodlikePowers message (26)
    async fn handle_request_godlike_powers(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("RequestGodlikePowers from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send GrantGodlikePowers response (deny by default)
        let godlike_powers = Self::create_grant_godlike_powers_packet(circuit_code, false)?;
        socket.send_to(&godlike_powers, addr).await?;
        tracing::debug!("Sent GrantGodlikePowers to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle GodlikeMessage message (27)
    async fn handle_godlike_message(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("GodlikeMessage from {}", addr);
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle EconomyDataRequest message (28)
    async fn handle_economy_data_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("EconomyDataRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send EconomyData response
        let economy_data = Self::create_economy_data_packet(circuit_code)?;
        socket.send_to(&economy_data, addr).await?;
        tracing::debug!("Sent EconomyData to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AvatarPropertiesRequest message (29)
    async fn handle_avatar_properties_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AvatarPropertiesRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send AvatarPropertiesReply response
        let avatar_properties = Self::create_avatar_properties_reply_packet(circuit_code)?;
        socket.send_to(&avatar_properties, addr).await?;
        tracing::debug!("Sent AvatarPropertiesReply to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AvatarInterestsRequest message (30)
    async fn handle_avatar_interests_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AvatarInterestsRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send AvatarInterestsReply response
        let avatar_interests = Self::create_avatar_interests_reply_packet(circuit_code)?;
        socket.send_to(&avatar_interests, addr).await?;
        tracing::debug!("Sent AvatarInterestsReply to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AvatarGroupsRequest message (31)
    async fn handle_avatar_groups_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AvatarGroupsRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send AvatarGroupsReply response
        let avatar_groups = Self::create_avatar_groups_reply_packet(circuit_code)?;
        socket.send_to(&avatar_groups, addr).await?;
        tracing::debug!("Sent AvatarGroupsReply to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AvatarPicksRequest message (32)
    async fn handle_avatar_picks_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AvatarPicksRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send AvatarPicksReply response
        let avatar_picks = Self::create_avatar_picks_reply_packet(circuit_code)?;
        socket.send_to(&avatar_picks, addr).await?;
        tracing::debug!("Sent AvatarPicksReply to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Handle AvatarClassifiedsRequest message (33)
    async fn handle_avatar_classifieds_request(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        tracing::debug!("AvatarClassifiedsRequest from {}", addr);
        
        // Get client circuit code
        let circuit_code = {
            let clients_lock = clients.read().await;
            clients_lock.get(&addr)
                .map(|client| client.circuit_code)
                .unwrap_or(0)
        };

        // Send AvatarClassifiedsReply response
        let avatar_classifieds = Self::create_avatar_classifieds_reply_packet(circuit_code)?;
        socket.send_to(&avatar_classifieds, addr).await?;
        tracing::debug!("Sent AvatarClassifiedsReply to {}", addr);
        
        // Send acknowledgment
        Self::send_packet_ack(clients, addr, sequence).await?;
        Ok(())
    }

    /// Send packet acknowledgment
    async fn send_packet_ack(
        clients: &Arc<RwLock<HashMap<SocketAddr, LLUDPClient>>>,
        addr: SocketAddr,
        sequence: u32,
    ) -> Result<()> {
        let mut clients_lock = clients.write().await;
        if let Some(client) = clients_lock.get_mut(&addr) {
            client.acknowledged_packets.push(sequence);
        }
        Ok(())
    }

    /// Create RegionInfo packet
    fn create_region_info_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x20); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for RegionInfo (148)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&148u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        // RegionInfo block
        packet.push(0x01); // Block count
        let sim_name = "OpenSim Next";
        packet.push(sim_name.len() as u8);
        packet.extend_from_slice(sim_name.as_bytes());
        
        let estate_id = 1u32;
        packet.extend_from_slice(&estate_id.to_le_bytes());
        
        let parent_estate_id = 1u32;
        packet.extend_from_slice(&parent_estate_id.to_le_bytes());
        
        let region_flags = 0x80003E80u32; // Standard region flags
        packet.extend_from_slice(&region_flags.to_le_bytes());
        
        let sim_access = 13u8; // PG rating
        packet.push(sim_access);
        
        let max_agents = 40u8;
        packet.push(max_agents);
        
        let billable_factor = 1.0f32;
        packet.extend_from_slice(&billable_factor.to_le_bytes());
        
        let object_bonus_factor = 1.0f32;
        packet.extend_from_slice(&object_bonus_factor.to_le_bytes());
        
        let water_height = 20.0f32;
        packet.extend_from_slice(&water_height.to_le_bytes());
        
        let terrain_raise_limit = 4.0f32;
        packet.extend_from_slice(&terrain_raise_limit.to_le_bytes());
        
        let terrain_lower_limit = -4.0f32;
        packet.extend_from_slice(&terrain_lower_limit.to_le_bytes());
        
        let price_per_meter = 1i32;
        packet.extend_from_slice(&price_per_meter.to_le_bytes());
        
        let redirect_grid_x = 0i32;
        packet.extend_from_slice(&redirect_grid_x.to_le_bytes());
        
        let redirect_grid_y = 0i32;
        packet.extend_from_slice(&redirect_grid_y.to_le_bytes());
        
        let use_estate_sun = 1u8;
        packet.push(use_estate_sun);
        
        Ok(packet)
    }

    /// Create EstateCovenant packet
    fn create_estate_covenant_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x21); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for EstateCovenant (145)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&145u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        // Data block
        packet.push(0x01); // Block count
        let covenant_id = [0u8; 16]; // CovenantID (null UUID - no covenant)
        packet.extend_from_slice(&covenant_id);
        
        let covenant_timestamp = 0u32; // CovenantTimestamp
        packet.extend_from_slice(&covenant_timestamp.to_le_bytes());
        
        let estate_name = "OpenSim Next Estate";
        packet.push(estate_name.len() as u8);
        packet.extend_from_slice(estate_name.as_bytes());
        
        let estate_owner_id = [0u8; 16]; // EstateOwnerID
        packet.extend_from_slice(&estate_owner_id);
        
        Ok(packet)
    }

    /// Create GrantGodlikePowers packet
    fn create_grant_godlike_powers_packet(circuit_code: u32, granted: bool) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x22); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for GrantGodlikePowers (161)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&161u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let session_id = [0u8; 16]; // SessionID
        packet.extend_from_slice(&session_id);
        
        // RequestBlock
        packet.push(0x01); // Block count
        let god_level = if granted { 255u8 } else { 0u8 }; // GodLevel
        packet.push(god_level);
        
        let token = [0u8; 16]; // Token
        packet.extend_from_slice(&token);
        
        Ok(packet)
    }

    /// Create EconomyData packet
    fn create_economy_data_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x23); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for EconomyData (130)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&130u16.to_be_bytes()); // Message number
        
        // Info block
        packet.push(0x01); // Block count
        let object_capacity = 15000i32;
        packet.extend_from_slice(&object_capacity.to_le_bytes());
        
        let object_count = 0i32;
        packet.extend_from_slice(&object_count.to_le_bytes());
        
        let prim_capacity = 15000i32;
        packet.extend_from_slice(&prim_capacity.to_le_bytes());
        
        let prim_count = 0i32;
        packet.extend_from_slice(&prim_count.to_le_bytes());
        
        let public_urls = 1000i32;
        packet.extend_from_slice(&public_urls.to_le_bytes());
        
        let buy_price = 1i32;
        packet.extend_from_slice(&buy_price.to_le_bytes());
        
        let rent_price = 1i32;
        packet.extend_from_slice(&rent_price.to_le_bytes());
        
        let area = 65536i32; // 256x256 region
        packet.extend_from_slice(&area.to_le_bytes());
        
        let estate_id = 1i32;
        packet.extend_from_slice(&estate_id.to_le_bytes());
        
        let parent_estate_id = 1i32;
        packet.extend_from_slice(&parent_estate_id.to_le_bytes());
        
        let mature_publish = 0i32;
        packet.extend_from_slice(&mature_publish.to_le_bytes());
        
        let snapshot_id = [0u8; 16]; // SnapshotID
        packet.extend_from_slice(&snapshot_id);
        
        let product_sku = [0u8; 16]; // ProductSKU
        packet.extend_from_slice(&product_sku);
        
        let product_name = "OpenSim Next";
        packet.push(product_name.len() as u8);
        packet.extend_from_slice(product_name.as_bytes());
        
        Ok(packet)
    }

    /// Create AvatarPropertiesReply packet
    fn create_avatar_properties_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x24); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarPropertiesReply (237)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&237u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let avatar_id = [0u8; 16]; // AvatarID
        packet.extend_from_slice(&avatar_id);
        
        // PropertiesData block
        packet.push(0x01); // Block count
        let image_id = [0u8; 16]; // ImageID
        packet.extend_from_slice(&image_id);
        
        let fl_image_id = [0u8; 16]; // FLImageID
        packet.extend_from_slice(&fl_image_id);
        
        let partner_id = [0u8; 16]; // PartnerID
        packet.extend_from_slice(&partner_id);
        
        let about_text = "OpenSim Next User";
        packet.push(about_text.len() as u8);
        packet.extend_from_slice(about_text.as_bytes());
        
        let fl_about_text = "";
        packet.push(fl_about_text.len() as u8);
        packet.extend_from_slice(fl_about_text.as_bytes());
        
        let born_on = "1/1/2000";
        packet.push(born_on.len() as u8);
        packet.extend_from_slice(born_on.as_bytes());
        
        let profile_url = "";
        packet.push(profile_url.len() as u8);
        packet.extend_from_slice(profile_url.as_bytes());
        
        let charter_member = [0u8; 16]; // CharterMember
        packet.extend_from_slice(&charter_member);
        
        let flags = 0u32; // Flags
        packet.extend_from_slice(&flags.to_le_bytes());
        
        Ok(packet)
    }

    /// Create AvatarInterestsReply packet
    fn create_avatar_interests_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x25); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarInterestsReply (238)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&238u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let avatar_id = [0u8; 16]; // AvatarID
        packet.extend_from_slice(&avatar_id);
        
        // PropertiesData block
        packet.push(0x01); // Block count
        let want_to_mask = 0u32; // WantToMask
        packet.extend_from_slice(&want_to_mask.to_le_bytes());
        
        let want_to_text = "";
        packet.push(want_to_text.len() as u8);
        packet.extend_from_slice(want_to_text.as_bytes());
        
        let skills_mask = 0u32; // SkillsMask
        packet.extend_from_slice(&skills_mask.to_le_bytes());
        
        let skills_text = "";
        packet.push(skills_text.len() as u8);
        packet.extend_from_slice(skills_text.as_bytes());
        
        let languages_text = "English";
        packet.push(languages_text.len() as u8);
        packet.extend_from_slice(languages_text.as_bytes());
        
        Ok(packet)
    }

    /// Create AvatarGroupsReply packet
    fn create_avatar_groups_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x26); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarGroupsReply (172)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&172u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let avatar_id = [0u8; 16]; // AvatarID
        packet.extend_from_slice(&avatar_id);
        
        // GroupData block (no groups)
        packet.push(0x00); // Block count (no groups)
        
        Ok(packet)
    }

    /// Create AvatarPicksReply packet
    fn create_avatar_picks_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x27); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarPicksReply (173)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&173u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let target_id = [0u8; 16]; // TargetID
        packet.extend_from_slice(&target_id);
        
        // Data block (no picks)
        packet.push(0x00); // Block count (no picks)
        
        Ok(packet)
    }

    /// Create AvatarClassifiedsReply packet
    fn create_avatar_classifieds_reply_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x28); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for AvatarClassifiedsReply (174)
        packet.push(0xFF); // Extended message format
        packet.push(0xFF); // Extended message format
        packet.extend_from_slice(&174u16.to_be_bytes()); // Message number
        
        // AgentData block
        packet.push(0x01); // Block count
        let agent_id = [0u8; 16]; // AgentID
        packet.extend_from_slice(&agent_id);
        
        let target_id = [0u8; 16]; // TargetID
        packet.extend_from_slice(&target_id);
        
        // Data block (no classifieds)
        packet.push(0x00); // Block count (no classifieds)
        
        Ok(packet)
    }

    /// Create Avatar ObjectUpdate packet - CRITICAL for login completion
    fn create_avatar_object_update_packet(circuit_code: u32) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        
        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x29); // Sequence number (low)
        packet.push(0x00); // Extra header length
        
        // Message number for ObjectUpdate (12)
        packet.push(12u8); // ObjectUpdate is a short message
        
        // RegionData block
        packet.push(0x01); // Block count
        let region_handle = DEFAULT_REGION_HANDLE;
        packet.extend_from_slice(&region_handle.to_le_bytes());
        let timestamp = 0u16; // TimeDilation timestamp
        packet.extend_from_slice(&timestamp.to_le_bytes());
        
        // ObjectData block for avatar
        packet.push(0x01); // Block count (one avatar object)
        
        // Avatar-specific object data
        let local_id = 1u32; // Local ID for avatar
        packet.extend_from_slice(&local_id.to_le_bytes());
        
        let update_flags = 0x40u8; // Avatar update flags
        packet.push(update_flags);
        
        let path_curve = 16u8; // Avatar path curve
        packet.push(path_curve);
        
        let profile_curve = 1u8; // Avatar profile curve  
        packet.push(profile_curve);
        
        let path_begin = 0u8;
        packet.push(path_begin);
        
        let path_end = 0u8;
        packet.push(path_end);
        
        let path_scale_x = 100u8;
        packet.push(path_scale_x);
        
        let path_scale_y = 100u8;
        packet.push(path_scale_y);
        
        let path_shear_x = 0u8;
        packet.push(path_shear_x);
        
        let path_shear_y = 0u8;
        packet.push(path_shear_y);
        
        let path_twist = 0i8;
        packet.push(path_twist as u8);
        
        let path_twist_begin = 0i8;
        packet.push(path_twist_begin as u8);
        
        let path_radius_offset = 0i8;
        packet.push(path_radius_offset as u8);
        
        let path_taper_x = 0i8;
        packet.push(path_taper_x as u8);
        
        let path_taper_y = 0i8;
        packet.push(path_taper_y as u8);
        
        let path_revolutions = 0u8;
        packet.push(path_revolutions);
        
        let path_skew = 0i8;
        packet.push(path_skew as u8);
        
        let profile_begin = 0u8;
        packet.push(profile_begin);
        
        let profile_end = 0u8;
        packet.push(profile_end);
        
        let profile_hollow = 0u8;
        packet.push(profile_hollow);
        
        // Avatar position (spawn at 128, 128, 25)
        let position_x = 128.0f32;
        packet.extend_from_slice(&position_x.to_le_bytes());
        
        let position_y = 128.0f32;
        packet.extend_from_slice(&position_y.to_le_bytes());
        
        let position_z = 25.0f32;
        packet.extend_from_slice(&position_z.to_le_bytes());
        
        // Avatar rotation (facing forward)
        let rotation_x = 0.0f32;
        packet.extend_from_slice(&rotation_x.to_le_bytes());
        
        let rotation_y = 0.0f32;
        packet.extend_from_slice(&rotation_y.to_le_bytes());
        
        let rotation_z = 0.0f32;
        packet.extend_from_slice(&rotation_z.to_le_bytes());
        
        let rotation_w = 1.0f32;
        packet.extend_from_slice(&rotation_w.to_le_bytes());
        
        // Avatar velocity (stationary)
        let velocity_x = 0.0f32;
        packet.extend_from_slice(&velocity_x.to_le_bytes());
        
        let velocity_y = 0.0f32;
        packet.extend_from_slice(&velocity_y.to_le_bytes());
        
        let velocity_z = 0.0f32;
        packet.extend_from_slice(&velocity_z.to_le_bytes());
        
        // Avatar angular velocity (not rotating)
        let angular_velocity_x = 0.0f32;
        packet.extend_from_slice(&angular_velocity_x.to_le_bytes());
        
        let angular_velocity_y = 0.0f32;
        packet.extend_from_slice(&angular_velocity_y.to_le_bytes());
        
        let angular_velocity_z = 0.0f32;
        packet.extend_from_slice(&angular_velocity_z.to_le_bytes());
        
        // Avatar scale (normal human size)
        let scale_x = 1.0f32;
        packet.extend_from_slice(&scale_x.to_le_bytes());
        
        let scale_y = 1.0f32;
        packet.extend_from_slice(&scale_y.to_le_bytes());
        
        let scale_z = 1.0f32;
        packet.extend_from_slice(&scale_z.to_le_bytes());
        
        // Additional avatar data
        let parent_id = 0u32; // No parent object
        packet.extend_from_slice(&parent_id.to_le_bytes());
        
        let update_type = 0u8; // Full update
        packet.push(update_type);
        
        Ok(packet)
    }

    /// Create AgentDataUpdate packet - FINAL login completion signal
    /// Parameters:
    /// - agent_id: The UUID of the agent
    /// - first_name: Agent's first name
    /// - last_name: Agent's last name
    /// - group_title: Optional group title (empty string if none)
    /// - group_name: Optional group name (empty string if none)
    /// - group_id: Optional group UUID (zero UUID if none)
    fn create_agent_data_update_packet(
        agent_id: uuid::Uuid,
        first_name: &str,
        last_name: &str,
        group_title: &str,
        group_name: &str,
        group_id: uuid::Uuid,
    ) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // LLUDP header
        packet.push(0x40); // Packet flags (reliable)
        packet.push(0x00); // Sequence number (high)
        packet.push(0x00); // Sequence number (mid)
        packet.push(0x2A); // Sequence number (low)
        packet.push(0x00); // Extra header length

        // Message number for AgentDataUpdate (20)
        packet.push(20u8); // AgentDataUpdate is a short message

        // AgentData block
        packet.push(0x01); // Block count

        // Agent ID - use actual agent UUID
        packet.extend_from_slice(agent_id.as_bytes());

        // First name
        packet.push(first_name.len() as u8);
        packet.extend_from_slice(first_name.as_bytes());

        // Last name
        packet.push(last_name.len() as u8);
        packet.extend_from_slice(last_name.as_bytes());

        // Group title
        packet.push(group_title.len() as u8);
        packet.extend_from_slice(group_title.as_bytes());

        // Group powers
        let group_powers = 0u64; // No group powers (TODO: implement group powers)
        packet.extend_from_slice(&group_powers.to_le_bytes());

        // Group name
        packet.push(group_name.len() as u8);
        packet.extend_from_slice(group_name.as_bytes());

        // Group ID
        packet.extend_from_slice(group_id.as_bytes());

        // Group insignia ID (TODO: implement group insignia)
        let group_insignia_id = uuid::Uuid::nil();
        packet.extend_from_slice(group_insignia_id.as_bytes());

        // Avatar age (TODO: calculate from account creation date)
        let avatar_age = 0u32;
        packet.extend_from_slice(&avatar_age.to_le_bytes());

        Ok(packet)
    }

}