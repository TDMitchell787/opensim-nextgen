//! Messaging System for OpenSim Next Social Features
//! 
//! Provides comprehensive messaging capabilities including direct messages,
//! group chat, notifications, message history, and real-time communication.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Messaging system manager
pub struct MessagingSystem {
    database: Arc<DatabaseManager>,
    config: SocialConfig,
    active_conversations: Arc<RwLock<HashMap<Uuid, Conversation>>>,
    message_cache: Arc<RwLock<HashMap<Uuid, Vec<Message>>>>,
    online_users: Arc<RwLock<HashMap<Uuid, UserSession>>>,
    message_sender: mpsc::UnboundedSender<MessageEvent>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<MessageEvent>>>>,
    rate_limiter: Arc<RwLock<HashMap<Uuid, MessageRateLimit>>>,
}

/// Message conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub conversation_id: Uuid,
    pub conversation_type: ConversationType,
    pub participants: Vec<Uuid>,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub settings: ConversationSettings,
    pub metadata: HashMap<String, String>,
}

/// Types of conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationType {
    DirectMessage,
    GroupChat,
    SystemNotification,
    Support,
}

/// Conversation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSettings {
    pub notifications_enabled: bool,
    pub message_retention_days: Option<u32>,
    pub allow_file_sharing: bool,
    pub allow_voice_messages: bool,
    pub moderated: bool,
    pub encryption_enabled: bool,
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub message_type: MessageType,
    pub content: MessageContent,
    pub sent_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub status: MessageStatus,
    pub reactions: HashMap<String, Vec<Uuid>>, // emoji -> user_ids
    pub thread_id: Option<Uuid>,
    pub reply_to: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

/// Types of messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Text,
    Image,
    Audio,
    Video,
    File,
    Location,
    System,
    Sticker,
    Poll,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub text: Option<String>,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub media_size: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub location: Option<LocationData>,
    pub poll: Option<PollData>,
    pub sticker_id: Option<String>,
}

/// Location data for location messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationData {
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub region_name: Option<String>,
}

/// Poll data for poll messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    pub question: String,
    pub options: Vec<PollOption>,
    pub multiple_choice: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub votes: HashMap<Uuid, Vec<u32>>, // user_id -> option_indices
}

/// Poll option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub text: String,
    pub vote_count: u32,
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
    Deleted,
}

/// User session for messaging
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: Uuid,
    pub online_since: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub status: OnlineStatus,
    pub device_info: Option<String>,
}

/// Message event for real-time processing
#[derive(Debug, Clone)]
pub enum MessageEvent {
    MessageSent(Message),
    MessageDelivered(Uuid, Uuid), // message_id, user_id
    MessageRead(Uuid, Uuid),      // message_id, user_id
    UserOnline(Uuid),
    UserOffline(Uuid),
    TypingStarted(Uuid, Uuid),    // conversation_id, user_id
    TypingStopped(Uuid, Uuid),    // conversation_id, user_id
}

/// Message rate limiting
#[derive(Debug, Clone)]
struct MessageRateLimit {
    user_id: Uuid,
    messages_this_minute: u32,
    last_reset: std::time::Instant,
}

/// Request to send a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Option<Uuid>,
    pub recipient_ids: Vec<Uuid>, // For new conversations
    pub message_type: MessageType,
    pub content: MessageContent,
    pub reply_to: Option<Uuid>,
    pub thread_id: Option<Uuid>,
}

/// Request to create a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub conversation_type: ConversationType,
    pub participant_ids: Vec<Uuid>,
    pub title: Option<String>,
    pub settings: Option<ConversationSettings>,
}

/// Message history request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryRequest {
    pub conversation_id: Uuid,
    pub before_message_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub include_system_messages: bool,
}

/// Message search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSearchCriteria {
    pub query: String,
    pub conversation_ids: Vec<Uuid>,
    pub sender_ids: Vec<Uuid>,
    pub message_types: Vec<MessageType>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub sort_by: MessageSortOption,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Message sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSortOption {
    Newest,
    Oldest,
    Relevance,
}

