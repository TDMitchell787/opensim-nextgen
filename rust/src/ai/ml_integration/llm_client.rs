use super::super::AIError;
use base64::{engine::general_purpose as base64_engine, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAIConfig {
    pub enabled: bool,
    pub behaviors_enabled: bool,
    pub llm_enabled: bool,
}

impl Default for HybridAIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            behaviors_enabled: false,
            llm_enabled: false,
        }
    }
}

static INI_PATHS: &[&str] = &["bin/llm.ini", "llm.ini", "rust/llm.ini"];

fn read_ini_contents() -> Option<String> {
    if let Ok(instance_dir) = std::env::var("OPENSIM_INSTANCE_DIR") {
        let instance_path = format!("{}/llm.ini", instance_dir);
        if let Ok(contents) = std::fs::read_to_string(&instance_path) {
            tracing::info!("[HybridAI] Loading config from {}", instance_path);
            return Some(contents);
        }
        let instance_bin_path = format!("{}/bin/llm.ini", instance_dir);
        if let Ok(contents) = std::fs::read_to_string(&instance_bin_path) {
            tracing::info!("[HybridAI] Loading config from {}", instance_bin_path);
            return Some(contents);
        }
    }
    for path in INI_PATHS {
        if let Ok(contents) = std::fs::read_to_string(path) {
            tracing::info!("[HybridAI] Loading config from {}", path);
            return Some(contents);
        }
    }
    None
}

