// Test comment to diagnose editing issues
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, warn};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use std::io::{Cursor, Read, Write};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    network::{
        security::SecurityManager,
        session::{Session, SessionManager},
        handlers::avatar::AvatarHandler,
        handlers::inventory::InventoryHandler,
        handlers::asset::AssetHandler,
        handlers::login::LoginHandler,
    },
    region::{
        avatar::appearance::{Appearance, VisualParams},
        RegionManager,
    },
    state::StateManager,
    database::user_accounts::UserAccountDatabase,
};

/// LLSD (Linden Lab Structured Data) value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum LLSDValue {
    #[serde(rename = "undef")]
    Undefined,
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "integer")]
    Integer(i32),
    #[serde(rename = "real")]
    Real(f64),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "uuid")]
    UUID(Uuid),
    #[serde(rename = "date")]
    Date(String), // Using string to represent date for simplicity
    #[serde(rename = "uri")]
    URI(String),
    #[serde(rename = "binary")]
    Binary(Vec<u8>),
    #[serde(rename = "map")]
    Map(HashMap<String, LLSDValue>),
    #[serde(rename = "array")]
    Array(Vec<LLSDValue>),
}

impl Default for LLSDValue {
    fn default() -> Self {
        LLSDValue::Undefined
    }
}

// Add convenience methods for type checking and conversion
impl LLSDValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            LLSDValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self {
            LLSDValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            LLSDValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_uuid(&self) -> Option<&Uuid> {
        match self {
            LLSDValue::UUID(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, LLSDValue>> {
        match self {
            LLSDValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<LLSDValue>> {
        match self {
            LLSDValue::Array(a) => Some(a),
            _ => None,
        }
    }
}

/// Represents a full LLSD message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLSDMessage {
    pub message_type: String,
    pub data: LLSDValue,
    pub session_id: Option<Uuid>,
    pub sequence: Option<u32>,
}

impl LLSDMessage {
    pub fn new(message_type: String, data: LLSDValue) -> Self {
        Self {
            message_type,
            data,
            session_id: None,
            sequence: None,
        }
    }

    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn get_message_name(&self) -> &str {
        &self.message_type
    }

    pub fn from_bytes(data: &[u8], format: LLSDFormat) -> Result<Self> {
        match format {
            LLSDFormat::Xml => {
                let s = std::str::from_utf8(data)
                    .map_err(|e| anyhow!("Invalid UTF-8 in XML data: {}", e))?;
                let value: LLSDValue = from_str(s)
                    .map_err(|e| anyhow!("Failed to parse XML: {}", e))?;
                Self::from_llsd_value(value)
            }
            LLSDFormat::Binary => {
                let mut cursor = Cursor::new(data);
                let value = LLSDParser::parse_binary_value(&mut cursor)
                    .map_err(|e| anyhow!("Failed to parse binary LLSD: {}", e))?;
                Self::from_llsd_value(value)
            }
        }
    }

    pub fn to_bytes(&self, format: LLSDFormat) -> Result<Vec<u8>> {
        let value = self.to_llsd_value();
        match format {
            LLSDFormat::Xml => {
                let s = to_string(&value)
                    .map_err(|e| anyhow!("Failed to serialize to XML: {}", e))?;
                Ok(s.into_bytes())
            }
            LLSDFormat::Binary => {
                let mut buffer = Vec::new();
                LLSDParser::serialize_binary_value(&mut buffer, &value)
                    .map_err(|e| anyhow!("Failed to serialize to binary: {}", e))?;
                Ok(buffer)
            }
        }
    }

    fn from_llsd_value(value: LLSDValue) -> Result<Self> {
        if let LLSDValue::Map(mut map) = value {
            let message_type = match map.remove("message_type") {
                Some(LLSDValue::String(s)) => s,
                _ => return Err(anyhow!("Missing or invalid 'message_type' in LLSD message")),
            };
            let data = map.remove("data").unwrap_or(LLSDValue::Undefined);
            let session_id = match map.remove("session_id") {
                Some(LLSDValue::UUID(u)) => Some(u),
                _ => None,
            };
            let sequence = match map.remove("sequence") {
                Some(LLSDValue::Integer(i)) => Some(i as u32),
                _ => None,
            };
            Ok(Self { message_type, data, session_id, sequence })
        } else {
            Err(anyhow!("LLSD message must be a map"))
        }
    }

    fn to_llsd_value(&self) -> LLSDValue {
        let mut map = HashMap::new();
        map.insert("message_type".to_string(), LLSDValue::String(self.message_type.clone()));
        map.insert("data".to_string(), self.data.clone());
        if let Some(sid) = self.session_id {
            map.insert("session_id".to_string(), LLSDValue::UUID(sid));
        }
        if let Some(seq) = self.sequence {
            map.insert("sequence".to_string(), LLSDValue::Integer(seq as i32));
        }
        LLSDValue::Map(map)
    }
}

// Handler registration trait for type safety
pub trait LLSDHandler: Send + Sync {
    fn message_types(&self) -> Vec<&'static str>;
    fn handle<'a>(&'a self, message: LLSDMessage, context: HandlerContext<'a>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<LLSDMessage>>> + Send + 'a>>;
}

// Context struct to pass dependencies to handlers
pub struct HandlerContext<'a> {
    pub session: Arc<RwLock<Session>>,
    pub security_manager: Arc<SecurityManager>,
    pub session_manager: Arc<SessionManager>,
    pub region_manager: Arc<RegionManager>,
    pub state_manager: Arc<StateManager>,
    pub asset_manager: Arc<crate::asset::AssetManager>,
    pub user_account_database: Arc<UserAccountDatabase>,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> HandlerContext<'a> {
    pub fn new(
        session: Arc<RwLock<Session>>,
        security_manager: Arc<SecurityManager>,
        session_manager: Arc<SessionManager>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<crate::asset::AssetManager>,
        user_account_database: Arc<UserAccountDatabase>,
    ) -> Self {
        Self {
            session,
            security_manager,
            session_manager,
            region_manager,
            state_manager,
            asset_manager,
            user_account_database,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Handles incoming LLSD messages with proper handler registration
pub struct LLSDMessageHandler {
    handlers: HashMap<String, Arc<dyn LLSDHandler>>,
    asset_handler: AssetHandler,
    avatar_handler: AvatarHandler,
    inventory_handler: InventoryHandler,
    login_handler: Option<Arc<LoginHandler>>,
}

impl Default for LLSDMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LLSDMessageHandler {
    /// Creates a new LLSDMessageHandler
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            asset_handler: AssetHandler::default(),
            avatar_handler: AvatarHandler::default(),
            inventory_handler: InventoryHandler::default(),
            login_handler: None,
        }
    }
    
    /// Sets the login handler
    pub fn set_login_handler(&mut self, login_handler: Arc<LoginHandler>) {
        self.login_handler = Some(login_handler);
    }

    /// Register a handler for specific message types
    pub fn register_handler<H: LLSDHandler + 'static>(&mut self, handler: H) {
        let message_types = handler.message_types();
        let arc_handler = Arc::new(handler);
        for msg_type in message_types {
            self.handlers.insert(msg_type.to_string(), arc_handler.clone());
        }
    }

    /// Handles an incoming LLSD message and returns an optional response
    pub async fn handle_message(
        &self,
        message: LLSDMessage,
        session: Arc<RwLock<Session>>,
        security_manager: Arc<SecurityManager>,
        session_manager: Arc<SessionManager>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<crate::asset::AssetManager>,
        user_account_database: Arc<UserAccountDatabase>,
    ) -> Result<Option<LLSDMessage>> {
        let message_name = message.get_message_name();
        
        // Check for registered handlers first
        if let Some(handler) = self.handlers.get(message_name) {
            let context = HandlerContext::new(
                session.clone(),
                security_manager.clone(),
                session_manager.clone(),
                region_manager.clone(),
                state_manager.clone(),
                asset_manager.clone(),
                user_account_database.clone(),
            );
            return handler.handle(message, context).await;
        }

        // Fall back to built-in handlers for backward compatibility
        match message_name {
            "LoginToSim" | "login_to_simulator" => {
                if let Some(login_handler) = &self.login_handler {
                    self.handle_login_request(message, session.clone(), login_handler.clone()).await
                } else {
                    self.handle_login(message, session.clone(), security_manager, session_manager).await
                }
            },
            "UpdateAvatarAppearance" => {
                let session_guard = session.read().await;
                if let Some(data_map) = message.data.as_map() {
                    let appearance = self.parse_appearance_data(data_map)?;
                    self.avatar_handler
                        .handle_update_appearance(session_guard.clone_session(), region_manager, appearance)
                        .await
                        .map_err(|e| anyhow!("Avatar appearance update failed: {}", e))?;
                }
                Ok(None)
            }
            "ViewerStats" => {
                info!("Received ViewerStats message");
                Ok(None)
            }
            "AgentWearablesRequest" => {
                let result = self.avatar_handler
                    .handle_agent_wearables_request(session, region_manager, asset_manager)
                    .await
                    .map_err(|e| anyhow!("Agent wearables request failed: {}", e))?;
                
                // Create response message
                let response = LLSDMessage::new("AgentWearablesResponse".to_string(), result);
                Ok(Some(response))
            }
            "FetchInventory" => {
                self.inventory_handler
                    .handle_fetch_inventory(session, state_manager)
                    .await
                    .map_err(|e| anyhow!("Inventory fetch failed: {}", e))
            }
            "UploadAsset" => {
                info!("Asset upload request received - implementation pending");
                Ok(None)
            }
            _ => {
                warn!("Unhandled LLSD message type: {}", message_name);
                Ok(None)
            }
        }
    }

    /// Handles login request using the new LoginHandler
    async fn handle_login_request(
        &self,
        message: LLSDMessage,
        session: Arc<RwLock<Session>>,
        login_handler: Arc<LoginHandler>,
    ) -> Result<Option<LLSDMessage>> {
        info!("Handling login request with new LoginHandler");
        
        // Parse the login request from LLSD message
        let login_request = self.parse_login_request(&message)?;
        
        // Get client IP from session
        let client_ip = {
            let session_guard = session.read().await;
            session_guard.client_addr
        };
        
        // Process login request
        match login_handler.handle_login_request(login_request, client_ip).await {
            Ok(login_response) => {
                // Convert LoginResponse to LLSD message
                let response_data = self.login_response_to_llsd(&login_response)?;
                
                // Update session if login was successful
                if login_response.login == "true" {
                    if let (Some(session_id), Some(agent_id)) = (&login_response.session_id, &login_response.agent_id) {
                        let mut session_guard = session.write().await;
                        session_guard.session_id = session_id.to_string();
                        session_guard.agent_id = *agent_id;
                        session_guard.user_id = format!("{} {}", 
                            login_response.first_name.as_deref().unwrap_or(""),
                            login_response.last_name.as_deref().unwrap_or(""));
                        info!("Login successful for user: {} (Agent: {})", session_guard.user_id, agent_id);
                    }
                }
                
                Ok(Some(LLSDMessage {
                    message_type: "LoginResponse".to_string(),
                    data: response_data,
                    session_id: login_response.session_id,
                    sequence: message.sequence,
                }))
            },
            Err(e) => {
                warn!("Login failed: {}", e);
                let error_response = LLSDValue::Map(HashMap::from([
                    ("login".to_string(), LLSDValue::String("false".to_string())),
                    ("message".to_string(), LLSDValue::String("Login failed".to_string())),
                    ("reason".to_string(), LLSDValue::String(e.to_string())),
                ]));
                
                Ok(Some(LLSDMessage {
                    message_type: "LoginResponse".to_string(),
                    data: error_response,
                    session_id: None,
                    sequence: message.sequence,
                }))
            }
        }
    }
    
    /// Parses an LLSD message into a LoginRequest
    fn parse_login_request(&self, message: &LLSDMessage) -> Result<crate::network::handlers::login::LoginRequest> {
        use crate::network::handlers::login::LoginRequest;
        
        let data_map = message.data.as_map()
            .ok_or_else(|| anyhow!("Login message must be a map"))?;
        
        let method = data_map.get("method")
            .and_then(|v| v.as_string())
            .unwrap_or("login_to_simulator")
            .to_string();
            
        let first = data_map.get("first")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let last = data_map.get("last")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let passwd = data_map.get("passwd")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let start = data_map.get("start")
            .and_then(|v| v.as_string())
            .unwrap_or("home")
            .to_string();
            
        let channel = data_map.get("channel")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown Viewer")
            .to_string();
            
        let version = data_map.get("version")
            .and_then(|v| v.as_string())
            .unwrap_or("0.0.0")
            .to_string();
            
        let platform = data_map.get("platform")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown")
            .to_string();
            
        let mac = data_map.get("mac")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let id0 = data_map.get("id0")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let agree_to_tos = data_map.get("agree_to_tos")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);
            
        let read_critical = data_map.get("read_critical")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);
            
        let viewer_digest = data_map.get("viewer_digest")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
            
        let options = data_map.get("options")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|item| item.as_string())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_else(Vec::new);
        
        Ok(LoginRequest {
            method,
            first,
            last,
            passwd,
            start,
            channel,
            version,
            platform,
            mac,
            id0,
            agree_to_tos,
            read_critical,
            viewer_digest,
            options,
            // Optional fields with default values
            address_size: None,
            extended_errors: None,
            host_id: None,
            mfa_hash: None,
            token: None,
            request_creds: None,
            skipoptional: None,
            inventory_host: None,
            want_to_login: None,
        })
    }
    