/// Conversation list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationSummary>,
    pub total_count: u32,
    pub unread_count: u32,
}

/// Conversation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub conversation: Conversation,
    pub last_message: Option<Message>,
    pub unread_count: u32,
    pub participant_info: Vec<ParticipantInfo>,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_image: Option<String>,
    pub online_status: OnlineStatus,
    pub last_read_message_id: Option<Uuid>,
}

impl MessagingSystem {
    /// Create new messaging system
    pub fn new(database: Arc<DatabaseManager>, config: SocialConfig) -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();

        Self {
            database,
            config,
            active_conversations: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            online_users: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize messaging system
    pub async fn initialize(&self) -> SocialResult<()> {
        info!("Initializing messaging system");

        // Create database tables
        self.create_tables().await?;

        // Load recent conversations and messages
        self.load_recent_data().await?;

        // Start message processing task
        self.start_message_processor().await;

        info!("Messaging system initialized successfully");
        Ok(())
    }

    /// Send a message
    pub async fn send_message(&self, sender_id: Uuid, request: SendMessageRequest) -> SocialResult<Message> {
        info!("Sending message from user {}", sender_id);

        // Check rate limits
        self.check_message_rate_limit(sender_id).await?;

        // Get or create conversation
        let conversation_id = if let Some(conv_id) = request.conversation_id {
            // Verify sender is participant
            self.verify_conversation_participant(conv_id, sender_id).await?;
            conv_id
        } else {
            // Create new conversation
            let create_request = CreateConversationRequest {
                conversation_type: if request.recipient_ids.len() == 1 {
                    ConversationType::DirectMessage
                } else {
                    ConversationType::GroupChat
                },
                participant_ids: {
                    let mut participants = request.recipient_ids.clone();
                    participants.push(sender_id);
                    participants
                },
                title: None,
                settings: None,
            };
            self.create_conversation(sender_id, create_request).await?.conversation_id
        };

        // Create message
        let message = Message {
            message_id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            message_type: request.message_type,
            content: request.content,
            sent_at: Utc::now(),
            edited_at: None,
            status: MessageStatus::Sent,
            reactions: HashMap::new(),
            thread_id: request.thread_id,
            reply_to: request.reply_to,
            metadata: HashMap::new(),
        };

        // Validate message content
        self.validate_message_content(&message).await?;

        // Save message to database
        self.save_message(&message).await?;

        // Update conversation last message time
        self.update_conversation_last_message(conversation_id, &message).await?;

        // Update cache
        {
            let mut cache = self.message_cache.write().await;
            cache.entry(conversation_id).or_insert_with(Vec::new).push(message.clone());
        }

        // Send real-time event
        let _ = self.message_sender.send(MessageEvent::MessageSent(message.clone()));

        // Update rate limiter
        self.update_message_rate_limit(sender_id).await;

        info!("Message sent successfully: {}", message.message_id);
        Ok(message)
    }

    /// Create a new conversation
    pub async fn create_conversation(&self, creator_id: Uuid, request: CreateConversationRequest) -> SocialResult<Conversation> {
        info!("Creating new conversation by user {}", creator_id);

        // Validate request
        self.validate_conversation_creation(&request, creator_id).await?;

        // Create conversation
        let conversation = Conversation {
            conversation_id: Uuid::new_v4(),
            conversation_type: request.conversation_type,
            participants: request.participant_ids,
            title: request.title,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_message_at: None,
            settings: request.settings.unwrap_or_default(),
            metadata: HashMap::new(),
        };

        // Save to database
        self.save_conversation(&conversation).await?;

        // Update cache
        {
            let mut conversations = self.active_conversations.write().await;
            conversations.insert(conversation.conversation_id, conversation.clone());
        }

        info!("Conversation created successfully: {}", conversation.conversation_id);
        Ok(conversation)
    }

    /// Get conversation history
    pub async fn get_conversation_history(&self, user_id: Uuid, request: MessageHistoryRequest) -> SocialResult<Vec<Message>> {
        debug!("Getting conversation history for user {} conversation {}", user_id, request.conversation_id);

        // Verify user is participant
        self.verify_conversation_participant(request.conversation_id, user_id).await?;

        // Get messages from cache first
        let cached_messages = {
            let cache = self.message_cache.read().await;
            cache.get(&request.conversation_id).cloned().unwrap_or_default()
        };

        let mut messages = cached_messages;

        // Apply filters
        if !request.include_system_messages {
            messages.retain(|m| !matches!(m.message_type, MessageType::System));
        }

        if let Some(before_id) = request.before_message_id {
            // Find the message and take messages before it
            if let Some(before_index) = messages.iter().position(|m| m.message_id == before_id) {
                messages.truncate(before_index);
            }
        }

        // Sort by sent time (newest first)
        messages.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));