fn parse_ini_sections(contents: &str) -> (HybridAIConfig, LLMConfig) {
    let mut ai = HybridAIConfig::default();
    let mut llm = LLMConfig::hardcoded_defaults();
    let mut current_section = String::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_lowercase();
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            match current_section.as_str() {
                "hybridai" => match key {
                    "enabled" => ai.enabled = val.eq_ignore_ascii_case("true"),
                    "behaviors_enabled" => ai.behaviors_enabled = val.eq_ignore_ascii_case("true"),
                    "llm_enabled" => ai.llm_enabled = val.eq_ignore_ascii_case("true"),
                    _ => {}
                },
                "llm" => match key {
                    "endpoint" => llm.endpoint = val.to_string(),
                    "model" => llm.model = val.to_string(),
                    "timeout" => {
                        if let Ok(v) = val.parse() {
                            llm.timeout_seconds = v;
                        }
                    }
                    "max_tokens" => {
                        if let Ok(v) = val.parse() {
                            llm.max_tokens = v;
                        }
                    }
                    "temperature" => {
                        if let Ok(v) = val.parse() {
                            llm.temperature = v;
                        }
                    }
                    "provider" => llm.provider = val.to_lowercase(),
                    "api_key" => llm.api_key = val.to_string(),
                    "context_window" | "num_ctx" => {
                        if let Ok(v) = val.parse() {
                            llm.context_window = v;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    (ai, llm)
}

impl HybridAIConfig {
    pub fn from_ini() -> Self {
        match read_ini_contents() {
            Some(contents) => {
                let (ai, _) = parse_ini_sections(&contents);
                tracing::info!(
                    "[HybridAI] enabled={}, behaviors={}, llm={}",
                    ai.enabled,
                    ai.behaviors_enabled,
                    ai.llm_enabled
                );
                ai
            }
            None => {
                tracing::info!("[HybridAI] No llm.ini found, AI NPCs disabled by default");
                Self::default()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub endpoint: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_tokens: usize,
    pub temperature: f32,
    pub provider: String,
    pub api_key: String,
    pub context_window: usize,
}

impl LLMConfig {
    pub fn from_ini() -> Self {
        match read_ini_contents() {
            Some(contents) => {
                let (_, llm) = parse_ini_sections(&contents);
                tracing::info!(
                    "[LLM] Config: model={}, endpoint={}, timeout={}s, temp={}, ctx={}",
                    llm.model,
                    llm.endpoint,
                    llm.timeout_seconds,
                    llm.temperature,
                    llm.context_window
                );
                llm
            }
            None => {
                tracing::info!("[LLM] No llm.ini found, using defaults");
                Self::hardcoded_defaults()
            }
        }
    }

    fn hardcoded_defaults() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            timeout_seconds: 30,
            max_tokens: 2048,
            temperature: 0.7,
            provider: "ollama".to_string(),
            api_key: String::new(),
            context_window: 32768,
        }
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self::from_ini()
    }
}

impl LLMConfig {
    pub fn resolve_provider(&mut self) {
        match self.provider.as_str() {
            "auto" => self.auto_detect_provider(),
            "claude-code" => self.discover_claude_code_credentials(),
            "anthropic" => {
                if self.api_key.is_empty() {
                    if let Some(key) = Self::find_anthropic_key() {
                        self.api_key = key;
                    }
                }
            }
            _ => {}
        }
    }

    fn auto_detect_provider(&mut self) {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                tracing::info!("[LLM] Auto-detected ANTHROPIC_API_KEY — using Anthropic provider");
                self.provider = "anthropic".to_string();
                self.api_key = key;
                self.apply_anthropic_defaults();
                return;
            }
        }

        if let Some(key) = Self::read_claude_code_credentials() {
            tracing::info!(
                "[LLM] Auto-detected Claude Code credentials — using Anthropic provider"
            );
            self.provider = "anthropic".to_string();
            self.api_key = key;
            self.apply_anthropic_defaults();
            return;
        }

        tracing::info!("[LLM] No API key found — using Ollama provider (local)");
        self.provider = "ollama".to_string();
    }

    fn discover_claude_code_credentials(&mut self) {
        if let Some(key) = Self::find_anthropic_key() {
            self.provider = "anthropic".to_string();
            self.api_key = key;
            self.apply_anthropic_defaults();
            tracing::info!("[LLM] Claude Code provider resolved — using Anthropic API with auto-discovered key");
        } else {
            tracing::warn!("[LLM] claude-code provider selected but no credentials found — falling back to Ollama");
            self.provider = "ollama".to_string();
        }
    }

    fn find_anthropic_key() -> Option<String> {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }
        Self::read_claude_code_credentials()
    }

    fn read_claude_code_credentials() -> Option<String> {
        let home = std::env::var("HOME").ok()?;

        let creds_path = format!("{}/.claude/.credentials.json", home);
        if let Ok(contents) = std::fs::read_to_string(&creds_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(key) = json.get("claudeAiApiKey").and_then(|v| v.as_str()) {
                    if !key.is_empty() {
                        tracing::info!("[LLM] Found API key in ~/.claude/.credentials.json");
                        return Some(key.to_string());
                    }
                }
                if let Some(key) = json.get("apiKey").and_then(|v| v.as_str()) {
                    if !key.is_empty() {
                        tracing::info!(
                            "[LLM] Found API key in ~/.claude/.credentials.json (apiKey)"
                        );
                        return Some(key.to_string());
                    }
                }
            }
        }

        let config_path = format!("{}/.claude/config.json", home);
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(key) = json.get("apiKey").and_then(|v| v.as_str()) {
                    if !key.is_empty() {
                        tracing::info!("[LLM] Found API key in ~/.claude/config.json");
                        return Some(key.to_string());
                    }
                }
            }
        }

        None
    }

    fn apply_anthropic_defaults(&mut self) {
        if self.model == "llama3.2"
            || self.model.starts_with("llama")
            || self.model.starts_with("mistral")
        {
            self.model = "claude-sonnet-4-20250514".to_string();
        }
        if self.endpoint == "http://localhost:11434" {
            self.endpoint = "https://api.anthropic.com".to_string();
        }
        if self.context_window <= 32768 {
            self.context_window = 200000;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub text: String,
    pub model: String,
    pub tokens_used: usize,
    pub processing_time_ms: u64,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    MaxTokens,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug)]