    /// Converts a LoginResponse to LLSD format
    fn login_response_to_llsd(&self, response: &crate::network::handlers::login::LoginResponse) -> Result<LLSDValue> {
        let mut map = HashMap::new();
        
        map.insert("login".to_string(), LLSDValue::String(response.login.clone()));
        map.insert("message".to_string(), LLSDValue::String(response.message.clone()));
        map.insert("reason".to_string(), LLSDValue::String(response.reason.clone()));
        
        if response.login == "true" {
            // Include success fields
            if let Some(session_id) = response.session_id {
                map.insert("session_id".to_string(), LLSDValue::UUID(session_id));
            }
            if let Some(secure_session_id) = response.secure_session_id {
                map.insert("secure_session_id".to_string(), LLSDValue::UUID(secure_session_id));
            }
            if let Some(agent_id) = response.agent_id {
                map.insert("agent_id".to_string(), LLSDValue::UUID(agent_id));
            }
            if let Some(first_name) = &response.first_name {
                map.insert("first_name".to_string(), LLSDValue::String(first_name.clone()));
            }
            if let Some(last_name) = &response.last_name {
                map.insert("last_name".to_string(), LLSDValue::String(last_name.clone()));
            }
            if let Some(sim_ip) = &response.sim_ip {
                map.insert("sim_ip".to_string(), LLSDValue::String(sim_ip.clone()));
            }
            if let Some(sim_port) = response.sim_port {
                map.insert("sim_port".to_string(), LLSDValue::Integer(sim_port as i32));
            }
            if let Some(region_x) = response.region_x {
                map.insert("region_x".to_string(), LLSDValue::Integer(region_x as i32));
            }
            if let Some(region_y) = response.region_y {
                map.insert("region_y".to_string(), LLSDValue::Integer(region_y as i32));
            }
            if let Some(look_at) = response.look_at {
                map.insert("look_at".to_string(), LLSDValue::String(format!("[{},{},{}]", look_at[0], look_at[1], look_at[2])));
            }
            if let Some(seconds) = response.seconds_since_epoch {
                map.insert("seconds_since_epoch".to_string(), LLSDValue::Integer(seconds as i32));
            }
            
            // Add empty inventory for now
            map.insert("inventory-root".to_string(), LLSDValue::Map(HashMap::new()));
            map.insert("inventory-skeleton".to_string(), LLSDValue::Map(HashMap::new()));
        }
        
        Ok(LLSDValue::Map(map))
    }

