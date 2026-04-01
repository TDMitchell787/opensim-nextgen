use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, error, debug};

/// Message template manager for LLUDP messages
/// Implements critical Second Life messages based on Cool Viewer's reliable patterns
#[derive(Debug, Clone)]
pub struct MessageTemplateManager {
    templates: HashMap<String, MessageTemplate>,
    message_numbers: HashMap<u32, String>,
    frequency_templates: HashMap<MessageFrequency, Vec<String>>,
}

/// Individual message template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub name: String,
    pub number: u32,
    pub frequency: MessageFrequency,
    pub trust: MessageTrust,
    pub encoding: MessageEncoding,
    pub blocks: Vec<MessageBlock>,
    pub zerocoded: bool,
    pub deprecated: bool,
}

/// Message frequency classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageFrequency {
    /// Fixed frequency messages (numbers 0-255)
    Fixed,
    /// Low frequency messages (numbers 256-...)
    Low,
    /// Medium frequency messages  
    Medium,
    /// High frequency messages
    High,
}

/// Message trust level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageTrust {
    /// Messages that must be trusted
    Trusted,
    /// Messages that don't require trust
    NotTrusted,
}

/// Message encoding type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageEncoding {
    /// Unencoded message
    Unencoded,
    /// Zero-coded message (RLE compression)
    Zerocoded,
}

/// Message block definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBlock {
    pub name: String,
    pub block_type: BlockType,
    pub fields: Vec<MessageField>,
}

/// Block type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    /// Single instance block
    Single,
    /// Multiple instances (with max count)
    Multiple(u8),
    /// Variable number of instances
    Variable,
}

/// Message field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageField {
    pub name: String,
    pub field_type: FieldType,
    pub size: Option<u8>,
}

/// Field type definitions matching Second Life protocol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    /// Unsigned 8-bit integer
    U8,
    /// Unsigned 16-bit integer
    U16,
    /// Unsigned 32-bit integer
    U32,
    /// Unsigned 64-bit integer
    U64,
    /// Signed 8-bit integer
    S8,
    /// Signed 16-bit integer
    S16,
    /// Signed 32-bit integer
    S32,
    /// Signed 64-bit integer
    S64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// 3D vector (3 floats)
    LLVector3,
    /// 3D vector (3 doubles)
    LLVector3d,
    /// Quaternion (4 floats)
    LLQuaternion,
    /// UUID (16 bytes)
    LLUUID,
    /// Boolean (1 byte)
    Bool,
    /// IP Address (4 bytes)
    IPAddr,
    /// IP Port (2 bytes)
    IPPort,
    /// Variable length data with size prefix
    Variable(u8),
    /// Fixed length data
    Fixed(u8),
}

/// Message template errors
#[derive(Debug, Error)]
pub enum MessageTemplateError {
    #[error("Message template not found: {0}")]
    NotFound(String),
    #[error("Invalid message number: {0}")]
    InvalidNumber(u32),
    #[error("Field type mismatch: expected {expected}, got {actual}")]
    FieldTypeMismatch { expected: String, actual: String },
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
}