        // Apply limit
        if let Some(limit) = request.limit {
            messages.truncate(limit as usize);
        }

        debug!("Retrieved {} messages from conversation history", messages.len());
        Ok(messages)
    }

    /// Get user's conversations
    pub async fn get_user_conversations(&self, user_id: Uuid) -> SocialResult<ConversationListResponse> {
        debug!("Getting conversations for user {}", user_id);

        let conversations = self.active_conversations.read().await;
        let mut user_conversations = Vec::new();
        let mut unread_count = 0;

        for conversation in conversations.values() {
            if conversation.participants.contains(&user_id) {
                // Get last message
                let last_message = self.get_conversation_last_message(conversation.conversation_id).await?;
                
                // Calculate unread count (placeholder)
                let conversation_unread_count = 0; // Would calculate actual unread count

                // Get participant info
                let participant_info = self.get_participant_info(&conversation.participants).await?;

                let summary = ConversationSummary {
                    conversation: conversation.clone(),
                    last_message,
                    unread_count: conversation_unread_count,
                    participant_info,
                };

                unread_count += conversation_unread_count;
                user_conversations.push(summary);
            }
        }

        // Sort by last message time
        user_conversations.sort_by(|a, b| {
            let a_time = a.last_message.as_ref().map(|m| m.sent_at).unwrap_or(a.conversation.created_at);
            let b_time = b.last_message.as_ref().map(|m| m.sent_at).unwrap_or(b.conversation.created_at);
            b_time.cmp(&a_time)
        });

        let response = ConversationListResponse {
            total_count: user_conversations.len() as u32,
            unread_count,
            conversations: user_conversations,
        };

        debug!("Retrieved {} conversations for user", response.total_count);
        Ok(response)
    }

    /// Search messages
    pub async fn search_messages(&self, user_id: Uuid, criteria: MessageSearchCriteria) -> SocialResult<Vec<Message>> {
        debug!("Searching messages for user {} with query: {}", user_id, criteria.query);

        let mut results = Vec::new();
        let cache = self.message_cache.read().await;

        // Search through cached messages
        for (conversation_id, messages) in cache.iter() {
            // Check if user is participant in conversation
            if self.is_conversation_participant(*conversation_id, user_id).await? {
                for message in messages {
                    if self.message_matches_criteria(message, &criteria) {
                        results.push(message.clone());
                    }
                }
            }
        }

        // Apply sorting
        match criteria.sort_by {
            MessageSortOption::Newest => {
                results.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
            }
            MessageSortOption::Oldest => {
                results.sort_by(|a, b| a.sent_at.cmp(&b.sent_at));
            }
            MessageSortOption::Relevance => {
                // Would implement relevance scoring
            }
        }

        // Apply pagination
        if let Some(offset) = criteria.offset {
            if offset as usize >= results.len() {
                results.clear();
            } else {
                results = results.into_iter().skip(offset as usize).collect();
            }
        }

        if let Some(limit) = criteria.limit {
            results.truncate(limit as usize);
        }

        debug!("Message search returned {} results", results.len());
        Ok(results)
    }

    /// Mark message as read
    pub async fn mark_message_read(&self, user_id: Uuid, message_id: Uuid) -> SocialResult<()> {
        debug!("Marking message {} as read by user {}", message_id, user_id);

        // Update message status in database
        self.update_message_read_status(message_id, user_id).await?;

        // Send real-time event
        let _ = self.message_sender.send(MessageEvent::MessageRead(message_id, user_id));

        debug!("Message marked as read");
        Ok(())
    }

    /// Add reaction to message
    pub async fn add_message_reaction(&self, user_id: Uuid, message_id: Uuid, emoji: String) -> SocialResult<()> {
        info!("Adding reaction '{}' to message {} by user {}", emoji, message_id, user_id);

        // Get message and verify user can react
        let mut message = self.get_message(message_id).await?;
        self.verify_conversation_participant(message.conversation_id, user_id).await?;

        // Add reaction
        message.reactions.entry(emoji.clone()).or_insert_with(Vec::new).push(user_id);

        // Save to database
        self.save_message(&message).await?;

        // Update cache
        {
            let mut cache = self.message_cache.write().await;
            if let Some(messages) = cache.get_mut(&message.conversation_id) {
                if let Some(cached_message) = messages.iter_mut().find(|m| m.message_id == message_id) {
                    cached_message.reactions = message.reactions.clone();
                }
            }
        }

        info!("Reaction added successfully");
        Ok(())
    }

    /// Update user online status
    pub async fn update_user_status(&self, user_id: Uuid, status: OnlineStatus) -> SocialResult<()> {
        debug!("Updating online status for user {} to {:?}", user_id, status);

        let now = Utc::now();
        
        {
            let mut online_users = self.online_users.write().await;
            
            if matches!(status, OnlineStatus::Online) {
                let session = UserSession {
                    user_id,
                    online_since: now,
                    last_seen: now,
                    status,
                    device_info: None,
                };
                online_users.insert(user_id, session);
                
                // Send online event
                let _ = self.message_sender.send(MessageEvent::UserOnline(user_id));
            } else {
                if let Some(session) = online_users.get_mut(&user_id) {
                    session.status = status.clone();
                    session.last_seen = now;
                }
                
                if matches!(status, OnlineStatus::Offline) {
                    online_users.remove(&user_id);
                    
                    // Send offline event
                    let _ = self.message_sender.send(MessageEvent::UserOffline(user_id));
                }
            }
        }

        debug!("User status updated");
        Ok(())
    }

    /// Start typing indicator
    pub async fn start_typing(&self, user_id: Uuid, conversation_id: Uuid) -> SocialResult<()> {
        debug!("User {} started typing in conversation {}", user_id, conversation_id);

        // Verify user is participant
        self.verify_conversation_participant(conversation_id, user_id).await?;

        // Send typing event
        let _ = self.message_sender.send(MessageEvent::TypingStarted(conversation_id, user_id));

        Ok(())
    }

    /// Stop typing indicator
    pub async fn stop_typing(&self, user_id: Uuid, conversation_id: Uuid) -> SocialResult<()> {
        debug!("User {} stopped typing in conversation {}", user_id, conversation_id);

        // Send typing stopped event
        let _ = self.message_sender.send(MessageEvent::TypingStopped(conversation_id, user_id));

        Ok(())
    }

    // Private helper methods

    async fn check_message_rate_limit(&self, user_id: Uuid) -> SocialResult<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = std::time::Instant::now();

        let user_limit = rate_limiter.entry(user_id).or_insert_with(|| MessageRateLimit {
            user_id,
            messages_this_minute: 0,
            last_reset: now,
        });

        // Reset if a minute has passed
        if now.duration_since(user_limit.last_reset).as_secs() >= 60 {
            user_limit.messages_this_minute = 0;
            user_limit.last_reset = now;
        }

        if user_limit.messages_this_minute >= self.config.message_rate_limit {
            return Err(SocialError::RateLimitExceeded { user_id });
        }

        Ok(())
    }

    async fn update_message_rate_limit(&self, user_id: Uuid) {
        let mut rate_limiter = self.rate_limiter.write().await;
        if let Some(user_limit) = rate_limiter.get_mut(&user_id) {
            user_limit.messages_this_minute += 1;
        }
    }

    async fn validate_message_content(&self, message: &Message) -> SocialResult<()> {
        match &message.message_type {
            MessageType::Text => {
                if let Some(text) = &message.content.text {
                    if text.trim().is_empty() {
                        return Err(SocialError::ValidationError {
                            message: "Message text cannot be empty".to_string(),
                        });
                    }
                    if text.len() > 4000 {
                        return Err(SocialError::ValidationError {
                            message: "Message text too long (max 4000 characters)".to_string(),
                        });
                    }
                }
            }
            MessageType::Image | MessageType::Audio | MessageType::Video | MessageType::File => {
                if message.content.media_url.is_none() {
                    return Err(SocialError::ValidationError {
                        message: "Media message requires media URL".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn validate_conversation_creation(&self, request: &CreateConversationRequest, creator_id: Uuid) -> SocialResult<()> {
        if request.participant_ids.is_empty() {
            return Err(SocialError::ValidationError {
                message: "Conversation must have at least one participant".to_string(),
            });
        }

        if !request.participant_ids.contains(&creator_id) {
            return Err(SocialError::ValidationError {
                message: "Creator must be included in participants".to_string(),
            });
        }

        // Check for duplicate participants
        let unique_participants: std::collections::HashSet<_> = request.participant_ids.iter().collect();
        if unique_participants.len() != request.participant_ids.len() {
            return Err(SocialError::ValidationError {
                message: "Duplicate participants not allowed".to_string(),
            });
        }

        Ok(())
    }

    async fn verify_conversation_participant(&self, conversation_id: Uuid, user_id: Uuid) -> SocialResult<()> {
        if !self.is_conversation_participant(conversation_id, user_id).await? {
            return Err(SocialError::AccessDenied {
                reason: "User is not a participant in this conversation".to_string(),
            });
        }
        Ok(())
    }

    async fn is_conversation_participant(&self, conversation_id: Uuid, user_id: Uuid) -> SocialResult<bool> {
        let conversations = self.active_conversations.read().await;
        
        if let Some(conversation) = conversations.get(&conversation_id) {
            Ok(conversation.participants.contains(&user_id))
        } else {
            Ok(false)
        }
    }

    fn message_matches_criteria(&self, message: &Message, criteria: &MessageSearchCriteria) -> bool {
        // Text search
        if let Some(text) = &message.content.text {
            if !text.to_lowercase().contains(&criteria.query.to_lowercase()) {
                return false;
            }
        }

        // Conversation filter
        if !criteria.conversation_ids.is_empty() && !criteria.conversation_ids.contains(&message.conversation_id) {
            return false;
        }

        // Sender filter
        if !criteria.sender_ids.is_empty() && !criteria.sender_ids.contains(&message.sender_id) {
            return false;
        }

        // Message type filter
        if !criteria.message_types.is_empty() && !criteria.message_types.contains(&message.message_type) {
            return false;
        }

        // Date range filter
        if let Some((start, end)) = &criteria.date_range {
            if message.sent_at < *start || message.sent_at > *end {
                return false;
            }
        }

        true
    }

    async fn update_conversation_last_message(&self, conversation_id: Uuid, message: &Message) -> SocialResult<()> {
        let mut conversations = self.active_conversations.write().await;
        if let Some(conversation) = conversations.get_mut(&conversation_id) {
            conversation.last_message_at = Some(message.sent_at);
            conversation.updated_at = message.sent_at;
        }
        Ok(())
    }

    async fn get_conversation_last_message(&self, conversation_id: Uuid) -> SocialResult<Option<Message>> {
        let cache = self.message_cache.read().await;
        
        if let Some(messages) = cache.get(&conversation_id) {
            // Get the most recent message
            let last_message = messages.iter().max_by_key(|m| m.sent_at).cloned();
            Ok(last_message)
        } else {
            Ok(None)
        }
    }

    async fn get_participant_info(&self, participant_ids: &[Uuid]) -> SocialResult<Vec<ParticipantInfo>> {
        let online_users = self.online_users.read().await;
        let statuses: HashMap<Uuid, OnlineStatus> = participant_ids.iter()
            .map(|&user_id| {
                let status = online_users.get(&user_id)
                    .map(|session| session.status.clone())
                    .unwrap_or(OnlineStatus::Offline);
                (user_id, status)
            })
            .collect();
        drop(online_users);

        let mut participants = Vec::new();

        for &user_id in participant_ids {
            let display_name = self.get_user_display_name(user_id).await;
            let online_status = statuses.get(&user_id).cloned().unwrap_or(OnlineStatus::Offline);

            let participant = ParticipantInfo {
                user_id,
                display_name,
                avatar_image: None,
                online_status,
                last_read_message_id: None,
            };

            participants.push(participant);
        }

        Ok(participants)
    }

    async fn get_message(&self, message_id: Uuid) -> SocialResult<Message> {
        let cache = self.message_cache.read().await;
        
        for messages in cache.values() {
            if let Some(message) = messages.iter().find(|m| m.message_id == message_id) {
                return Ok(message.clone());
            }
        }

        Err(SocialError::ValidationError {
            message: "Message not found".to_string(),
        })
    }

    async fn start_message_processor(&self) {
        // Would start background task to process message events
        // For now, just consume the receiver to prevent it from blocking
        let mut receiver = self.message_receiver.write().await.take();
        if let Some(mut rx) = receiver {
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    // Process message event
                    debug!("Processing message event: {:?}", event);
                }
            });
        }
    }

    // Database operations (placeholder implementations)

    async fn create_tables(&self) -> SocialResult<()> {
        Ok(())
    }

    async fn load_recent_data(&self) -> SocialResult<()> {
        Ok(())
    }

    async fn save_message(&self, _message: &Message) -> SocialResult<()> {
        Ok(())
    }

    async fn save_conversation(&self, _conversation: &Conversation) -> SocialResult<()> {
        Ok(())
    }

    async fn update_message_read_status(&self, _message_id: Uuid, _user_id: Uuid) -> SocialResult<()> {
        Ok(())
    }

    async fn get_user_display_name(&self, user_id: Uuid) -> String {
        if let Ok(pool) = self.database.legacy_pool() {
            let row_result = sqlx::query(
                "SELECT FirstName, LastName FROM UserAccounts WHERE PrincipalID = $1"
            )
            .bind(user_id.to_string())
            .fetch_optional(pool)
            .await;

            if let Ok(Some(row)) = row_result {
                let first_name: String = row.try_get("FirstName").unwrap_or_else(|_| "Unknown".to_string());
                let last_name: String = row.try_get("LastName").unwrap_or_else(|_| "User".to_string());
                return format!("{} {}", first_name, last_name);
            }
        }
        format!("User {}", &user_id.to_string()[..8])
    }
}

impl Default for MessageSearchCriteria {
    fn default() -> Self {
        Self {
            query: String::new(),
            conversation_ids: Vec::new(),
            sender_ids: Vec::new(),
            message_types: Vec::new(),
            date_range: None,
            sort_by: MessageSortOption::Newest,
            limit: None,
            offset: None,
        }
    }
}

impl Default for ConversationSettings {
    fn default() -> Self {
        Self {
            notifications_enabled: true,
            message_retention_days: None,
            allow_file_sharing: true,
            allow_voice_messages: true,
            moderated: false,
            encryption_enabled: false,
        }
    }
}

impl Default for MessageContent {
    fn default() -> Self {
        Self {
            text: None,
            media_url: None,
            media_type: None,
            media_size: None,
            thumbnail_url: None,
            location: None,
            poll: None,
            sticker_id: None,
        }
    }
}

impl Default for MessageSortOption {
    fn default() -> Self {
        Self::Newest
    }
}