    async fn handle_login(
        &self,
        message: LLSDMessage,
        session: Arc<RwLock<Session>>,
        security_manager: Arc<SecurityManager>,
        session_manager: Arc<SessionManager>,
    ) -> Result<Option<LLSDMessage>> {
        info!("Handling LoginToSim message");

        let (username, password) = match message.data.as_map() {
            Some(map) => {
                let user = map.get("user")
                    .and_then(|v| v.as_string())
                    .unwrap_or("");
                let pass = map.get("password")
                    .and_then(|v| v.as_string())
                    .unwrap_or("");
                (user, pass)
            },
            None => return Err(anyhow!("Invalid login message format - expected map")),
        };

        if username.is_empty() || password.is_empty() {
            return Err(anyhow!("Username and password are required"));
        }

        match security_manager.authenticate(username, password).await {
            Ok(auth_token) => {
                let mut session_guard = session.write().await;
                session_guard.user_id = username.to_string();
                session_manager.update_session((*session_guard).clone()).await;

                info!("User '{}' successfully logged in with session ID: {}", username, session_guard.session_id);

                let response_data = LLSDValue::Map(HashMap::from([
                    ("success".to_string(), LLSDValue::Boolean(true)),
                    ("session_id".to_string(), LLSDValue::String(session_guard.session_id.clone())),
                    ("auth_token".to_string(), LLSDValue::String(auth_token)),
                    ("message".to_string(), LLSDValue::String("Login successful".to_string())),
                ]));

                Ok(Some(LLSDMessage {
                    message_type: "LoginToSimResponse".to_string(),
                    data: response_data,
                    session_id: Uuid::parse_str(&session_guard.session_id).ok(),
                    sequence: message.sequence,
                }))
            }
            Err(e) => {
                warn!("Authentication failed for user '{}': {}", username, e);
                let response_data = LLSDValue::Map(HashMap::from([
                    ("success".to_string(), LLSDValue::Boolean(false)),
                    ("message".to_string(), LLSDValue::String(format!("Authentication failed: {}", e))),
                ]));
                Ok(Some(LLSDMessage {
                    message_type: "LoginToSimResponse".to_string(),
                    data: response_data,
                    session_id: None,
                    sequence: message.sequence,
                }))
            }
        }
    }