impl MessageTemplateManager {
    /// Create new message template manager with all critical SL messages
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            message_numbers: HashMap::new(),
            frequency_templates: HashMap::new(),
        };

        // Load all critical Second Life messages
        manager.load_critical_messages();
        manager
    }

    /// Load critical Second Life messages based on Cool Viewer patterns
    fn load_critical_messages(&mut self) {
        // Fixed frequency messages (0-255)
        self.add_use_circuit_code();
        self.add_packet_ack();
        self.add_start_ping_check();
        self.add_complete_ping_check();
        self.add_agent_update();
        self.add_agent_data_update();
        self.add_region_handshake();
        self.add_agent_movement_complete();

        // Low frequency messages (256+)
        self.add_login_complete();
        self.add_logout_request();
        self.add_agent_throttle();
        self.add_enable_simulator();

        info!("Loaded {} critical LLUDP message templates", self.templates.len());
    }

    /// Add UseCircuitCode message (Critical for login)
    fn add_use_circuit_code(&mut self) {
        let template = MessageTemplate {
            name: "UseCircuitCode".to_string(),
            number: 1,
            frequency: MessageFrequency::Fixed,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
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
        self.add_template(template);
    }

    /// Add PacketAck message (Critical for reliability)
    fn add_packet_ack(&mut self) {
        let template = MessageTemplate {
            name: "PacketAck".to_string(),
            number: 251,
            frequency: MessageFrequency::Fixed,
            trust: MessageTrust::NotTrusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "Packets".to_string(),
                    block_type: BlockType::Variable,
                    fields: vec![
                        MessageField {
                            name: "ID".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add StartPingCheck message (Network diagnostics)
    fn add_start_ping_check(&mut self) {
        let template = MessageTemplate {
            name: "StartPingCheck".to_string(),
            number: 2,
            frequency: MessageFrequency::Fixed,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "PingID".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "PingID".to_string(),
                            field_type: FieldType::U8,
                            size: None,
                        },
                        MessageField {
                            name: "OldestUnacked".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add CompletePingCheck message (Network diagnostics)
    fn add_complete_ping_check(&mut self) {
        let template = MessageTemplate {
            name: "CompletePingCheck".to_string(),
            number: 3,
            frequency: MessageFrequency::Fixed,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "PingID".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "PingID".to_string(),
                            field_type: FieldType::U8,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add AgentUpdate message (Critical for avatar movement)
    fn add_agent_update(&mut self) {
        let template = MessageTemplate {
            name: "AgentUpdate".to_string(),
            number: 4,
            frequency: MessageFrequency::High,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Zerocoded,
            zerocoded: true,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "SessionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "BodyRotation".to_string(),
                            field_type: FieldType::LLQuaternion,
                            size: None,
                        },
                        MessageField {
                            name: "HeadRotation".to_string(),
                            field_type: FieldType::LLQuaternion,
                            size: None,
                        },
                        MessageField {
                            name: "State".to_string(),
                            field_type: FieldType::U8,
                            size: None,
                        },
                        MessageField {
                            name: "CameraCenter".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "CameraAtAxis".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "CameraLeftAxis".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "CameraUpAxis".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "Far".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "ControlFlags".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                        MessageField {
                            name: "Flags".to_string(),
                            field_type: FieldType::U8,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add AgentDataUpdate message (Agent information updates)
    fn add_agent_data_update(&mut self) {
        let template = MessageTemplate {
            name: "AgentDataUpdate".to_string(),
            number: 5,
            frequency: MessageFrequency::Medium,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Zerocoded,
            zerocoded: true,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "FirstName".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                        MessageField {
                            name: "LastName".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                        MessageField {
                            name: "GroupTitle".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                        MessageField {
                            name: "ActiveGroupID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "GroupPowers".to_string(),
                            field_type: FieldType::U64,
                            size: None,
                        },
                        MessageField {
                            name: "GroupName".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add RegionHandshake message (Region connection)
    fn add_region_handshake(&mut self) {
        let template = MessageTemplate {
            name: "RegionHandshake".to_string(),
            number: 148,
            frequency: MessageFrequency::Low,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Zerocoded,
            zerocoded: true,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "RegionInfo".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "RegionFlags".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                        MessageField {
                            name: "SimAccess".to_string(),
                            field_type: FieldType::U8,
                            size: None,
                        },
                        MessageField {
                            name: "SimName".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                        MessageField {
                            name: "SimOwner".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "IsEstateManager".to_string(),
                            field_type: FieldType::Bool,
                            size: None,
                        },
                        MessageField {
                            name: "WaterHeight".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "BillableFactor".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "CacheID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainBase0".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainBase1".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainBase2".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainBase3".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainDetail0".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainDetail1".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainDetail2".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainDetail3".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainStartHeight00".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainStartHeight01".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainStartHeight10".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainStartHeight11".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainHeightRange00".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainHeightRange01".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainHeightRange10".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                        MessageField {
                            name: "TerrainHeightRange11".to_string(),
                            field_type: FieldType::F32,
                            size: None,
                        },
                    ],
                },
                MessageBlock {
                    name: "RegionInfo2".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "RegionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add AgentMovementComplete message (Movement confirmation)
    fn add_agent_movement_complete(&mut self) {
        let template = MessageTemplate {
            name: "AgentMovementComplete".to_string(),
            number: 249,
            frequency: MessageFrequency::Fixed,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Zerocoded,
            zerocoded: true,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "SessionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                    ],
                },
                MessageBlock {
                    name: "Data".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "Position".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "LookAt".to_string(),
                            field_type: FieldType::LLVector3,
                            size: None,
                        },
                        MessageField {
                            name: "RegionHandle".to_string(),
                            field_type: FieldType::U64,
                            size: None,
                        },
                        MessageField {
                            name: "Timestamp".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                    ],
                },
                MessageBlock {
                    name: "SimData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "ChannelVersion".to_string(),
                            field_type: FieldType::Variable(2),
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add additional low frequency messages
    fn add_login_complete(&mut self) {
        let template = MessageTemplate {
            name: "LoginComplete".to_string(),
            number: 256,
            frequency: MessageFrequency::Low,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "LocationID".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    fn add_logout_request(&mut self) {
        let template = MessageTemplate {
            name: "LogoutRequest".to_string(),
            number: 257,
            frequency: MessageFrequency::Low,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "SessionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    fn add_agent_throttle(&mut self) {
        let template = MessageTemplate {
            name: "AgentThrottle".to_string(),
            number: 258,
            frequency: MessageFrequency::Low,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Unencoded,
            zerocoded: false,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "AgentData".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "AgentID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "SessionID".to_string(),
                            field_type: FieldType::LLUUID,
                            size: None,
                        },
                        MessageField {
                            name: "CircuitCode".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                    ],
                },
                MessageBlock {
                    name: "Throttle".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "GenCounter".to_string(),
                            field_type: FieldType::U32,
                            size: None,
                        },
                        MessageField {
                            name: "Throttles".to_string(),
                            field_type: FieldType::Variable(1),
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    fn add_enable_simulator(&mut self) {
        let template = MessageTemplate {
            name: "EnableSimulator".to_string(),
            number: 259,
            frequency: MessageFrequency::Low,
            trust: MessageTrust::Trusted,
            encoding: MessageEncoding::Zerocoded,
            zerocoded: true,
            deprecated: false,
            blocks: vec![
                MessageBlock {
                    name: "SimulatorInfo".to_string(),
                    block_type: BlockType::Single,
                    fields: vec![
                        MessageField {
                            name: "Handle".to_string(),
                            field_type: FieldType::U64,
                            size: None,
                        },
                        MessageField {
                            name: "IP".to_string(),
                            field_type: FieldType::IPAddr,
                            size: None,
                        },
                        MessageField {
                            name: "Port".to_string(),
                            field_type: FieldType::IPPort,
                            size: None,
                        },
                    ],
                },
            ],
        };
        self.add_template(template);
    }

    /// Add a template to the manager
    fn add_template(&mut self, template: MessageTemplate) {
        let name = template.name.clone();
        let number = template.number;
        let frequency = template.frequency.clone();

        // Store in templates map
        self.templates.insert(name.clone(), template);
        
        // Store in message numbers map
        self.message_numbers.insert(number, name.clone());
        
        // Store in frequency map
        self.frequency_templates
            .entry(frequency)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Result<&MessageTemplate, MessageTemplateError> {
        self.templates.get(name).ok_or_else(|| MessageTemplateError::NotFound(name.to_string()))
    }

    /// Get template by message number
    pub fn get_template_by_number(&self, number: u32) -> Result<&MessageTemplate, MessageTemplateError> {
        let name = self.message_numbers.get(&number)
            .ok_or_else(|| MessageTemplateError::InvalidNumber(number))?;
        self.get_template(name)
    }

    /// Get all templates for a frequency
    pub fn get_templates_by_frequency(&self, frequency: &MessageFrequency) -> Vec<&MessageTemplate> {
        self.frequency_templates.get(frequency)
            .map(|names| names.iter()
                .filter_map(|name| self.templates.get(name))
                .collect())
            .unwrap_or_default()
    }

    /// Check if message is critical for login
    pub fn is_critical_message(&self, name: &str) -> bool {
        matches!(name, 
            "UseCircuitCode" | 
            "PacketAck" | 
            "StartPingCheck" | 
            "CompletePingCheck" | 
            "AgentUpdate" | 
            "AgentMovementComplete" |
            "RegionHandshake" |
            "LoginComplete"
        )
    }

    /// Get field size for a field type
    pub fn get_field_size(&self, field_type: &FieldType) -> Option<usize> {
        match field_type {
            FieldType::U8 | FieldType::S8 | FieldType::Bool => Some(1),
            FieldType::U16 | FieldType::S16 | FieldType::IPPort => Some(2),
            FieldType::U32 | FieldType::S32 | FieldType::F32 | FieldType::IPAddr => Some(4),
            FieldType::U64 | FieldType::S64 | FieldType::F64 => Some(8),
            FieldType::LLVector3 => Some(12), // 3 floats
            FieldType::LLVector3d => Some(24), // 3 doubles
            FieldType::LLQuaternion => Some(16), // 4 floats
            FieldType::LLUUID => Some(16),
            FieldType::Variable(_) | FieldType::Fixed(0) => None, // Variable size
            FieldType::Fixed(size) => Some(*size as usize),
        }
    }

    /// Validate message structure
    pub fn validate_message(&self, name: &str, data: &[u8]) -> Result<(), MessageTemplateError> {
        let template = self.get_template(name)?;
        
        // Basic validation - check if we have minimum required data
        let mut expected_size = 0;
        for block in &template.blocks {
            match block.block_type {
                BlockType::Single => {
                    for field in &block.fields {
                        if let Some(size) = self.get_field_size(&field.field_type) {
                            expected_size += size;
                        }
                    }
                }
                _ => {
                    // Variable blocks - basic validation not possible without parsing
                    break;
                }
            }
        }

        if data.len() < expected_size {
            return Err(MessageTemplateError::InvalidDataLength {
                expected: expected_size,
                actual: data.len(),
            });
        }

        Ok(())
    }

    /// Get all template names
    pub fn get_all_template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> MessageTemplateStatistics {
        let mut stats = MessageTemplateStatistics::default();
        
        stats.total_templates = self.templates.len();
        
        for template in self.templates.values() {
            match template.frequency {
                MessageFrequency::Fixed => stats.fixed_frequency += 1,
                MessageFrequency::Low => stats.low_frequency += 1,
                MessageFrequency::Medium => stats.medium_frequency += 1,
                MessageFrequency::High => stats.high_frequency += 1,
            }
            
            if template.trust == MessageTrust::Trusted {
                stats.trusted_messages += 1;
            }
            
            if template.zerocoded {
                stats.zerocoded_messages += 1;
            }
            
            if self.is_critical_message(&template.name) {
                stats.critical_messages += 1;
            }
        }
        
        stats
    }
}

/// Message template statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MessageTemplateStatistics {
    pub total_templates: usize,
    pub fixed_frequency: usize,
    pub low_frequency: usize,
    pub medium_frequency: usize,
    pub high_frequency: usize,
    pub trusted_messages: usize,
    pub zerocoded_messages: usize,
    pub critical_messages: usize,
}

impl Default for MessageTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_template_manager_creation() {
        let manager = MessageTemplateManager::new();
        assert!(manager.templates.len() > 0);
        assert!(manager.get_template("UseCircuitCode").is_ok());
        assert!(manager.get_template("PacketAck").is_ok());
        assert!(manager.get_template("AgentUpdate").is_ok());
    }

    #[test]
    fn test_get_template_by_number() {
        let manager = MessageTemplateManager::new();
        assert!(manager.get_template_by_number(1).is_ok()); // UseCircuitCode
        assert!(manager.get_template_by_number(251).is_ok()); // PacketAck
        assert!(manager.get_template_by_number(999999).is_err()); // Invalid
    }

    #[test]
    fn test_critical_messages() {
        let manager = MessageTemplateManager::new();
        assert!(manager.is_critical_message("UseCircuitCode"));
        assert!(manager.is_critical_message("PacketAck"));
        assert!(manager.is_critical_message("AgentUpdate"));
        assert!(!manager.is_critical_message("InvalidMessage"));
    }

    #[test]
    fn test_field_sizes() {
        let manager = MessageTemplateManager::new();
        assert_eq!(manager.get_field_size(&FieldType::U32), Some(4));
        assert_eq!(manager.get_field_size(&FieldType::LLUUID), Some(16));
        assert_eq!(manager.get_field_size(&FieldType::LLVector3), Some(12));
        assert_eq!(manager.get_field_size(&FieldType::Variable(1)), None);
    }
}