pub struct LocalLLMClient {
    config: LLMConfig,
    http_client: reqwest::Client,
    conversation_history: Arc<RwLock<Vec<ChatMessage>>>,
    total_requests: Arc<RwLock<usize>>,
    total_tokens: Arc<RwLock<usize>>,
}

impl LocalLLMClient {
    pub async fn new(mut config: LLMConfig) -> Result<Arc<Self>, AIError> {
        config.resolve_provider();

        tracing::info!(
            "[LLM] Provider: {}, model: {}, endpoint: {}, ctx: {}",
            config.provider,
            config.model,
            config.endpoint,
            config.context_window
        );

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                AIError::ConfigurationError(format!("Failed to create HTTP client: {}", e))
            })?;

        let client = Self {
            config,
            http_client,
            conversation_history: Arc::new(RwLock::new(Vec::new())),
            total_requests: Arc::new(RwLock::new(0)),
            total_tokens: Arc::new(RwLock::new(0)),
        };

        Ok(Arc::new(client))
    }

    pub async fn health_check(&self) -> bool {
        if self.config.provider == "anthropic" {
            if self.config.api_key.is_empty() {
                tracing::warn!("[LLM] Anthropic provider configured but no api_key set");
                return false;
            }
            tracing::info!(
                "[LLM] Anthropic provider configured with model={}",
                self.config.model
            );
            return true;
        }
        let url = format!("{}/api/tags", self.config.endpoint);
        match self.http_client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<LLMResponse, AIError> {
        let start_time = std::time::Instant::now();

        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
                "num_ctx": self.config.context_window,
            }
        });

        let url = format!("{}/api/generate", self.config.endpoint);

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::InferenceFailed(format!(
                "LLM request failed with status: {}",
                response.status()
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Failed to parse response: {}", e)))?;

        let text = response_body["response"].as_str().unwrap_or("").to_string();

        let tokens_used = response_body["eval_count"].as_u64().unwrap_or(0) as usize;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        *self.total_requests.write().await += 1;
        *self.total_tokens.write().await += tokens_used;

        Ok(LLMResponse {
            text,
            model: self.config.model.clone(),
            tokens_used,
            processing_time_ms,
            finish_reason: FinishReason::Stop,
        })
    }

    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<LLMResponse, AIError> {
        if self.config.provider == "anthropic" {
            return self.chat_anthropic(messages).await;
        }
        self.chat_ollama(messages).await
    }

    pub async fn chat_with_image(
        &self,
        system_prompt: &str,
        user_text: &str,
        image_data: &[u8],
        media_type: &str,
    ) -> Result<LLMResponse, AIError> {
        if self.config.provider != "anthropic" {
            return self
                .chat_with_image_ollama(system_prompt, user_text, image_data, media_type)
                .await;
        }

        let start_time = std::time::Instant::now();
        let b64 = base64_engine::STANDARD.encode(image_data);

        let effective_max_tokens = if self.config.max_tokens <= 2048 {
            8192
        } else {
            self.config.max_tokens
        };

        let request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": effective_max_tokens,
            "temperature": self.config.temperature,
            "system": system_prompt,
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": b64,
                        }
                    },
                    {
                        "type": "text",
                        "text": user_text,
                    }
                ]
            }]
        });

        let endpoint = if self.config.endpoint.contains("anthropic.com") {
            self.config.endpoint.clone()
        } else {
            "https://api.anthropic.com".to_string()
        };
        let url = format!("{}/v1/messages", endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Vision request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AIError::InferenceFailed(format!(
                "Vision API error {}: {}",
                status, body
            )));
        }

        let response_body: serde_json::Value = response.json().await.map_err(|e| {
            AIError::InferenceFailed(format!("Failed to parse vision response: {}", e))
        })?;

        let text = response_body["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = response_body["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize;
        let output_tokens = response_body["usage"]["output_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;
        let tokens_used = input_tokens + output_tokens;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        *self.total_requests.write().await += 1;
        *self.total_tokens.write().await += tokens_used;

        tracing::info!(
            "[LLM] Vision response: {}ms, {} tokens (in={}, out={})",
            processing_time_ms,
            tokens_used,
            input_tokens,
            output_tokens
        );

        Ok(LLMResponse {
            text,
            model: self.config.model.clone(),
            tokens_used,
            processing_time_ms,
            finish_reason: FinishReason::Stop,
        })
    }

    async fn chat_with_image_ollama(
        &self,
        system_prompt: &str,
        user_text: &str,
        image_data: &[u8],
        _media_type: &str,
    ) -> Result<LLMResponse, AIError> {
        let start_time = std::time::Instant::now();
        let b64 = base64_engine::STANDARD.encode(image_data);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_text, "images": [b64]}
            ],
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
                "num_ctx": self.config.context_window,
            }
        });

        let url = format!("{}/api/chat", self.config.endpoint);
        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                AIError::InferenceFailed(format!("Ollama vision request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AIError::InferenceFailed(format!(
                "Ollama vision error: {}",
                response.status()
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Parse failed: {}", e)))?;

        let text = response_body["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let tokens_used = response_body["eval_count"].as_u64().unwrap_or(0) as usize;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        *self.total_requests.write().await += 1;
        *self.total_tokens.write().await += tokens_used;

        Ok(LLMResponse {
            text,
            model: self.config.model.clone(),
            tokens_used,
            processing_time_ms,
            finish_reason: FinishReason::Stop,
        })
    }

    async fn chat_anthropic(&self, messages: &[ChatMessage]) -> Result<LLMResponse, AIError> {
        let start_time = std::time::Instant::now();

        let system_text: Option<String> = messages
            .iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.clone());

        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "user",
                    },
                    "content": msg.content,
                })
            })
            .collect();

        let effective_max_tokens = if self.config.max_tokens <= 2048 {
            8192
        } else {
            self.config.max_tokens
        };

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": effective_max_tokens,
            "temperature": self.config.temperature,
            "messages": api_messages,
        });

        if let Some(sys) = &system_text {
            request_body["system"] = serde_json::Value::String(sys.clone());
        }

        let endpoint = if self.config.endpoint.contains("anthropic.com") {
            self.config.endpoint.clone()
        } else {
            "https://api.anthropic.com".to_string()
        };
        let url = format!("{}/v1/messages", endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Anthropic request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("[LLM] Anthropic API error {}: {}", status, body);
            return Err(AIError::InferenceFailed(format!(
                "Anthropic API returned {}: {}",
                status, body
            )));
        }

        let response_body: serde_json::Value = response.json().await.map_err(|e| {
            AIError::InferenceFailed(format!("Failed to parse Anthropic response: {}", e))
        })?;

        let text = response_body["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = response_body["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize;
        let output_tokens = response_body["usage"]["output_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;
        let tokens_used = input_tokens + output_tokens;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        *self.total_requests.write().await += 1;
        *self.total_tokens.write().await += tokens_used;

        tracing::info!(
            "[LLM] Anthropic response: {}ms, {} tokens (in={}, out={})",
            processing_time_ms,
            tokens_used,
            input_tokens,
            output_tokens
        );

        Ok(LLMResponse {
            text,
            model: self.config.model.clone(),
            tokens_used,
            processing_time_ms,
            finish_reason: FinishReason::Stop,
        })
    }

    async fn chat_ollama(&self, messages: &[ChatMessage]) -> Result<LLMResponse, AIError> {
        let start_time = std::time::Instant::now();

        let formatted_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        MessageRole::System => "system",
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                    },
                    "content": msg.content,
                })
            })
            .collect();

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": formatted_messages,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
                "num_ctx": self.config.context_window,
            }
        });

        let url = format!("{}/api/chat", self.config.endpoint);

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::InferenceFailed(format!(
                "LLM chat request failed with status: {}",
                response.status()
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Failed to parse response: {}", e)))?;

        let text = response_body["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tokens_used = response_body["eval_count"].as_u64().unwrap_or(0) as usize;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        *self.total_requests.write().await += 1;
        *self.total_tokens.write().await += tokens_used;

        Ok(LLMResponse {
            text,
            model: self.config.model.clone(),
            tokens_used,
            processing_time_ms,
            finish_reason: FinishReason::Stop,
        })
    }

    pub async fn generate_with_system(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LLMResponse, AIError> {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_prompt.to_string(),
            },
        ];

        self.chat(&messages).await
    }

    pub async fn continue_conversation(&self, user_message: &str) -> Result<LLMResponse, AIError> {
        let mut history = self.conversation_history.write().await;

        history.push(ChatMessage {
            role: MessageRole::User,
            content: user_message.to_string(),
        });

        let messages: Vec<ChatMessage> = history.clone();
        drop(history);

        let response = self.chat(&messages).await?;

        let mut history = self.conversation_history.write().await;
        history.push(ChatMessage {
            role: MessageRole::Assistant,
            content: response.text.clone(),
        });

        Ok(response)
    }

    pub async fn clear_conversation(&self) {
        self.conversation_history.write().await.clear();
    }

    pub async fn set_system_prompt(&self, system_prompt: &str) {
        let mut history = self.conversation_history.write().await;
        history.clear();
        history.push(ChatMessage {
            role: MessageRole::System,
            content: system_prompt.to_string(),
        });
    }

    pub async fn analyze_content(&self, content: &str) -> Result<ContentAnalysisResult, AIError> {
        let system_prompt = "You are an expert content analyzer. Analyze the provided content and return a structured analysis including quality score (0-1), key themes, sentiment, and suggestions for improvement.";

        let user_prompt = format!(
            "Analyze this content:\n\n{}\n\nProvide your analysis in a structured format.",
            content
        );

        let response = self
            .generate_with_system(system_prompt, &user_prompt)
            .await?;

        Ok(ContentAnalysisResult {
            raw_analysis: response.text,
            model_used: response.model,
            processing_time_ms: response.processing_time_ms,
        })
    }

    pub async fn generate_description(&self, prompt: &str) -> Result<String, AIError> {
        let system_prompt = "You are a creative writer specializing in descriptions for virtual world content. Generate vivid, engaging descriptions.";

        let response = self.generate_with_system(system_prompt, prompt).await?;
        Ok(response.text)
    }

    pub async fn suggest_improvements(&self, content: &str) -> Result<Vec<String>, AIError> {
        let system_prompt = "You are an expert reviewer. Analyze the provided content and suggest specific, actionable improvements. Return each suggestion on a new line.";

        let user_prompt = format!(
            "Review this content and suggest improvements:\n\n{}",
            content
        );

        let response = self
            .generate_with_system(system_prompt, &user_prompt)
            .await?;

        let suggestions: Vec<String> = response
            .text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('-')
                    || trimmed.starts_with('*')
                    || trimmed.starts_with(char::is_numeric)
                {
                    trimmed
                        .trim_start_matches(|c: char| {
                            c == '-' || c == '*' || c == '.' || c.is_numeric() || c.is_whitespace()
                        })
                        .to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        Ok(suggestions)
    }

    pub async fn get_stats(&self) -> LLMStats {
        LLMStats {
            total_requests: *self.total_requests.read().await,
            total_tokens: *self.total_tokens.read().await,
            model: self.config.model.clone(),
            endpoint: self.config.endpoint.clone(),
        }
    }

    pub fn get_config(&self) -> &LLMConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysisResult {
    pub raw_analysis: String,
    pub model_used: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMStats {
    pub total_requests: usize,
    pub total_tokens: usize,
    pub model: String,
    pub endpoint: String,
}