    fn parse_appearance_data(&self, data: &HashMap<String, LLSDValue>) -> Result<Appearance> {
        let serial = data.get("serial")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;

        let visual_params = if let Some(vp_map) = data.get("visual_params").and_then(|v| v.as_map()) {
            let mut params = HashMap::new();
            for (k, v) in vp_map {
                if let (Ok(key), Some(LLSDValue::Real(val))) = (k.parse::<u32>(), Some(v)) {
                    params.insert(key, *val as f32);
                }
            }
            VisualParams { params }
        } else {
            VisualParams::default()
        };

        Ok(Appearance {
            serial,
            visual_params,
            ..Default::default()
        })
    }
}

/// Represents the format of an LLSD message
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LLSDFormat {
    Xml,
    Binary,
}

struct LLSDParser;

impl LLSDParser {
    fn parse_binary_value<R: Read>(reader: &mut R) -> Result<LLSDValue> {
        let mut type_char = [0u8; 1];
        reader.read_exact(&mut type_char)
            .map_err(|e| anyhow!("Failed to read type character: {}", e))?;
        
        match type_char[0] as char {
            '!' => Ok(LLSDValue::Undefined),
            '1' => Ok(LLSDValue::Boolean(true)),
            '0' => Ok(LLSDValue::Boolean(false)),
            'i' => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)
                    .map_err(|e| anyhow!("Failed to read integer: {}", e))?;
                Ok(LLSDValue::Integer(i32::from_be_bytes(buf)))
            }
            'r' => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)
                    .map_err(|e| anyhow!("Failed to read real: {}", e))?;
                Ok(LLSDValue::Real(f64::from_be_bytes(buf)))
            }
            'u' => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf)
                    .map_err(|e| anyhow!("Failed to read UUID: {}", e))?;
                Ok(LLSDValue::UUID(Uuid::from_bytes(buf)))
            }
            's' => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)
                    .map_err(|e| anyhow!("Failed to read string length: {}", e))?;
                let len = u32::from_be_bytes(len_buf) as usize;
                
                if len > 1_000_000 { // Reasonable limit
                    return Err(anyhow!("String length too large: {}", len));
                }
                
                let mut str_buf = vec![0u8; len];
                reader.read_exact(&mut str_buf)
                    .map_err(|e| anyhow!("Failed to read string data: {}", e))?;
                let string = String::from_utf8(str_buf)
                    .map_err(|e| anyhow!("Invalid UTF-8 in string: {}", e))?;
                Ok(LLSDValue::String(string))
            }
            'l' => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)
                    .map_err(|e| anyhow!("Failed to read binary length: {}", e))?;
                let len = u32::from_be_bytes(len_buf) as usize;
                
                if len > 10_000_000 { // Reasonable limit for binary data
                    return Err(anyhow!("Binary data length too large: {}", len));
                }
                
                let mut bin_buf = vec![0u8; len];
                reader.read_exact(&mut bin_buf)
                    .map_err(|e| anyhow!("Failed to read binary data: {}", e))?;
                Ok(LLSDValue::Binary(bin_buf))
            }
            'd' => {
                let mut len_buf = [0u8; 8];
                reader.read_exact(&mut len_buf)
                    .map_err(|e| anyhow!("Failed to read date: {}", e))?;
                let time = f64::from_be_bytes(len_buf);
                Ok(LLSDValue::Date(time.to_string()))
            }
            '{' => {
                let mut map = HashMap::new();
                loop {
                    // Check for end of map
                    let mut next_byte = [0u8; 1];
                    match reader.read_exact(&mut next_byte) {
                        Ok(_) => {
                            if next_byte[0] == b'}' {
                                break;
                            }
                            // Put the byte back by reading it as the start of the next value
                            let mut key_data = vec![next_byte[0]];
                            
                            // Read enough bytes for a typical string length header
                            let mut temp_buf = [0u8; 4];
                            reader.read_exact(&mut temp_buf)?;
                            key_data.extend_from_slice(&temp_buf);
                            
                            // Parse string length and read the rest
                            if key_data.len() >= 5 && key_data[0] == b's' {
                                let len = u32::from_be_bytes([key_data[1], key_data[2], key_data[3], key_data[4]]) as usize;
                                if len < 1000 { // Reasonable limit
                                    let mut string_data = vec![0u8; len];
                                    reader.read_exact(&mut string_data)?;
                                    let key = String::from_utf8(string_data)
                                        .map_err(|e| anyhow!("Invalid UTF-8 in map key: {}", e))?;
                                    
                                    let value = Self::parse_binary_value(reader)?;
                                    map.insert(key, value);
                                } else {
                                    return Err(anyhow!("Map key too long: {}", len));
                                }
                            } else {
                                return Err(anyhow!("Expected string key in map"));
                            }
                        }
                        Err(_) => break,
                    }
                }
                Ok(LLSDValue::Map(map))
            }
            '[' => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)
                    .map_err(|e| anyhow!("Failed to read array length: {}", e))?;
                let len = u32::from_be_bytes(len_buf);
                
                if len > 1_000_000 { // Reasonable limit
                    return Err(anyhow!("Array length too large: {}", len));
                }
                
                let mut array = Vec::with_capacity(len as usize);
                for i in 0..len {
                    array.push(Self::parse_binary_value(reader)
                        .map_err(|e| anyhow!("Failed to parse array element {}: {}", i, e))?);
                }
                Ok(LLSDValue::Array(array))
            }
            c => Err(anyhow!("Unknown LLSD binary type character: '{}'", c)),
        }
    }

    fn serialize_binary_value<W: Write>(writer: &mut W, value: &LLSDValue) -> Result<()> {
        match value {
            LLSDValue::Undefined => writer.write_all(b"!")
                .map_err(|e| anyhow!("Failed to write undefined: {}", e))?,
            LLSDValue::Boolean(b) => writer.write_all(if *b { b"1" } else { b"0" })
                .map_err(|e| anyhow!("Failed to write boolean: {}", e))?,
            LLSDValue::Integer(i) => {
                writer.write_all(b"i")
                    .map_err(|e| anyhow!("Failed to write integer prefix: {}", e))?;
                writer.write_all(&i.to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write integer value: {}", e))?;
            }
            LLSDValue::Real(r) => {
                writer.write_all(b"r")
                    .map_err(|e| anyhow!("Failed to write real prefix: {}", e))?;
                writer.write_all(&r.to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write real value: {}", e))?;
            }
            LLSDValue::UUID(u) => {
                writer.write_all(b"u")
                    .map_err(|e| anyhow!("Failed to write UUID prefix: {}", e))?;
                writer.write_all(u.as_bytes())
                    .map_err(|e| anyhow!("Failed to write UUID value: {}", e))?;
            }
            LLSDValue::String(s) => {
                writer.write_all(b"s")
                    .map_err(|e| anyhow!("Failed to write string prefix: {}", e))?;
                writer.write_all(&(s.len() as u32).to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write string length: {}", e))?;
                writer.write_all(s.as_bytes())
                    .map_err(|e| anyhow!("Failed to write string data: {}", e))?;
            }
            LLSDValue::Binary(b) => {
                writer.write_all(b"l")
                    .map_err(|e| anyhow!("Failed to write binary prefix: {}", e))?;
                writer.write_all(&(b.len() as u32).to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write binary length: {}", e))?;
                writer.write_all(b)
                    .map_err(|e| anyhow!("Failed to write binary data: {}", e))?;
            }
            LLSDValue::Date(d) => {
                writer.write_all(b"d")
                    .map_err(|e| anyhow!("Failed to write date prefix: {}", e))?;
                let time: f64 = d.parse().unwrap_or(0.0);
                writer.write_all(&time.to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write date value: {}", e))?;
            }
            LLSDValue::Map(map) => {
                writer.write_all(b"{")
                    .map_err(|e| anyhow!("Failed to write map start: {}", e))?;
                for (k, v) in map {
                    Self::serialize_binary_value(writer, &LLSDValue::String(k.clone()))?;
                    Self::serialize_binary_value(writer, v)?;
                }
                writer.write_all(b"}")
                    .map_err(|e| anyhow!("Failed to write map end: {}", e))?;
            }
            LLSDValue::Array(arr) => {
                writer.write_all(b"[")
                    .map_err(|e| anyhow!("Failed to write array start: {}", e))?;
                writer.write_all(&(arr.len() as u32).to_be_bytes())
                    .map_err(|e| anyhow!("Failed to write array length: {}", e))?;
                for (i, v) in arr.iter().enumerate() {
                    Self::serialize_binary_value(writer, v)
                        .map_err(|e| anyhow!("Failed to serialize array element {}: {}", i, e))?;
                }
                writer.write_all(b"]")
                    .map_err(|e| anyhow!("Failed to write array end: {}", e))?;
            }
            LLSDValue::URI(uri) => {
                // Serialize URI as string for now
                Self::serialize_binary_value(writer, &LLSDValue::String(uri.clone()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llsd_value_convenience_methods() {
        let string_val = LLSDValue::String("test".to_string());
        assert_eq!(string_val.as_string(), Some("test"));
        assert_eq!(string_val.as_integer(), None);

        let int_val = LLSDValue::Integer(42);
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(int_val.as_string(), None);
    }

    #[test]
    fn test_llsd_message_builder() {
        let msg = LLSDMessage::new("Test".to_string(), LLSDValue::String("data".to_string()))
            .with_session(Uuid::new_v4())
            .with_sequence(123);
        
        assert_eq!(msg.message_type, "Test");
        assert!(msg.session_id.is_some());
        assert_eq!(msg.sequence, Some(123));
    }

    #[test]
    fn test_llsd_xml_serialization_deserialization() {
        let map_data = HashMap::from([
            ("message_type".to_string(), LLSDValue::String("TestMessage".to_string())),
            ("data".to_string(), LLSDValue::Map(HashMap::from([
                ("value1".to_string(), LLSDValue::Integer(42)),
                ("value2".to_string(), LLSDValue::String("Hello".to_string())),
            ]))),
        ]);
        let message_value = LLSDValue::Map(map_data);
        let xml_string = to_string(&message_value).unwrap();
        let deserialized: LLSDValue = from_str(&xml_string).unwrap();
        assert_eq!(message_value, deserialized);
    }

    #[test]
    fn test_llsd_binary_serialization_deserialization() {
        let map_data = HashMap::from([
           ("message_type".to_string(), LLSDValue::String("TestMessage".to_string())),
           ("data".to_string(), LLSDValue::Map(HashMap::from([
               ("value1".to_string(), LLSDValue::Integer(42)),
               ("value2".to_string(), LLSDValue::String("Hello".to_string())),
           ]))),
       ]);
       let message_value = LLSDValue::Map(map_data);
       let mut binary_data = Vec::new();
       LLSDParser::serialize_binary_value(&mut binary_data, &message_value).unwrap();
       let mut cursor = Cursor::new(&binary_data);
       let deserialized = LLSDParser::parse_binary_value(&mut cursor).unwrap();
       assert_eq!(message_value, deserialized);
    }

    #[test]
    fn test_error_handling() {
        // Test invalid message format
        let result = LLSDMessage::from_llsd_value(LLSDValue::String("invalid".to_string()));
        assert!(result.is_err());

        // Test empty data
        let result = LLSDParser::parse_binary_value(&mut Cursor::new(&[]));
        assert!(result.is_err());

        // Test invalid UTF-8 in string
        let mut invalid_string_data = vec![b's'];
        invalid_string_data.extend_from_slice(&4u32.to_be_bytes()); // length 4
        invalid_string_data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // invalid UTF-8
        let result = LLSDParser::parse_binary_value(&mut Cursor::new(&invalid_string_data));
        assert!(result.is_err());
    }

    #[test]
    fn test_message_handler_registration() {
        struct TestHandler;
        
        impl LLSDHandler for TestHandler {
            fn message_types(&self) -> Vec<&'static str> {
                vec!["TestMessage"]
            }
            
            fn handle<'a>(&'a self, _message: LLSDMessage, _context: HandlerContext<'a>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<LLSDMessage>>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(Some(LLSDMessage::new(
                        "TestResponse".to_string(), 
                        LLSDValue::String("handled".to_string())
                    )))
                })
            }
        }

        let mut handler = LLSDMessageHandler::new();
        handler.register_handler(TestHandler);
        
        // Verify handler was registered
        assert!(handler.handlers.contains_key("TestMessage"));
    }

    #[test]
    fn test_appearance_parsing() {
        let handler = LLSDMessageHandler::new();
        let mut appearance_data = HashMap::new();
        appearance_data.insert("serial".to_string(), LLSDValue::Integer(42));
        
        let mut visual_params = HashMap::new();
        visual_params.insert("1".to_string(), LLSDValue::Real(0.5));
        visual_params.insert("2".to_string(), LLSDValue::Real(0.8));
        appearance_data.insert("visual_params".to_string(), LLSDValue::Map(visual_params));

        let result = handler.parse_appearance_data(&appearance_data);
        assert!(result.is_ok());
        
        let appearance = result.unwrap();
        assert_eq!(appearance.serial, 42);
        assert_eq!(appearance.visual_params.params.len(), 2);
    }
}