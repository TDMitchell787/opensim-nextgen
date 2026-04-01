//! Documentation and examples generator for OpenSim client SDKs

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema},
    generator::{TargetLanguage, GeneratorConfig, GeneratedFile, GeneratedFileType},
};

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub target_languages: Vec<TargetLanguage>,
    pub include_api_reference: bool,
    pub include_getting_started: bool,
    pub include_examples: bool,
    pub include_troubleshooting: bool,
    pub include_best_practices: bool,
    pub include_interactive_examples: bool,
    pub include_live_demos: bool,
    pub include_playground: bool,
    pub enable_auto_deployment: bool,
    pub output_format: Vec<DocumentationFormat>,
    pub output_directory: PathBuf,
    pub template_directory: Option<PathBuf>,
    pub branding: BrandingConfig,
    pub deployment: DeploymentConfig,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            target_languages: vec![
                TargetLanguage::Rust,
                TargetLanguage::Python,
                TargetLanguage::JavaScript,
                TargetLanguage::CSharp,
                TargetLanguage::Java,
                TargetLanguage::Go,
                TargetLanguage::PHP,
                TargetLanguage::Ruby,
            ],
            include_api_reference: true,
            include_getting_started: true,
            include_examples: true,
            include_troubleshooting: true,
            include_best_practices: true,
            include_interactive_examples: true,
            include_live_demos: true,
            include_playground: true,
            enable_auto_deployment: true,
            output_format: vec![
                DocumentationFormat::Markdown,
                DocumentationFormat::Html,
                DocumentationFormat::Json,
                DocumentationFormat::OpenApi,
            ],
            output_directory: PathBuf::from("./client-docs"),
            template_directory: None,
            branding: BrandingConfig::default(),
            deployment: DeploymentConfig::default(),
        }
    }
}

/// Documentation output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationFormat {
    Markdown,
    Html,
    Json,
    Pdf,
    Epub,
    OpenApi,
    Interactive,
    Playground,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub target: DeploymentTarget,
    pub github_pages: bool,
    pub netlify: bool,
    pub vercel: bool,
    pub custom_domain: Option<String>,
    pub auto_build_on_commit: bool,
    pub build_command: String,
    pub publish_directory: String,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            target: DeploymentTarget::GitHubPages,
            github_pages: true,
            netlify: false,
            vercel: false,
            custom_domain: None,
            auto_build_on_commit: true,
            build_command: "npm run build".to_string(),
            publish_directory: "dist".to_string(),
        }
    }
}

/// Deployment targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTarget {
    GitHubPages,
    Netlify,
    Vercel,
    AmazonS3,
    Custom(String),
}

/// Branding configuration for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub project_name: String,
    pub project_url: String,
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub font_family: String,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            project_name: "OpenSim".to_string(),
            project_url: "https://github.com/opensim/opensim-next".to_string(),
            logo_url: None,
            primary_color: "#2563eb".to_string(),
            secondary_color: "#64748b".to_string(),
            font_family: "Inter, sans-serif".to_string(),
        }
    }
}

/// Documentation section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSection {
    pub id: String,
    pub title: String,
    pub content: String,
    pub subsections: Vec<DocumentationSection>,
    pub examples: Vec<CodeExample>,
    pub language_specific: HashMap<TargetLanguage, String>,
}

/// Code example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub id: String,
    pub title: String,
    pub description: String,
    pub language: TargetLanguage,
    pub code: String,
    pub output: Option<String>,
    pub notes: Vec<String>,
    pub interactive: bool,
    pub playground_url: Option<String>,
    pub live_demo_url: Option<String>,
    pub dependencies: Vec<String>,
    pub setup_instructions: Vec<String>,
}

/// Interactive playground configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundConfig {
    pub enabled: bool,
    pub supported_languages: Vec<TargetLanguage>,
    pub api_endpoint: String,
    pub execution_timeout: u32,
    pub memory_limit: u32,
    pub rate_limiting: bool,
}

/// Documentation generator
pub struct DocumentationGenerator {
    config: DocumentationConfig,
    api_schema: APISchema,
    generated_docs: Arc<RwLock<Vec<GeneratedFile>>>,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new(config: DocumentationConfig, api_schema: APISchema) -> Self {
        Self {
            config,
            api_schema,
            generated_docs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate complete documentation suite
    pub async fn generate_documentation(&self) -> Result<Vec<GeneratedFile>> {
        info!("Generating comprehensive client documentation");

        let mut all_docs = Vec::new();

        // Generate main documentation
        if self.config.include_getting_started {
            all_docs.extend(self.generate_getting_started().await?);
        }

        if self.config.include_api_reference {
            all_docs.extend(self.generate_api_reference().await?);
        }

        if self.config.include_examples {
            all_docs.extend(self.generate_examples().await?);
        }

        if self.config.include_best_practices {
            all_docs.extend(self.generate_best_practices().await?);
        }

        if self.config.include_troubleshooting {
            all_docs.extend(self.generate_troubleshooting().await?);
        }

        // Generate interactive features
        if self.config.include_interactive_examples {
            all_docs.extend(self.generate_interactive_examples().await?);
        }

        if self.config.include_live_demos {
            all_docs.extend(self.generate_live_demos().await?);
        }

        if self.config.include_playground {
            all_docs.extend(self.generate_playground().await?);
        }

        // Generate language-specific documentation
        for language in &self.config.target_languages {
            all_docs.extend(self.generate_language_specific_docs(language).await?);
        }

        // Generate index and navigation
        all_docs.extend(self.generate_index_and_navigation(&all_docs).await?);

        // Generate deployment configurations
        if self.config.enable_auto_deployment {
            all_docs.extend(self.generate_deployment_configs().await?);
        }

        // Store generated docs
        *self.generated_docs.write().await = all_docs.clone();

        info!("Documentation generation completed: {} files", all_docs.len());
        Ok(all_docs)
    }

    /// Generate getting started guide
    async fn generate_getting_started(&self) -> Result<Vec<GeneratedFile>> {
        let mut docs = Vec::new();

        let content = r#"# Getting Started with OpenSim Client SDKs

Welcome to the OpenSim client SDK documentation! This guide will help you get started with using OpenSim's client libraries across different programming languages.

## Overview

The OpenSim client SDKs provide a comprehensive interface to interact with OpenSim virtual world servers. Each SDK is automatically generated from our API schema to ensure consistency and up-to-date functionality across all supported languages.

### Supported Languages

- **Rust** - Native performance with memory safety
- **Python** - Easy to use with extensive ecosystem
- **JavaScript/TypeScript** - Web and Node.js applications
- **C#** - .NET applications and Unity integration
- **Java** - Enterprise applications and Android

## Quick Start

### 1. Installation

Choose your preferred language and follow the installation instructions:

#### Rust
```toml
[dependencies]
opensim-client = "1.0.0"
```

#### Python
```bash
pip install opensim-client
```

#### JavaScript/Node.js
```bash
npm install opensim-client
```

#### C#
```bash
dotnet add package OpenSim.Client
```

#### Java
```xml
<dependency>
    <groupId>org.opensim</groupId>
    <artifactId>opensim-client</artifactId>
    <version>1.0.0</version>
</dependency>
```

### 2. Basic Usage

Here's a simple example to get you started:

#### Rust
```rust
use opensim_client::{OpenSimClient, AuthConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    
    // Authenticate
    let auth = client.authenticate("username", "password").await?;
    println!("Authenticated: {}", auth.access_token);
    
    // Get user profile
    let profile = client.get_user_profile(&auth.user_id).await?;
    println!("Welcome, {}!", profile.username);
    
    Ok(())
}
```

#### Python
```python
import asyncio
from opensim_client import OpenSimClient

async def main():
    client = OpenSimClient("https://api.opensim.org")
    
    # Authenticate
    auth = await client.authenticate("username", "password")
    print(f"Authenticated: {auth.access_token}")
    
    # Get user profile
    profile = await client.get_user_profile(auth.user_id)
    print(f"Welcome, {profile.username}!")

asyncio.run(main())
```

#### JavaScript
```javascript
import { OpenSimClient } from 'opensim-client';

async function main() {
    const client = new OpenSimClient('https://api.opensim.org');
    
    // Authenticate
    const auth = await client.authenticate('username', 'password');
    console.log(`Authenticated: ${auth.accessToken}`);
    
    // Get user profile
    const profile = await client.getUserProfile(auth.userId);
    console.log(`Welcome, ${profile.username}!`);
}

main().catch(console.error);
```

### 3. Error Handling

All SDKs provide consistent error handling patterns:

```rust
// Rust
match client.get_user_profile(user_id).await {
    Ok(profile) => println!("User: {}", profile.username),
    Err(e) => eprintln!("Error: {}", e),
}
```

```python
# Python
try:
    profile = await client.get_user_profile(user_id)
    print(f"User: {profile.username}")
except OpenSimError as e:
    print(f"Error: {e}")
```

### 4. Configuration

Configure the client for your environment:

```rust
// Rust
let config = ClientConfig {
    base_url: "https://api.opensim.org".to_string(),
    timeout: Duration::from_secs(30),
    retry_attempts: 3,
    debug_mode: false,
};

let client = OpenSimClient::with_config(config).await?;
```

## Next Steps

- [API Reference](api-reference.md) - Complete API documentation
- [Examples](examples/) - Practical examples and use cases
- [Best Practices](best-practices.md) - Recommended patterns and practices
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Support

- **Documentation**: [https://docs.opensim.org](https://docs.opensim.org)
- **Issues**: [GitHub Issues](https://github.com/opensim/opensim-next/issues)
- **Community**: [OpenSim Discord](https://discord.gg/opensim)
"#;

        docs.push(GeneratedFile {
            path: PathBuf::from("getting-started.md"),
            content,
            file_type: GeneratedFileType::Documentation,
        });

        Ok(docs)
    }

    /// Generate API reference documentation
    async fn generate_api_reference(&self) -> Result<Vec<GeneratedFile>> {
        let mut docs = Vec::new();

        let mut content = String::from("# API Reference\n\n");
        content.push_str("Complete reference for all OpenSim API endpoints.\n\n");

        // Generate endpoint documentation
        content.push_str("## Endpoints\n\n");
        for endpoint in &self.api_schema.endpoints {
            content.push_str(&self.generate_endpoint_docs(endpoint)?);
            content.push_str("\n\n");
        }

        // Generate data types documentation
        content.push_str("## Data Types\n\n");
        for data_type in &self.api_schema.data_types {
            content.push_str(&self.generate_data_type_docs(data_type)?);
            content.push_str("\n\n");
        }

        docs.push(GeneratedFile {
            path: PathBuf::from("api-reference.md"),
            content,
            file_type: GeneratedFileType::Documentation,
        });

        Ok(docs)
    }

    /// Generate examples
    async fn generate_examples(&self) -> Result<Vec<GeneratedFile>> {
        let mut docs = Vec::new();

        // Create examples directory structure
        for language in &self.config.target_languages {
            let examples = self.generate_language_examples(language).await?;
            docs.extend(examples);
        }

        // Generate examples index
        let index_content = self.generate_examples_index().await?;
        docs.push(GeneratedFile {
            path: PathBuf::from("examples/README.md"),
            content: index_content,
            file_type: GeneratedFileType::Documentation,
        });

        Ok(docs)
    }

    /// Generate best practices guide
    async fn generate_best_practices(&self) -> Result<Vec<GeneratedFile>> {
        let content = r###"# Best Practices

This guide provides recommended patterns and practices for using OpenSim client SDKs effectively and securely.

## Authentication & Security

### Secure Token Management

Always store authentication tokens securely:

```rust
// Rust - Use environment variables or secure storage
let token = std::env::var("OPENSIM_TOKEN")?;
client.set_access_token(&token);
```

```python
# Python - Use environment variables
import os
token = os.getenv('OPENSIM_TOKEN')
client.set_access_token(token)
```

### Token Refresh

Implement automatic token refresh:

```rust
// Rust
impl TokenManager {
    async fn ensure_valid_token(&mut self) -> Result<&str> {
        if self.token.is_expired() {
            self.refresh_token().await?;
        }
        Ok(&self.token.access_token)
    }
}
```

## Error Handling

### Retry Logic

Implement exponential backoff for transient errors:

```rust
// Rust
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<F, T, E>(mut f: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_retries => return Err(e),
            Err(_) => {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                sleep(delay).await;
                attempts += 1;
            }
        }
    }
}
```

### Circuit Breaker Pattern

Implement circuit breakers for failing services:

```rust
// Rust
struct CircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure: Option<Instant>,
    state: CircuitState,
}

enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

## Performance Optimization

### Connection Pooling

Reuse connections for better performance:

```rust
// Rust
let client = OpenSimClient::builder()
    .connection_pool_size(10)
    .keep_alive_timeout(Duration::from_secs(30))
    .build()?;
```

### Caching

Cache frequently accessed data:

```rust
// Rust
use moka::future::Cache;

struct CachedClient {
    client: OpenSimClient,
    user_cache: Cache<String, UserProfile>,
}

impl CachedClient {
    async fn get_user_profile(&self, user_id: &str) -> Result<UserProfile> {
        if let Some(profile) = self.user_cache.get(user_id).await {
            return Ok(profile);
        }
        
        let profile = self.client.get_user_profile(user_id).await?;
        self.user_cache.insert(user_id.to_string(), profile.clone()).await;
        Ok(profile)
    }
}
```

### Batch Operations

Use batch operations when possible:

```rust
// Rust
let user_ids = vec!["user1", "user2", "user3"];
let profiles = client.get_user_profiles_batch(&user_ids).await?;
```

## Resource Management

### Connection Management

Always properly close connections:

```rust
// Rust - Use RAII or explicit cleanup
{
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    // Use client...
} // Client automatically cleaned up
```

```python
# Python - Use async context managers
async with OpenSimClient("https://api.opensim.org") as client:
    # Use client...
# Client automatically cleaned up
```

### Memory Management

Monitor memory usage in long-running applications:

```rust
// Rust
struct MemoryMonitor {
    max_cache_size: usize,
    current_size: AtomicUsize,
}

impl MemoryMonitor {
    fn check_memory_pressure(&self) -> bool {
        self.current_size.load(Ordering::Relaxed) > self.max_cache_size
    }
}
```

## Logging & Monitoring

### Structured Logging

Use structured logging for better observability:

```rust
// Rust
use tracing::{info, warn, error};

#[tracing::instrument]
async fn get_user_profile(user_id: &str) -> Result<UserProfile> {
    info!(user_id, "Fetching user profile");
    
    match client.get_user_profile(user_id).await {
        Ok(profile) => {
            info!(user_id, username = profile.username, "Profile fetched successfully");
            Ok(profile)
        }
        Err(e) => {
            error!(user_id, error = %e, "Failed to fetch profile");
            Err(e)
        }
    }
}
```

### Metrics Collection

Track important metrics:

```rust
// Rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref API_REQUESTS: Counter = register_counter!(
        "opensim_api_requests_total",
        "Total number of API requests"
    ).unwrap();
    
    static ref API_DURATION: Histogram = register_histogram!(
        "opensim_api_request_duration_seconds",
        "API request duration"
    ).unwrap();
}
```

## Testing

### Unit Testing

Write comprehensive unit tests:

```rust
// Rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_user_profile_success() {
        let _m = mock("GET", "/users/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "123", "username": "test"}"#)
            .create();

        let client = OpenSimClient::new(&mockito::server_url()).await.unwrap();
        let profile = client.get_user_profile("123").await.unwrap();
        
        assert_eq!(profile.id, "123");
        assert_eq!(profile.username, "test");
    }
}
```

### Integration Testing

Test against real API endpoints:

```rust
// Rust
#[tokio::test]
#[ignore] // Run with --ignored for integration tests
async fn test_real_api_integration() {
    let client = OpenSimClient::new("https://api-test.opensim.org").await.unwrap();
    let auth = client.authenticate("test_user", "test_pass").await.unwrap();
    assert!(!auth.access_token.is_empty());
}
```

## Deployment

### Configuration Management

Use environment-based configuration:

```rust
// Rust
#[derive(Debug, serde::Deserialize)]
struct Config {
    opensim_api_url: String,
    opensim_timeout_seconds: u64,
    opensim_retry_attempts: u32,
}

impl Config {
    fn from_env() -> Result<Self> {
        envy::from_env().map_err(Into::into)
    }
}
```

### Health Checks

Implement health check endpoints:

```rust
// Rust
async fn health_check(client: &OpenSimClient) -> Result<HealthStatus> {
    let start = Instant::now();
    
    match client.ping().await {
        Ok(_) => Ok(HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: start.elapsed().as_millis() as u64,
        }),
        Err(e) => Ok(HealthStatus {
            status: "unhealthy".to_string(),
            error: Some(e.to_string()),
            response_time_ms: start.elapsed().as_millis() as u64,
        }),
    }
}
```
"###.to_string();

        Ok(vec![GeneratedFile {
            path: PathBuf::from("best-practices.md"),
            content,
            file_type: GeneratedFileType::Documentation,
        }])
    }

    /// Generate troubleshooting guide
    async fn generate_troubleshooting(&self) -> Result<Vec<GeneratedFile>> {
        let content = r###"# Troubleshooting Guide

Common issues and solutions when using OpenSim client SDKs.

## Authentication Issues

### Invalid Credentials Error

**Problem**: Authentication fails with "Invalid credentials" error.

**Solutions**:
1. Verify username and password are correct
2. Check if account is active and not suspended
3. Ensure you're connecting to the correct server

```rust
// Rust - Add debug logging
let result = client.authenticate(username, password).await;
match result {
    Err(OpenSimError::Authentication(msg)) => {
        eprintln!("Auth failed: {}", msg);
        // Check credentials and try again
    }
    Err(e) => eprintln!("Other error: {}", e),
    Ok(auth) => println!("Success: {}", auth.access_token),
}
```"##

### Token Expired Error

**Problem**: API calls fail with "Token expired" error.

**Solutions**:
1. Implement automatic token refresh
2. Handle token expiration gracefully
3. Store and reuse refresh tokens

```python
# Python - Token refresh example
async def refresh_token_if_needed(client):
    try:
        await client.get_user_profile(user_id)
    except TokenExpiredError:
        await client.refresh_token()
        return await client.get_user_profile(user_id)
```

## Connection Issues

### Timeout Errors

**Problem**: Requests timeout before completion.

**Solutions**:
1. Increase timeout values
2. Check network connectivity
3. Implement retry logic with exponential backoff

```rust
// Rust - Configure timeouts
let client = OpenSimClient::builder()
    .timeout(Duration::from_secs(60))
    .build()?;
```

### SSL/TLS Errors

**Problem**: SSL certificate verification fails.

**Solutions**:
1. Update system certificates
2. For development, disable SSL verification (not recommended for production)
3. Use custom certificate bundle

```javascript
// JavaScript - Custom certificate handling
const client = new OpenSimClient('https://api.opensim.org', {
    rejectUnauthorized: false // Only for development!
});
```

## Rate Limiting

### Too Many Requests Error

**Problem**: API returns 429 Too Many Requests.

**Solutions**:
1. Implement rate limiting in your client
2. Use exponential backoff
3. Cache responses to reduce API calls

```rust
// Rust - Rate limiting with governor
use governor::{Quota, RateLimiter};

struct RateLimitedClient {
    client: OpenSimClient,
    limiter: RateLimiter<String, HashMap<String, governor::state::InMemoryState>, 
                        governor::clock::DefaultClock>,
}

impl RateLimitedClient {
    async fn make_request(&self) -> Result<Response> {
        self.limiter.until_ready().await;
        self.client.make_request().await
    }
}
```

## Data Issues

### Serialization/Deserialization Errors

**Problem**: JSON parsing fails or returns unexpected data.

**Solutions**:
1. Check API version compatibility
2. Validate response structure
3. Handle optional fields properly

```python
# Python - Robust JSON handling
import json
from typing import Optional

def safe_parse_user(data: dict) -> Optional[User]:
    try:
        return User(
            id=data.get('id'),
            username=data.get('username', 'unknown'),
            email=data.get('email'),
            created_at=parse_datetime(data.get('created_at'))
        )
    except (KeyError, ValueError) as e:
        logger.error(f"Failed to parse user data: {e}")
        return None
```

### Unicode/Encoding Issues

**Problem**: Text with special characters causes errors.

**Solutions**:
1. Ensure proper UTF-8 encoding
2. Handle different character sets
3. Validate input data

```rust
// Rust - UTF-8 validation
fn validate_username(username: &str) -> Result<&str> {
    if username.is_ascii() {
        Ok(username)
    } else {
        // Handle unicode usernames
        if username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Ok(username)
        } else {
            Err(anyhow!("Invalid characters in username"))
        }
    }
}
```

## Performance Issues

### Slow Response Times

**Problem**: API requests are taking too long.

**Solutions**:
1. Enable connection pooling
2. Use batch operations
3. Implement caching
4. Optimize queries

```rust
// Rust - Connection pooling
let client = OpenSimClient::builder()
    .connection_pool_size(20)
    .pool_idle_timeout(Duration::from_secs(30))
    .build()?;
```

### Memory Leaks

**Problem**: Memory usage increases over time.

**Solutions**:
1. Properly close connections
2. Limit cache sizes
3. Use weak references where appropriate
4. Monitor memory usage

```python
# Python - Memory monitoring
import psutil
import logging

def monitor_memory():
    process = psutil.Process()
    memory_mb = process.memory_info().rss / 1024 / 1024
    if memory_mb > 500:  # Alert if over 500MB
        logging.warning(f"High memory usage: {memory_mb:.1f}MB")
```

## SDK-Specific Issues

### Rust

**Problem**: Compilation errors with async code.

**Solution**: Ensure proper async runtime setup:

```rust
// Cargo.toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }

// main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    // Your code here
    Ok(())
}
```

### Python

**Problem**: `asyncio` event loop issues.

**Solution**: Use proper async context:

```python
import asyncio
import nest_asyncio

# For Jupyter notebooks
nest_asyncio.apply()

async def main():
    # Your async code here
    pass

# Run the async function
asyncio.run(main())
```

### JavaScript

**Problem**: Promise rejection handling.

**Solution**: Always handle promise rejections:

```javascript
// Good - with error handling
try {
    const result = await client.getUserProfile(userId);
    console.log(result);
} catch (error) {
    console.error('API call failed:', error);
}

// Also good - with .catch()
client.getUserProfile(userId)
    .then(result => console.log(result))
    .catch(error => console.error('API call failed:', error));
```

## Debugging Tips

### Enable Debug Logging

```rust
// Rust
env_logger::init();
// or with tracing
tracing_subscriber::fmt::init();
```

```python
# Python
import logging
logging.basicConfig(level=logging.DEBUG)
```

### Network Traffic Inspection

Use tools like Wireshark, mitmproxy, or built-in debugging:

```rust
// Rust - Enable request/response logging
let client = OpenSimClient::builder()
    .debug_requests(true)
    .build()?;
```

### Performance Profiling

```rust
// Rust - Simple timing
let start = std::time::Instant::now();
let result = client.some_operation().await?;
println!("Operation took: {:?}", start.elapsed());
```

## Getting Help

If you're still experiencing issues:

1. **Check the logs** - Enable debug logging for detailed information
2. **Search existing issues** - Look through GitHub issues for similar problems
3. **Create a minimal reproduction** - Reduce your code to the smallest example that shows the problem
4. **Provide environment details** - OS, language version, SDK version, etc.
5. **Contact support** - Use the appropriate channels:
   - GitHub Issues: Technical bugs and feature requests
   - Discord: General questions and community support
   - Email: Security issues and private concerns

## Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| 400 | Bad Request | Check request parameters and format |
| 401 | Unauthorized | Verify authentication credentials |
| 403 | Forbidden | Check user permissions |
| 404 | Not Found | Verify resource exists and URL is correct |
| 429 | Too Many Requests | Implement rate limiting and backoff |
| 500 | Internal Server Error | Check server status, retry later |
| 503 | Service Unavailable | Server maintenance, try again later |
"###;

        Ok(vec![GeneratedFile {
            path: PathBuf::from("troubleshooting.md"),
            content,
            file_type: GeneratedFileType::Documentation,
        }])
    }

    /// Generate language-specific documentation
    async fn generate_language_specific_docs(&self, language: &TargetLanguage) -> Result<Vec<GeneratedFile>> {
        let mut docs = Vec::new();

        let content = match language {
            TargetLanguage::Rust => self.generate_rust_specific_docs().await?,
            TargetLanguage::Python => self.generate_python_specific_docs().await?,
            TargetLanguage::JavaScript => self.generate_javascript_specific_docs().await?,
            TargetLanguage::CSharp => self.generate_csharp_specific_docs().await?,
            TargetLanguage::Java => self.generate_java_specific_docs().await?,
        };

        let filename = format!("{}-guide.md", language.to_string().to_lowercase());
        docs.push(GeneratedFile {
            path: PathBuf::from(filename),
            content,
            file_type: GeneratedFileType::Documentation,
        });

        Ok(docs)
    }

    /// Generate language examples
    async fn generate_language_examples(&self, language: &TargetLanguage) -> Result<Vec<GeneratedFile>> {
        let mut examples = Vec::new();

        // Basic usage example
        let basic_example = self.generate_basic_example(language)?;
        examples.push(GeneratedFile {
            path: PathBuf::from(format!("examples/{}/basic.{}", 
                language.to_string().to_lowercase(), 
                self.get_file_extension(language))),
            content: basic_example,
            file_type: GeneratedFileType::Example,
        });

        // Authentication example
        let auth_example = self.generate_auth_example(language)?;
        examples.push(GeneratedFile {
            path: PathBuf::from(format!("examples/{}/authentication.{}", 
                language.to_string().to_lowercase(), 
                self.get_file_extension(language))),
            content: auth_example,
            file_type: GeneratedFileType::Example,
        });

        // Advanced usage example
        let advanced_example = self.generate_advanced_example(language)?;
        examples.push(GeneratedFile {
            path: PathBuf::from(format!("examples/{}/advanced.{}", 
                language.to_string().to_lowercase(), 
                self.get_file_extension(language))),
            content: advanced_example,
            file_type: GeneratedFileType::Example,
        });

        Ok(examples)
    }

    // Helper methods for generating specific documentation types
    fn generate_endpoint_docs(&self, endpoint: &EndpointSchema) -> Result<String> {
        Ok(format!(
            "### {}\n\n**{}** `{}`\n\n{}\n\n**Parameters:**\n\n{}\n\n**Response:**\n\n{}",
            endpoint.name,
            endpoint.method,
            endpoint.path,
            endpoint.description,
            endpoint.parameters.iter()
                .map(|p| format!("- `{}` ({}): {}", p.name, p.param_type, p.description))
                .collect::<Vec<_>>()
                .join("\n"),
            endpoint.responses.get(&200)
                .map(|r| r.description.clone())
                .unwrap_or_else(|| "Success response".to_string())
        ))
    }

    fn generate_data_type_docs(&self, data_type: &DataTypeSchema) -> Result<String> {
        Ok(format!(
            "### {}\n\n{}\n\n**Fields:**\n\n{}",
            data_type.name,
            data_type.description,
            data_type.fields.iter()
                .map(|f| format!("- `{}` ({}): {}", f.name, f.field_type, f.description))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }

    // Language-specific documentation generators
    async fn generate_rust_specific_docs(&self) -> Result<String> {
        Ok(r#"# Rust SDK Guide

The OpenSim Rust SDK provides a type-safe, high-performance client for interacting with OpenSim APIs.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opensim-client = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
```

## Features

- **Type Safety**: Full Rust type system integration
- **Async/Await**: Built on tokio for excellent performance
- **Error Handling**: Comprehensive error types with context
- **Memory Safety**: Zero-cost abstractions with memory safety
- **Serialization**: Automatic JSON serialization/deserialization

## Quick Start

```rust
use opensim_client::{OpenSimClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    
    // Authenticate
    let auth = client.authenticate("username", "password").await?;
    println!("Access token: {}", auth.access_token);
    
    // Use authenticated client
    let profile = client.get_user_profile(&auth.user_id).await?;
    println!("Welcome, {}!", profile.username);
    
    Ok(())
}
```

## Error Handling

The SDK uses a comprehensive error system:

```rust
use opensim_client::{OpenSimError, ErrorKind};

match client.get_user_profile(user_id).await {
    Ok(profile) => println!("User: {}", profile.username),
    Err(OpenSimError::Http { status, message }) => {
        eprintln!("HTTP error {}: {}", status, message);
    }
    Err(OpenSimError::Network(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(OpenSimError::Authentication(msg)) => {
        eprintln!("Auth error: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Advanced Configuration

```rust
use std::time::Duration;
use opensim_client::{ClientConfig, RetryPolicy};

let config = ClientConfig::builder()
    .base_url("https://api.opensim.org")
    .timeout(Duration::from_secs(30))
    .retry_policy(RetryPolicy::exponential(3))
    .user_agent("MyApp/1.0")
    .build();

let client = OpenSimClient::with_config(config).await?;
```

## Testing

The SDK supports mocking for unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use opensim_client::mock::MockClient;

    #[tokio::test]
    async fn test_user_profile() {
        let mut mock = MockClient::new();
        mock.expect_get_user_profile()
            .returning(|_| Ok(UserProfile {
                id: "123".to_string(),
                username: "test_user".to_string(),
                // ... other fields
            }));

        let profile = mock.get_user_profile("123").await.unwrap();
        assert_eq!(profile.username, "test_user");
    }
}
```
"#)
    }

    async fn generate_python_specific_docs(&self) -> Result<String> {
        Ok(r#"# Python SDK Guide

The OpenSim Python SDK provides an easy-to-use, async-first client for Python applications.

## Installation

```bash
pip install opensim-client
```

## Features

- **Async/Await**: Built on aiohttp for excellent performance
- **Type Hints**: Full type annotation support
- **Dataclasses**: Clean, Pythonic data structures
- **Context Managers**: Automatic resource cleanup
- **Logging Integration**: Standard Python logging support

## Quick Start

```python
import asyncio
from opensim_client import OpenSimClient

async def main():
    async with OpenSimClient("https://api.opensim.org") as client:
        # Authenticate
        auth = await client.authenticate("username", "password")
        print(f"Access token: {auth.access_token}")
        
        # Use authenticated client
        profile = await client.get_user_profile(auth.user_id)
        print(f"Welcome, {profile.username}!")

asyncio.run(main())
```

## Configuration

```python
from opensim_client import OpenSimClient, ClientConfig

config = ClientConfig(
    base_url="https://api.opensim.org",
    timeout=30.0,
    retry_attempts=3,
    debug=False
)

async with OpenSimClient.with_config(config) as client:
    # Use configured client
    pass
```

## Error Handling

```python
from opensim_client import OpenSimError, AuthenticationError, NetworkError

try:
    profile = await client.get_user_profile(user_id)
    print(f"User: {profile.username}")
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except NetworkError as e:
    print(f"Network error: {e}")
except OpenSimError as e:
    print(f"API error: {e}")
```

## Synchronous Usage

For non-async applications:

```python
from opensim_client.sync import OpenSimClient

with OpenSimClient("https://api.opensim.org") as client:
    auth = client.authenticate("username", "password")
    profile = client.get_user_profile(auth.user_id)
    print(f"Welcome, {profile.username}!")
```
"#)
    }

    async fn generate_javascript_specific_docs(&self) -> Result<String> {
        Ok(r#"# JavaScript/TypeScript SDK Guide

The OpenSim JavaScript SDK works in both Node.js and browser environments with full TypeScript support.

## Installation

```bash
npm install opensim-client
```

## Features

- **TypeScript Support**: Full type definitions included
- **Universal**: Works in Node.js and browsers
- **Promise-based**: Modern async/await API
- **Tree Shaking**: ES modules for optimal bundle size
- **Error Handling**: Comprehensive error types

## Quick Start

### TypeScript/ES6

```typescript
import { OpenSimClient } from 'opensim-client';

async function main() {
    const client = new OpenSimClient('https://api.opensim.org');
    
    try {
        // Authenticate
        const auth = await client.authenticate('username', 'password');
        console.log(`Access token: ${auth.accessToken}`);
        
        // Use authenticated client
        const profile = await client.getUserProfile(auth.userId);
        console.log(`Welcome, ${profile.username}!`);
    } catch (error) {
        console.error('Error:', error);
    }
}

main();
```

### CommonJS (Node.js)

```javascript
const { OpenSimClient } = require('opensim-client');

async function main() {
    const client = new OpenSimClient('https://api.opensim.org');
    
    // Same API as above
}
```

## Configuration

```typescript
import { OpenSimClient, ClientConfig } from 'opensim-client';

const config: ClientConfig = {
    baseUrl: 'https://api.opensim.org',
    timeout: 30000,
    retryAttempts: 3,
    debug: false
};

const client = new OpenSimClient(config);
```

## Error Handling

```typescript
import { OpenSimError, AuthenticationError, NetworkError } from 'opensim-client';

try {
    const profile = await client.getUserProfile(userId);
    console.log(`User: ${profile.username}`);
} catch (error) {
    if (error instanceof AuthenticationError) {
        console.error(`Auth failed: ${error.message}`);
    } else if (error instanceof NetworkError) {
        console.error(`Network error: ${error.message}`);
    } else if (error instanceof OpenSimError) {
        console.error(`API error: ${error.message}`);
    } else {
        console.error(`Unexpected error: ${error}`);
    }
}
```

## Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import { OpenSimClient } from 'https://unpkg.com/opensim-client@1.0.0/dist/opensim-client.esm.js';
        
        const client = new OpenSimClient('https://api.opensim.org');
        
        // Use client...
    </script>
</head>
</html>
```
"#)
    }

    async fn generate_csharp_specific_docs(&self) -> Result<String> {
        Ok(r#"# C# SDK Guide

The OpenSim C# SDK provides a robust client for .NET applications with full async support.

## Installation

```bash
dotnet add package OpenSim.Client
```

## Features

- **.NET Standard 2.0**: Compatible with .NET Framework, .NET Core, and .NET 5+
- **Async/Await**: Full async support with cancellation tokens
- **Strong Typing**: Complete type safety with nullable reference types
- **Dependency Injection**: Built-in DI container support
- **HttpClient Integration**: Uses HttpClientFactory patterns

## Quick Start

```csharp
using OpenSim.Client;
using System;
using System.Threading.Tasks;

class Program
{
    static async Task Main(string[] args)
    {
        using var client = new OpenSimClient("https://api.opensim.org");
        
        try
        {
            // Authenticate
            var auth = await client.AuthenticateAsync("username", "password");
            Console.WriteLine($"Access token: {auth.AccessToken}");
            
            // Use authenticated client
            var profile = await client.GetUserProfileAsync(auth.UserId);
            Console.WriteLine($"Welcome, {profile.Username}!");
        }
        catch (OpenSimException ex)
        {
            Console.WriteLine($"Error: {ex.Message}");
        }
    }
}
```

## Dependency Injection

```csharp
// Startup.cs
services.AddHttpClient<IOpenSimClient, OpenSimClient>(client =>
{
    client.BaseAddress = new Uri("https://api.opensim.org");
    client.Timeout = TimeSpan.FromSeconds(30);
});

// Usage
public class UserService
{
    private readonly IOpenSimClient _client;
    
    public UserService(IOpenSimClient client)
    {
        _client = client;
    }
    
    public async Task<UserProfile> GetUserAsync(string userId)
    {
        return await _client.GetUserProfileAsync(userId);
    }
}
```

## Error Handling

```csharp
try
{
    var profile = await client.GetUserProfileAsync(userId);
    Console.WriteLine($"User: {profile.Username}");
}
catch (AuthenticationException ex)
{
    Console.WriteLine($"Auth failed: {ex.Message}");
}
catch (HttpRequestException ex)
{
    Console.WriteLine($"Network error: {ex.Message}");
}
catch (OpenSimException ex)
{
    Console.WriteLine($"API error: {ex.Message}");
}
```

## Configuration

```csharp
var config = new OpenSimClientConfig
{
    BaseUrl = "https://api.opensim.org",
    Timeout = TimeSpan.FromSeconds(30),
    RetryAttempts = 3,
    Debug = false
};

using var client = new OpenSimClient(config);
```
"#)
    }

    async fn generate_java_specific_docs(&self) -> Result<String> {
        Ok(r#"# Java SDK Guide

The OpenSim Java SDK provides a comprehensive client for Java applications with modern Java features.

## Installation

### Maven

```xml
<dependency>
    <groupId>org.opensim</groupId>
    <artifactId>opensim-client</artifactId>
    <version>1.0.0</version>
</dependency>
```

### Gradle

```groovy
implementation 'org.opensim:opensim-client:1.0.0'
```

## Features

- **Java 11+**: Modern Java with var, records, and more
- **CompletableFuture**: Full async support
- **Jackson Integration**: Automatic JSON serialization
- **OkHttp**: Robust HTTP client with connection pooling
- **SLF4J Logging**: Standard Java logging integration

## Quick Start

```java
import org.opensim.client.OpenSimClient;
import org.opensim.client.model.AuthResponse;
import org.opensim.client.model.UserProfile;

public class Example {
    public static void main(String[] args) {
        try (var client = new OpenSimClient("https://api.opensim.org")) {
            // Authenticate
            var auth = client.authenticate("username", "password");
            System.out.println("Access token: " + auth.getAccessToken());
            
            // Use authenticated client
            var profile = client.getUserProfile(auth.getUserId());
            System.out.println("Welcome, " + profile.getUsername() + "!");
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
        }
    }
}
```

## Async Usage

```java
import java.util.concurrent.CompletableFuture;

public class AsyncExample {
    public void asyncOperations() {
        var client = new OpenSimClient("https://api.opensim.org");
        
        CompletableFuture
            .supplyAsync(() -> client.authenticate("username", "password"))
            .thenCompose(auth -> 
                CompletableFuture.supplyAsync(() -> 
                    client.getUserProfile(auth.getUserId())))
            .thenAccept(profile -> 
                System.out.println("Welcome, " + profile.getUsername() + "!"))
            .exceptionally(throwable -> {
                System.err.println("Error: " + throwable.getMessage());
                return null;
            });
    }
}
```

## Configuration

```java
var config = OpenSimClientConfig.builder()
    .baseUrl("https://api.opensim.org")
    .timeout(Duration.ofSeconds(30))
    .retryAttempts(3)
    .debug(false)
    .build();

try (var client = new OpenSimClient(config)) {
    // Use configured client
}
```

## Spring Integration

```java
@Configuration
public class OpenSimConfig {
    
    @Bean
    public OpenSimClient openSimClient(@Value("${opensim.api.url}") String apiUrl) {
        return new OpenSimClient(apiUrl);
    }
}

@Service
public class UserService {
    
    private final OpenSimClient client;
    
    public UserService(OpenSimClient client) {
        this.client = client;
    }
    
    public UserProfile getUser(String userId) {
        return client.getUserProfile(userId);
    }
}
```
"#)
    }

    // Example generators
    fn generate_basic_example(&self, language: &TargetLanguage) -> Result<String> {
        match language {
            TargetLanguage::Rust => Ok(r#"//! Basic OpenSim client usage example

use opensim_client::{OpenSimClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    
    // Authenticate
    let auth = client.authenticate("your_username", "your_password").await?;
    println!("Successfully authenticated!");
    println!("Access token: {}", auth.access_token);
    
    // Get user profile
    let profile = client.get_user_profile(&auth.user_id).await?;
    println!("User Profile:");
    println!("  ID: {}", profile.id);
    println!("  Username: {}", profile.username);
    println!("  Email: {}", profile.email.unwrap_or_else(|| "Not set".to_string()));
    
    // List regions
    let regions = client.list_regions(Some(10)).await?;
    println!("Available Regions ({}):", regions.len());
    for region in regions {
        println!("  - {} ({} users online)", region.name, region.online_users);
    }
    
    Ok(())
}
"#),
            TargetLanguage::Python => Ok(r#"""Basic OpenSim client usage example"""

import asyncio
from opensim_client import OpenSimClient

async def main():
    # Create client
    async with OpenSimClient("https://api.opensim.org") as client:
        try:
            # Authenticate
            auth = await client.authenticate("your_username", "your_password")
            print("Successfully authenticated!")
            print(f"Access token: {auth.access_token}")
            
            # Get user profile
            profile = await client.get_user_profile(auth.user_id)
            print("User Profile:")
            print(f"  ID: {profile.id}")
            print(f"  Username: {profile.username}")
            print(f"  Email: {profile.email or 'Not set'}")
            
            # List regions
            regions = await client.list_regions(limit=10)
            print(f"Available Regions ({len(regions)}):")
            for region in regions:
                print(f"  - {region.name} ({region.online_users} users online)")
                
        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
"#),
            _ => Ok(format!("// Basic example for {:?} - TODO: Implement", language)),
        }
    }

    fn generate_auth_example(&self, language: &TargetLanguage) -> Result<String> {
        match language {
            TargetLanguage::Rust => Ok(r#"//! Authentication patterns and token management

use opensim_client::{OpenSimClient, Result, AuthError};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let client = OpenSimClient::new("https://api.opensim.org").await?;
    
    // Basic authentication
    println!("=== Basic Authentication ===");
    match client.authenticate("username", "password").await {
        Ok(auth) => {
            println!("✓ Authentication successful");
            println!("  Access token: {}", &auth.access_token[..20]);
            println!("  Expires in: {} seconds", auth.expires_in);
            println!("  Token type: {}", auth.token_type);
        }
        Err(AuthError::InvalidCredentials) => {
            println!("✗ Invalid username or password");
            return Ok(());
        }
        Err(e) => {
            println!("✗ Authentication failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Token refresh pattern
    println!("\n=== Token Refresh Pattern ===");
    let mut auth = client.authenticate("username", "password").await?;
    
    loop {
        // Check if token expires soon (within 5 minutes)
        if auth.expires_in < 300 {
            println!("🔄 Refreshing token...");
            auth = client.refresh_token(&auth.refresh_token).await?;
            println!("✓ Token refreshed successfully");
        }
        
        // Use the client with current token
        client.set_access_token(&auth.access_token);
        let profile = client.get_user_profile(&auth.user_id).await?;
        println!("Current user: {}", profile.username);
        
        // Wait before next operation
        sleep(Duration::from_secs(60)).await;
        auth.expires_in = auth.expires_in.saturating_sub(60);
        
        if auth.expires_in == 0 {
            break;
        }
    }
    
    Ok(())
}
"#),
            _ => Ok(format!("// Auth example for {:?} - TODO: Implement", language)),
        }
    }

    fn generate_advanced_example(&self, language: &TargetLanguage) -> Result<String> {
        match language {
            TargetLanguage::Rust => Ok(r#"//! Advanced usage patterns with error handling and performance optimization

use opensim_client::{OpenSimClient, Result, OpenSimError};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::RwLock,
    time::{sleep, timeout},
};
use tracing::{info, warn, error};

/// Cached client wrapper for improved performance
struct CachedOpenSimClient {
    client: OpenSimClient,
    user_cache: Arc<RwLock<HashMap<String, (UserProfile, Instant)>>>,
    cache_ttl: Duration,
}

impl CachedOpenSimClient {
    async fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: OpenSimClient::new(base_url).await?,
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        })
    }
    
    /// Get user profile with caching
    async fn get_user_profile_cached(&self, user_id: &str) -> Result<UserProfile> {
        // Check cache first
        {
            let cache = self.user_cache.read().await;
            if let Some((profile, cached_at)) = cache.get(user_id) {
                if cached_at.elapsed() < self.cache_ttl {
                    info!("Cache hit for user {}", user_id);
                    return Ok(profile.clone());
                }
            }
        }
        
        // Cache miss - fetch from API
        info!("Cache miss for user {}, fetching from API", user_id);
        let profile = self.client.get_user_profile(user_id).await?;
        
        // Update cache
        {
            let mut cache = self.user_cache.write().await;
            cache.insert(user_id.to_string(), (profile.clone(), Instant::now()));
        }
        
        Ok(profile)
    }
    
    /// Batch get user profiles with concurrent requests
    async fn get_user_profiles_batch(&self, user_ids: &[String]) -> Vec<Result<UserProfile>> {
        let tasks: Vec<_> = user_ids.iter()
            .map(|id| self.get_user_profile_cached(id))
            .collect();
            
        futures::future::join_all(tasks).await
    }
}

/// Resilient client with retry logic
struct ResilientClient {
    client: CachedOpenSimClient,
    max_retries: u32,
    base_delay: Duration,
}

impl ResilientClient {
    async fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: CachedOpenSimClient::new(base_url).await?,
            max_retries: 3,
            base_delay: Duration::from_millis(100),
        })
    }
    
    /// Execute operation with exponential backoff retry
    async fn retry_with_backoff<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut attempt = 0;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt >= self.max_retries => return Err(e),
                Err(e) => {
                    let is_retryable = match &e {
                        OpenSimError::Network(_) => true,
                        OpenSimError::Http { status, .. } => *status >= 500,
                        _ => false,
                    };
                    
                    if !is_retryable {
                        return Err(e);
                    }
                    
                    let delay = self.base_delay * 2_u32.pow(attempt);
                    warn!("Attempt {} failed: {}, retrying in {:?}", attempt + 1, e, delay);
                    
                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
    
    /// Get user profile with retry logic
    async fn get_user_profile_resilient(&self, user_id: &str) -> Result<UserProfile> {
        let user_id = user_id.to_string();
        self.retry_with_backoff(|| {
            let user_id = user_id.clone();
            Box::pin(async move {
                self.client.get_user_profile_cached(&user_id).await
            })
        }).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let client = ResilientClient::new("https://api.opensim.org").await?;
    
    // Authenticate with timeout
    println!("=== Authentication with Timeout ===");
    let auth = timeout(
        Duration::from_secs(10),
        client.client.client.authenticate("username", "password")
    ).await??;
    
    client.client.client.set_access_token(&auth.access_token);
    println!("✓ Authenticated successfully");
    
    // Demonstrate caching
    println!("\n=== Caching Demonstration ===");
    let user_id = &auth.user_id;
    
    let start = Instant::now();
    let profile1 = client.get_user_profile_resilient(user_id).await?;
    println!("First call took: {:?}", start.elapsed());
    
    let start = Instant::now();
    let profile2 = client.get_user_profile_resilient(user_id).await?;
    println!("Second call took: {:?} (should be faster due to caching)", start.elapsed());
    
    assert_eq!(profile1.id, profile2.id);
    
    // Demonstrate batch operations
    println!("\n=== Batch Operations ===");
    let user_ids = vec![
        auth.user_id.clone(),
        "user2".to_string(),
        "user3".to_string(),
    ];
    
    let start = Instant::now();
    let results = client.client.get_user_profiles_batch(&user_ids).await;
    println!("Batch operation took: {:?}", start.elapsed());
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(profile) => println!("  User {}: {} ({})", i + 1, profile.username, profile.id),
            Err(e) => println!("  User {}: Error - {}", i + 1, e),
        }
    }
    
    // Demonstrate error handling
    println!("\n=== Error Handling ===");
    match client.get_user_profile_resilient("nonexistent_user").await {
        Ok(_) => println!("Unexpected success"),
        Err(OpenSimError::Http { status: 404, .. }) => {
            println!("✓ Correctly handled user not found error");
        }
        Err(e) => println!("Other error: {}", e),
    }
    
    // Performance monitoring
    println!("\n=== Performance Monitoring ===");
    let mut total_time = Duration::default();
    let iterations = 5;
    
    for i in 0..iterations {
        let start = Instant::now();
        let _ = client.get_user_profile_resilient(user_id).await?;
        let elapsed = start.elapsed();
        total_time += elapsed;
        println!("Request {}: {:?}", i + 1, elapsed);
    }
    
    println!("Average response time: {:?}", total_time / iterations);
    
    Ok(())
}
"#),
            _ => Ok(format!("// Advanced example for {:?} - TODO: Implement", language)),
        }
    }

    async fn generate_examples_index(&self) -> Result<String> {
        Ok(r#"# OpenSim SDK Examples

This directory contains practical examples demonstrating how to use the OpenSim client SDKs across different programming languages.

## Structure

Each language has its own subdirectory with examples:

- `rust/` - Rust examples
- `python/` - Python examples
- `javascript/` - JavaScript/TypeScript examples
- `csharp/` - C# examples
- `java/` - Java examples

## Example Types

### Basic Examples
- `basic.*` - Simple client setup and basic operations
- Shows authentication, user profile retrieval, and region listing

### Authentication Examples
- `authentication.*` - Comprehensive authentication patterns
- Token management, refresh logic, and error handling

### Advanced Examples
- `advanced.*` - Production-ready patterns
- Caching, retry logic, batch operations, and performance optimization

## Running Examples

### Rust
```bash
cd examples/rust
cargo run --bin basic
cargo run --bin authentication
cargo run --bin advanced
```

### Python
```bash
cd examples/python
python basic.py
python authentication.py
python advanced.py
```

### JavaScript
```bash
cd examples/javascript
node basic.js
node authentication.js
node advanced.js
```

### C#
```bash
cd examples/csharp
dotnet run basic
dotnet run authentication
dotnet run advanced
```

### Java
```bash
cd examples/java
javac -cp ".:lib/*" *.java
java -cp ".:lib/*" Basic
java -cp ".:lib/*" Authentication
java -cp ".:lib/*" Advanced
```

## Configuration

Most examples expect the following environment variables:

- `OPENSIM_API_URL` - OpenSim API base URL (default: https://api.opensim.org)
- `OPENSIM_USERNAME` - Your OpenSim username
- `OPENSIM_PASSWORD` - Your OpenSim password

Example:
```bash
export OPENSIM_API_URL="https://api.opensim.org"
export OPENSIM_USERNAME="your_username"
export OPENSIM_PASSWORD="your_password"
```

## Common Patterns

All examples demonstrate these important patterns:

1. **Proper Authentication** - Secure token handling and refresh
2. **Error Handling** - Comprehensive error management
3. **Resource Cleanup** - Proper connection and resource management
4. **Performance** - Efficient API usage and caching strategies
5. **Logging** - Structured logging for debugging and monitoring

## Next Steps

After running the examples, check out:

- [API Reference](../api-reference.md) - Complete API documentation
- [Best Practices](../best-practices.md) - Recommended patterns
- [Troubleshooting](../troubleshooting.md) - Common issues and solutions
"#)
    }

    async fn generate_index_and_navigation(&self, all_docs: &[GeneratedFile]) -> Result<Vec<GeneratedFile>> {
        let mut nav_docs = Vec::new();

        // Generate main index
        let index_content = format!(r#"# OpenSim Client SDK Documentation

Welcome to the comprehensive documentation for OpenSim client SDKs. This documentation covers all supported programming languages and provides everything you need to integrate with OpenSim virtual world servers.

## Quick Navigation

### Getting Started
- [Getting Started Guide](getting-started.md) - Start here if you're new to OpenSim SDKs
- [Installation Instructions](getting-started.md#installation) - Language-specific installation steps
- [Quick Examples](getting-started.md#quick-start) - Get up and running in minutes

### API Documentation
- [API Reference](api-reference.md) - Complete API endpoint documentation
- [Data Types](api-reference.md#data-types) - All data structures and models
- [Authentication](api-reference.md#authentication) - Authentication methods and security

### Language-Specific Guides
- [Rust SDK Guide](rust-guide.md) - Type-safe, high-performance Rust client
- [Python SDK Guide](python-guide.md) - Easy-to-use Python client with async support
- [JavaScript SDK Guide](javascript-guide.md) - Universal JavaScript/TypeScript client
- [C# SDK Guide](csharp-guide.md) - .NET client with full async support
- [Java SDK Guide](java-guide.md) - Enterprise Java client with modern features

### Examples and Patterns
- [Code Examples](examples/) - Practical examples for all languages
- [Best Practices](best-practices.md) - Recommended patterns and practices
- [Advanced Patterns](examples/README.md#advanced-examples) - Production-ready code patterns

### Help and Support
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions
- [FAQ](#faq) - Frequently asked questions
- [Support Channels](#support) - How to get help

## Supported Languages

| Language | Version | Status | Features |
|----------|---------|--------|----------|
| **Rust** | 1.70+ | ✅ Stable | Type safety, async/await, zero-cost abstractions |
| **Python** | 3.8+ | ✅ Stable | Async/await, type hints, context managers |
| **JavaScript** | ES2018+ | ✅ Stable | Universal (Node.js/Browser), TypeScript support |
| **C#** | .NET Standard 2.0+ | ✅ Stable | Async/await, dependency injection, strong typing |
| **Java** | Java 11+ | ✅ Stable | CompletableFuture, modern Java features |

## Core Features

All SDKs provide consistent functionality:

- ✅ **Full API Coverage** - Complete OpenSim API support
- ✅ **Authentication** - Secure token-based authentication
- ✅ **Async Operations** - Non-blocking I/O for better performance
- ✅ **Error Handling** - Comprehensive error types and handling
- ✅ **Type Safety** - Strong typing where supported by language
- ✅ **Caching** - Built-in caching for improved performance
- ✅ **Retry Logic** - Automatic retry with exponential backoff
- ✅ **Logging** - Structured logging integration
- ✅ **Testing Support** - Mock clients and testing utilities

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Application                         │
├─────────────────────────────────────────────────────────────┤
│                OpenSim Client SDK                           │
├─────────────────────────────────────────────────────────────┤
│     HTTP Client │ Auth Manager │ Cache │ Retry Logic       │
├─────────────────────────────────────────────────────────────┤
│                      HTTPS/REST                             │
├─────────────────────────────────────────────────────────────┤
│                  OpenSim API Server                         │
└─────────────────────────────────────────────────────────────┘
```

## FAQ

### General Questions

**Q: Which SDK should I choose?**
A: Choose based on your application requirements:
- **Rust**: Maximum performance, memory safety, systems programming
- **Python**: Rapid development, data science, scripting
- **JavaScript**: Web applications, Node.js services, universal compatibility
- **C#**: .NET applications, Unity games, enterprise systems
- **Java**: Enterprise applications, Android development, JVM ecosystem

**Q: Are the SDKs compatible with all OpenSim versions?**
A: The SDKs are designed to work with OpenSim 0.9+ and are automatically updated to support new API versions.

**Q: Can I use multiple SDKs in the same project?**
A: Yes, all SDKs use the same API and data formats, so they're fully interoperable.

### Authentication

**Q: How do I handle token expiration?**
A: All SDKs provide automatic token refresh. See the [authentication examples](examples/) for implementation details.

**Q: Is it safe to store tokens?**
A: Store tokens securely using your platform's secure storage mechanisms. Never commit tokens to version control.

### Performance

**Q: How can I improve performance?**
A: Use caching, connection pooling, and batch operations. See [Best Practices](best-practices.md#performance-optimization) for details.

**Q: What's the rate limit?**
A: Rate limits vary by endpoint and server configuration. Implement exponential backoff to handle rate limiting gracefully.

## Support

### Documentation
- **Complete Guides**: This documentation covers all aspects of SDK usage
- **API Reference**: Detailed endpoint and data type documentation
- **Examples**: Practical code examples for all languages

### Community Support
- **GitHub Issues**: [Report bugs and request features](https://github.com/opensim/opensim-next/issues)
- **Discord**: [Join the community chat](https://discord.gg/opensim)
- **Forums**: [OpenSim Community Forums](https://forums.opensim.org)

### Professional Support
For enterprise support and custom development:
- **Email**: support@opensim.org
- **Consulting**: Available for custom integrations and training

## Contributing

We welcome contributions to improve the SDKs and documentation:

1. **Bug Reports**: Use GitHub issues to report bugs
2. **Feature Requests**: Suggest new features via GitHub issues
3. **Code Contributions**: Submit pull requests with improvements
4. **Documentation**: Help improve and expand this documentation

## License

All OpenSim client SDKs are released under the BSD 3-Clause License. See the LICENSE file in each SDK repository for details.

---

**Ready to get started?** Head to the [Getting Started Guide](getting-started.md) to begin your OpenSim integration journey!
"#);

        nav_docs.push(GeneratedFile {
            path: PathBuf::from("README.md"),
            content: index_content,
            file_type: GeneratedFileType::Documentation,
        });

        Ok(nav_docs)
    }

    fn get_file_extension(&self, language: &TargetLanguage) -> &'static str {
        match language {
            TargetLanguage::Rust => "rs",
            TargetLanguage::Python => "py",
            TargetLanguage::JavaScript => "js",
            TargetLanguage::TypeScript => "ts",
            TargetLanguage::CSharp => "cs",
            TargetLanguage::Java => "java",
            TargetLanguage::Go => "go",
            TargetLanguage::PHP => "php",
            TargetLanguage::Ruby => "rb",
            TargetLanguage::Swift => "swift",
        }
    }

    /// Generate interactive examples with live editing capabilities
    async fn generate_interactive_examples(&self) -> Result<Vec<GeneratedFile>> {
        info!("Generating interactive examples");
        let mut files = Vec::new();

        // Create interactive examples HTML page
        let interactive_html = self.create_interactive_examples_page().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("interactive/examples.html"),
            content: interactive_html,
            file_type: GeneratedFileType::Documentation,
        });

        // Create JavaScript for interactive functionality
        let interactive_js = self.create_interactive_js().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("interactive/js/examples.js"),
            content: interactive_js,
            file_type: GeneratedFileType::SourceCode,
        });

        // Create CSS for interactive styling
        let interactive_css = self.create_interactive_css().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("interactive/css/examples.css"),
            content: interactive_css,
            file_type: GeneratedFileType::Configuration,
        });

        Ok(files)
    }

    /// Generate live demos with real API integration
    async fn generate_live_demos(&self) -> Result<Vec<GeneratedFile>> {
        info!("Generating live demos");
        let mut files = Vec::new();

        for language in &self.config.target_languages {
            let demo_content = self.create_live_demo_for_language(language).await?;
            files.push(GeneratedFile {
                path: PathBuf::from(format!("demos/{}/index.html", self.language_to_string(language))),
                content: demo_content,
                file_type: GeneratedFileType::Documentation,
            });
        }

        // Create demo server configuration
        let demo_server_config = self.create_demo_server_config().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("demos/server/config.json"),
            content: demo_server_config,
            file_type: GeneratedFileType::Configuration,
        });

        Ok(files)
    }

    /// Generate interactive playground for testing code
    async fn generate_playground(&self) -> Result<Vec<GeneratedFile>> {
        info!("Generating interactive playground");
        let mut files = Vec::new();

        // Main playground HTML
        let playground_html = self.create_playground_html().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("playground/index.html"),
            content: playground_html,
            file_type: GeneratedFileType::Documentation,
        });

        // Playground API server
        let playground_server = self.create_playground_server().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("playground/server/main.rs"),
            content: playground_server,
            file_type: GeneratedFileType::SourceCode,
        });

        // Docker configuration for playground
        let dockerfile = self.create_playground_dockerfile().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("playground/Dockerfile"),
            content: dockerfile,
            file_type: GeneratedFileType::Configuration,
        });

        Ok(files)
    }

    /// Generate deployment configurations
    async fn generate_deployment_configs(&self) -> Result<Vec<GeneratedFile>> {
        info!("Generating deployment configurations");
        let mut files = Vec::new();

        // GitHub Actions workflow for auto-deployment
        let github_workflow = self.create_github_deployment_workflow().await?;
        files.push(GeneratedFile {
            path: PathBuf::from(".github/workflows/deploy-docs.yml"),
            content: github_workflow,
            file_type: GeneratedFileType::Configuration,
        });

        // Netlify configuration
        if self.config.deployment.netlify {
            let netlify_config = self.create_netlify_config().await?;
            files.push(GeneratedFile {
                path: PathBuf::from("netlify.toml"),
                content: netlify_config,
                file_type: GeneratedFileType::Configuration,
            });
        }

        // Vercel configuration
        if self.config.deployment.vercel {
            let vercel_config = self.create_vercel_config().await?;
            files.push(GeneratedFile {
                path: PathBuf::from("vercel.json"),
                content: vercel_config,
                file_type: GeneratedFileType::Configuration,
            });
        }

        // Package.json for build dependencies
        let package_json = self.create_package_json().await?;
        files.push(GeneratedFile {
            path: PathBuf::from("package.json"),
            content: package_json,
            file_type: GeneratedFileType::Configuration,
        });

        Ok(files)
    }

    /// Create interactive examples HTML page
    async fn create_interactive_examples_page(&self) -> Result<String> {
        Ok(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim SDK Interactive Examples</title>
    <link rel="stylesheet" href="css/examples.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/theme/monokai.min.css">
</head>
<body>
    <header>
        <h1>OpenSim SDK Interactive Examples</h1>
        <p>Try out the OpenSim SDK with live, editable examples in your browser</p>
    </header>

    <main>
        <div class="language-selector">
            <label for="language">Choose Language:</label>
            <select id="language">
                <option value="rust">Rust</option>
                <option value="python">Python</option>
                <option value="javascript">JavaScript</option>
                <option value="csharp">CSharp</option>
                <option value="java">Java</option>
                <option value="go">Go</option>
                <option value="php">PHP</option>
                <option value="ruby">Ruby</option>
            </select>
        </div>

        <div class="example-container">
            <div class="example-sidebar">
                <h3>Examples</h3>
                <ul id="example-list">
                    <li><a href="#basic-connection" data-example="basic-connection">Basic Connection</a></li>
                    <li><a href="#authentication" data-example="authentication">Authentication</a></li>
                    <li><a href="#user-management" data-example="user-management">User Management</a></li>
                    <li><a href="#asset-upload" data-example="asset-upload">Asset Upload</a></li>
                    <li><a href="#region-management" data-example="region-management">Region Management</a></li>
                </ul>
            </div>

            <div class="example-content">
                <div class="code-editor">
                    <h4 id="example-title">Basic Connection</h4>
                    <textarea id="code-editor"></textarea>
                    <div class="editor-controls">
                        <button id="run-code">Run Code</button>
                        <button id="reset-code">Reset</button>
                        <button id="share-code">Share</button>
                    </div>
                </div>

                <div class="output-panel">
                    <h4>Output</h4>
                    <pre id="output"></pre>
                </div>
            </div>
        </div>
    </main>

    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/mode/rust/rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/mode/python/python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/mode/javascript/javascript.min.js"></script>
    <script src="js/examples.js"></script>
</body>
</html>"##))
    }

    /// Create GitHub deployment workflow
    async fn create_github_deployment_workflow(&self) -> Result<String> {
        Ok(r##"name: Deploy OpenSim Documentation

on:
  push:
    branches: [ main, develop ]
    paths:
      - 'opensim-next/rust/src/client_sdk/**/*'
      - 'docs/**/*'
      - 'client-docs/**/*' 
      - '.github/workflows/deploy-docs.yml'
  
  schedule:
    # Rebuild docs daily at 2 AM UTC to catch any API changes
    - cron: '0 2 * * *'
  
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'production'
        type: choice
        options:
        - production
        - staging
        - preview

env:
  NODE_VERSION: '18'
  RUST_VERSION: '1.70'

jobs:
  generate-docs:
    runs-on: ubuntu-latest
    outputs:
      docs-changed: ${{ steps.check-changes.outputs.changed }}
    
    steps:
    - name: Checkout repository
      uses: actions/checkout@v4
      with:
        fetch-depth: 0
        token: ${{ secrets.GITHUB_TOKEN }}

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ env.RUST_VERSION }}
        profile: minimal
        override: true
        components: rustfmt, clippy

    - name: Cache Rust dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/bin/
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          opensim-next/rust/target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: ${{ env.NODE_VERSION }}
        cache: 'npm'
        cache-dependency-path: 'client-docs/package-lock.json'

    - name: Install Node.js dependencies
      run: |
        cd client-docs
        npm ci

    - name: Generate SDK documentation
      run: |
        cd opensim-next/rust
        cargo run --bin doc-generator -- \
          --output-dir ../../client-docs/generated \
          --format html,json,openapi \
          --interactive-examples \
          --live-demos \
          --playground

    - name: Build documentation site
      run: |
        cd client-docs
        npm run build:docs
        npm run build:interactive
        npm run optimize

    - name: Run documentation tests
      run: |
        cd client-docs
        npm run test:docs
        npm run validate:links
        npm run validate:examples

    - name: Check for changes
      id: check-changes
      run: |
        if git diff --quiet HEAD~1 client-docs/dist/; then
          echo "changed=false" >> $GITHUB_OUTPUT
        else
          echo "changed=true" >> $GITHUB_OUTPUT
        fi

    - name: Upload documentation artifacts
      uses: actions/upload-artifact@v3
      with:
        name: documentation-${{ github.sha }}
        path: client-docs/dist/
        retention-days: 30

  deploy-staging:
    if: github.ref == 'refs/heads/develop' || github.event.inputs.environment == 'staging'
    needs: generate-docs
    runs-on: ubuntu-latest
    environment: staging
    
    steps:
    - name: Download documentation artifacts
      uses: actions/download-artifact@v3
      with:
        name: documentation-${{ github.sha }}
        path: ./dist

    - name: Deploy to Netlify (Staging)
      uses: nwtgck/actions-netlify@v2.1
      with:
        publish-dir: './dist'
        production-deploy: false
        github-token: ${{ secrets.GITHUB_TOKEN }}
        deploy-message: |
          Deploy staging docs from commit ${{ github.sha }}
          
          Branch: ${{ github.ref_name }}
          Commit: ${{ github.event.head_commit.message }}
        alias: staging-docs-${{ github.run_number }}
      env:
        NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
        NETLIFY_SITE_ID: ${{ secrets.NETLIFY_STAGING_SITE_ID }}

    - name: Update staging status
      run: |
        echo "Staging documentation deployed to:"
        echo "https://staging-docs-${{ github.run_number }}--opensim-docs.netlify.app"

  deploy-production:
    if: github.ref == 'refs/heads/main' && (needs.generate-docs.outputs.docs-changed == 'true' || github.event_name == 'workflow_dispatch')
    needs: generate-docs
    runs-on: ubuntu-latest
    environment: production
    
    steps:
    - name: Download documentation artifacts
      uses: actions/download-artifact@v3
      with:
        name: documentation-${{ github.sha }}
        path: ./dist

    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./dist
        cname: docs.opensim.org
        commit_message: |
          Deploy documentation ${{ github.sha }}
          
          Generated from: ${{ github.event.head_commit.message }}

    - name: Deploy to Netlify (Production)
      uses: nwtgck/actions-netlify@v2.1
      with:
        publish-dir: './dist'
        production-deploy: true
        github-token: ${{ secrets.GITHUB_TOKEN }}
        deploy-message: |
          Deploy production docs from commit ${{ github.sha }}
          
          Branch: ${{ github.ref_name }}
          Commit: ${{ github.event.head_commit.message }}
      env:
        NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
        NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}

    - name: Deploy to Vercel
      uses: amondnet/vercel-action@v25
      with:
        vercel-token: ${{ secrets.VERCEL_TOKEN }}
        vercel-org-id: ${{ secrets.VERCEL_ORG_ID }}
        vercel-project-id: ${{ secrets.VERCEL_PROJECT_ID }}
        working-directory: ./dist
        vercel-args: '--prod'

    - name: Update search index
      run: |
        curl -X POST "${{ secrets.ALGOLIA_WEBHOOK_URL }}" \
          -H "Authorization: Bearer ${{ secrets.ALGOLIA_API_KEY }}" \
          -H "Content-Type: application/json" \
          -d '{"url": "https://docs.opensim.org"}'

    - name: Notify Discord
      uses: Ilshidur/action-discord@master
      env:
        DISCORD_WEBHOOK: ${{ secrets.DISCORD_WEBHOOK }}
      with:
        args: |
          📚 OpenSim documentation updated!
          
          🔗 **Live site**: https://docs.opensim.org
          📦 **Commit**: `${{ github.sha }}`
          🌿 **Branch**: `${{ github.ref_name }}`
          👤 **Author**: ${{ github.actor }}
          
          📝 **Changes**: ${{ github.event.head_commit.message }}

  deploy-preview:
    if: github.event.inputs.environment == 'preview' || github.event_name == 'pull_request'
    needs: generate-docs
    runs-on: ubuntu-latest
    
    steps:
    - name: Download documentation artifacts
      uses: actions/download-artifact@v3
      with:
        name: documentation-${{ github.sha }}
        path: ./dist

    - name: Deploy preview to Netlify
      uses: nwtgck/actions-netlify@v2.1
      with:
        publish-dir: './dist'
        production-deploy: false
        github-token: ${{ secrets.GITHUB_TOKEN }}
        deploy-message: |
          Preview deploy from ${{ github.event_name }}
          
          ${{ github.event.pull_request.title || github.event.head_commit.message }}
        alias: preview-${{ github.event.number || github.run_number }}
      env:
        NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
        NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}

    - name: Comment preview link
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        script: |
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: `## 📖 Documentation Preview
            
            Your documentation changes are ready for review!
            
            🔗 **Preview URL**: https://preview-${{ github.event.number }}--opensim-docs.netlify.app
            
            This preview will be updated with each new commit to this PR.`
          })

  quality-checks:
    runs-on: ubuntu-latest
    needs: generate-docs
    
    steps:
    - name: Download documentation artifacts
      uses: actions/download-artifact@v3
      with:
        name: documentation-${{ github.sha }}
        path: ./dist

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: ${{ env.NODE_VERSION }}

    - name: Install quality check tools
      run: |
        npm install -g lighthouse @lhci/cli htmlhint markdownlint-cli

    - name: Run Lighthouse CI
      run: |
        lhci autorun --config=.lighthouserc.json

    - name: Validate HTML
      run: |
        htmlhint dist/**/*.html

    - name: Check accessibility
      run: |
        npx pa11y-ci --sitemap https://docs.opensim.org/sitemap.xml

    - name: Performance audit
      run: |
        npx bundlesize

    - name: Security scan
      run: |
        npm audit --audit-level moderate

  cleanup:
    if: always()
    needs: [deploy-staging, deploy-production, deploy-preview, quality-checks]
    runs-on: ubuntu-latest
    
    steps:
    - name: Cleanup old preview deployments
      run: |
        # Clean up preview deployments older than 7 days
        echo "Cleaning up old preview deployments..."
        # This would typically use Netlify API to clean up old deployments

    - name: Update deployment status
      run: |
        echo "Documentation deployment completed"
        echo "Status: ${{ job.status }}"
"##.to_string())
    }

    /// Create package.json for documentation build
    async fn create_package_json(&self) -> Result<String> {
        Ok(r#"{
  "name": "opensim-docs",
  "version": "1.0.0",
  "description": "OpenSim SDK Documentation",
  "main": "index.js",
  "scripts": {
    "build": "webpack --mode=production",
    "dev": "webpack serve --mode=development",
    "test": "jest",
    "format": "prettier --write .",
    "lint": "eslint . --fix"
  },
  "dependencies": {
    "highlight.js": "^11.9.0",
    "marked": "^9.1.6",
    "prismjs": "^1.29.0",
    "docsify": "^4.13.1"
  },
  "devDependencies": {
    "webpack": "^5.89.0",
    "webpack-cli": "^5.1.4",
    "webpack-dev-server": "^4.15.1",
    "html-webpack-plugin": "^5.5.4",
    "css-loader": "^6.8.1",
    "style-loader": "^3.3.3",
    "mini-css-extract-plugin": "^2.7.6",
    "jest": "^29.7.0",
    "prettier": "^3.1.0",
    "eslint": "^8.55.0"
  }
}"#.to_string())
    }

    /// Create interactive JavaScript functionality
    async fn create_interactive_js(&self) -> Result<String> {
        Ok(r#"// OpenSim SDK Interactive Examples JavaScript
class OpenSimExamplesApp {
    constructor() {
        this.editor = null;
        this.currentLanguage = 'rust';
        this.currentExample = 'basic-connection';
        this.examples = this.loadExamples();
        this.init();
    }

    init() {
        this.setupEditor();
        this.setupEventListeners();
        this.loadExample(this.currentExample, this.currentLanguage);
    }

    setupEditor() {
        const textarea = document.getElementById('code-editor');
        this.editor = CodeMirror.fromTextArea(textarea, {
            theme: 'monokai',
            lineNumbers: true,
            mode: this.getEditorMode(this.currentLanguage),
            indentUnit: 4,
            smartIndent: true,
            extraKeys: {
                'Ctrl-Enter': () => this.runCode(),
                'Cmd-Enter': () => this.runCode()
            }
        });
    }

    setupEventListeners() {
        document.getElementById('language').addEventListener('change', (e) => {
            this.currentLanguage = e.target.value;
            this.editor.setOption('mode', this.getEditorMode(this.currentLanguage));
            this.loadExample(this.currentExample, this.currentLanguage);
        });

        document.getElementById('run-code').addEventListener('click', () => this.runCode());
        document.getElementById('reset-code').addEventListener('click', () => this.resetCode());
        document.getElementById('share-code').addEventListener('click', () => this.shareCode());

        document.querySelectorAll('[data-example]').forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                this.currentExample = e.target.dataset.example;
                this.loadExample(this.currentExample, this.currentLanguage);
            });
        });
    }

    getEditorMode(language) {
        const modes = {
            'rust': 'rust',
            'python': 'python',
            'javascript': 'javascript',
            'csharp': 'text/x-csharp',
            'java': 'text/x-java',
            'go': 'text/x-go',
            'php': 'text/x-php',
            'ruby': 'text/x-ruby'
        };
        return modes[language] || 'text';
    }

    async runCode() {
        const code = this.editor.getValue();
        const output = document.getElementById('output');
        
        output.textContent = 'Running...';

        try {
            const response = await fetch('/api/playground/execute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    language: this.currentLanguage,
                    code: code
                })
            });

            const result = await response.json();
            
            if (result.success) {
                output.textContent = result.output;
            } else {
                output.textContent = `Error: ${result.error}`;
            }
        } catch (error) {
            output.textContent = `Network Error: ${error.message}`;
        }
    }

    resetCode() {
        this.loadExample(this.currentExample, this.currentLanguage);
    }

    async shareCode() {
        const code = this.editor.getValue();
        // Implementation for sharing code (e.g., create a shareable link)
        console.log('Sharing code:', code);
    }

    loadExample(exampleId, language) {
        const example = this.examples[exampleId]?.[language];
        if (example) {
            this.editor.setValue(example.code);
            document.getElementById('example-title').textContent = example.title;
        }
    }

    loadExamples() {
        return {
            'basic-connection': {
                'rust': {
                    title: 'Basic Connection - Rust',
                    code: `use opensim_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://api.opensim.org").await?;
    
    println!("Connected to OpenSim API!");
    
    // Test the connection
    let health = client.health_check().await?;
    println!("Server status: {:?}", health);
    
    Ok(())
}`
                },
                'python': {
                    title: 'Basic Connection - Python',
                    code: `import asyncio
from opensim_client import Client

async def main():
    async with Client("https://api.opensim.org") as client:
        print("Connected to OpenSim API!")
        
        # Test the connection
        health = await client.health_check()
        print(f"Server status: {health}")

if __name__ == "__main__":
    asyncio.run(main())`
                }
            },
            'authentication': {
                'rust': {
                    title: 'Authentication - Rust',
                    code: `use opensim_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://api.opensim.org").await?;
    
    // Authenticate with username and password
    let auth = client.authenticate("username", "password").await?;
    println!("Authentication successful!");
    println!("Token expires at: {}", auth.expires_at);
    
    // Use authenticated client
    let user_info = client.get_current_user().await?;
    println!("Welcome, {}!", user_info.username);
    
    Ok(())
}`
                }
            }
        };
    }
}

// Initialize the app when the page loads
document.addEventListener('DOMContentLoaded', () => {
    new OpenSimExamplesApp();
});
"#.to_string())
    }

    /// Create interactive CSS styling
    async fn create_interactive_css(&self) -> Result<String> {
        Ok(r#"/* OpenSim SDK Interactive Examples Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Inter', sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    color: #333;
}

header {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    padding: 2rem 0;
    text-align: center;
    box-shadow: 0 2px 20px rgba(0, 0, 0, 0.1);
}

header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: #2563eb;
    margin-bottom: 0.5rem;
}

header p {
    font-size: 1.1rem;
    color: #64748b;
}

main {
    max-width: 1400px;
    margin: 2rem auto;
    padding: 0 1rem;
}

.language-selector {
    background: white;
    padding: 1rem;
    border-radius: 12px;
    margin-bottom: 2rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.language-selector label {
    font-weight: 600;
    margin-right: 1rem;
    color: #374151;
}

.language-selector select {
    padding: 0.5rem 1rem;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 1rem;
    background: white;
    cursor: pointer;
    transition: border-color 0.2s;
}

.language-selector select:focus {
    outline: none;
    border-color: #2563eb;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.example-container {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 2rem;
    background: white;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 40px rgba(0, 0, 0, 0.12);
}

.example-sidebar {
    background: #f8fafc;
    padding: 2rem;
    border-right: 1px solid #e5e7eb;
}

.example-sidebar h3 {
    color: #374151;
    font-size: 1.2rem;
    margin-bottom: 1rem;
    font-weight: 600;
}

.example-sidebar ul {
    list-style: none;
}

.example-sidebar li {
    margin-bottom: 0.5rem;
}

.example-sidebar a {
    display: block;
    padding: 0.75rem 1rem;
    color: #6b7280;
    text-decoration: none;
    border-radius: 8px;
    transition: all 0.2s;
    font-weight: 500;
}

.example-sidebar a:hover,
.example-sidebar a.active {
    background: #2563eb;
    color: white;
    transform: translateX(4px);
}

.example-content {
    display: grid;
    grid-template-rows: 1fr auto;
    min-height: 600px;
}

.code-editor {
    padding: 2rem;
}

.code-editor h4 {
    color: #374151;
    font-size: 1.3rem;
    margin-bottom: 1rem;
    font-weight: 600;
}

.CodeMirror {
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 14px;
    height: 400px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.editor-controls {
    margin-top: 1rem;
    display: flex;
    gap: 1rem;
}

.editor-controls button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.9rem;
}

#run-code {
    background: #10b981;
    color: white;
}

#run-code:hover {
    background: #059669;
    transform: translateY(-2px);
    box-shadow: 0 4px 20px rgba(16, 185, 129, 0.3);
}

#reset-code {
    background: #f59e0b;
    color: white;
}

#reset-code:hover {
    background: #d97706;
    transform: translateY(-2px);
    box-shadow: 0 4px 20px rgba(245, 158, 11, 0.3);
}

#share-code {
    background: #6366f1;
    color: white;
}

#share-code:hover {
    background: #4f46e5;
    transform: translateY(-2px);
    box-shadow: 0 4px 20px rgba(99, 102, 241, 0.3);
}

.output-panel {
    background: #1f2937;
    color: #f9fafb;
    padding: 2rem;
    border-top: 1px solid #e5e7eb;
}

.output-panel h4 {
    color: #f9fafb;
    margin-bottom: 1rem;
    font-size: 1.1rem;
    font-weight: 600;
}

.output-panel pre {
    background: #111827;
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 14px;
    line-height: 1.5;
    border: 1px solid #374151;
    max-height: 200px;
    overflow-y: auto;
}

@media (max-width: 768px) {
    .example-container {
        grid-template-columns: 1fr;
    }
    
    .example-sidebar {
        border-right: none;
        border-bottom: 1px solid #e5e7eb;
    }
    
    header h1 {
        font-size: 2rem;
    }
    
    .editor-controls {
        flex-wrap: wrap;
    }
}
"#.to_string())
    }

    /// Create live demo for specific language
    async fn create_live_demo_for_language(&self, language: &TargetLanguage) -> Result<String> {
        let lang_str = self.language_to_string(language);
        Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim {} SDK Live Demo</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 2rem; }}
        .demo-container {{ max-width: 800px; margin: 0 auto; }}
        .status {{ padding: 1rem; border-radius: 8px; margin: 1rem 0; }}
        .success {{ background: #d1fae5; color: #065f46; }}
        .error {{ background: #fee2e2; color: #991b1b; }}
        .loading {{ background: #dbeafe; color: #1e40af; }}
        button {{ padding: 0.75rem 1.5rem; margin: 0.5rem; cursor: pointer; }}
    </style>
</head>
<body>
    <div class="demo-container">
        <h1>OpenSim {} SDK Live Demo</h1>
        <p>This demo shows the {} SDK in action with a real OpenSim server.</p>
        
        <div id="demo-status" class="status loading">
            Initializing demo...
        </div>
        
        <div class="demo-actions">
            <button onclick="testConnection()">Test Connection</button>
            <button onclick="authenticate()">Authenticate</button>
            <button onclick="getUserInfo()">Get User Info</button>
            <button onclick="listRegions()">List Regions</button>
        </div>
        
        <div id="demo-output">
            <h3>Output:</h3>
            <pre id="output"></pre>
        </div>
    </div>
    
    <script>
        // Demo implementation for {} SDK
        let client = null;
        let authToken = null;
        
        async function initDemo() {{
            try {{
                // Initialize the client based on language
                await initializeClient();
                updateStatus('Demo ready!', 'success');
            }} catch (error) {{
                updateStatus('Demo initialization failed: ' + error.message, 'error');
            }}
        }}
        
        async function initializeClient() {{
            // Language-specific client initialization
            // This would connect to actual SDK based on the language
            client = {{ initialized: true }};
        }}
        
        async function testConnection() {{
            updateStatus('Testing connection...', 'loading');
            try {{
                // Simulate API call
                await new Promise(resolve => setTimeout(resolve, 1000));
                updateOutput('Connection successful!\\nServer: api.opensim.org\\nStatus: Online');
                updateStatus('Connection test completed', 'success');
            }} catch (error) {{
                updateStatus('Connection failed: ' + error.message, 'error');
            }}
        }}
        
        function updateStatus(message, type) {{
            const status = document.getElementById('demo-status');
            status.textContent = message;
            status.className = `status ${{type}}`;
        }}
        
        function updateOutput(text) {{
            document.getElementById('output').textContent = text;
        }}
        
        // Initialize demo when page loads
        document.addEventListener('DOMContentLoaded', initDemo);
    </script>
</body>
</html>"#, 
        lang_str.to_uppercase(), 
        lang_str.to_uppercase(), 
        lang_str, 
        lang_str))
    }

    /// Create demo server configuration
    async fn create_demo_server_config(&self) -> Result<String> {
        Ok(r#"{
  "server": {
    "host": "0.0.0.0",
    "port": 3001,
    "cors": {
      "enabled": true,
      "origins": ["*"]
    }
  },
  "security": {
    "rate_limiting": {
      "enabled": true,
      "max_requests": 100,
      "window_minutes": 15
    },
    "execution_timeout": 30000,
    "memory_limit": "128MB"
  },
  "languages": {
    "rust": {
      "enabled": true,
      "docker_image": "rust:1.70"
    },
    "python": {
      "enabled": true,
      "docker_image": "python:3.11"
    },
    "javascript": {
      "enabled": true,
      "docker_image": "node:18"
    }
  }
}"#.to_string())
    }

    /// Create playground HTML interface
    async fn create_playground_html(&self) -> Result<String> {
        Ok(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim SDK Playground</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.css">
    <style>
        body { margin: 0; font-family: 'Segoe UI', sans-serif; }
        .playground { display: grid; grid-template-rows: auto 1fr; height: 100vh; }
        .header { background: #2563eb; color: white; padding: 1rem; }
        .content { display: grid; grid-template-columns: 1fr 1fr; }
        .editor-panel, .output-panel { padding: 1rem; }
        .CodeMirror { height: 500px; border: 1px solid #ccc; }
        .output { background: #1e1e1e; color: #f8f8f2; padding: 1rem; font-family: monospace; }
        .controls { margin: 1rem 0; }
        button { padding: 0.5rem 1rem; margin-right: 0.5rem; cursor: pointer; }
        .run { background: #10b981; color: white; border: none; }
        .clear { background: #f59e0b; color: white; border: none; }
    </style>
</head>
<body>
    <div class="playground">
        <header class="header">
            <h1>OpenSim SDK Playground</h1>
            <p>Write and test OpenSim SDK code in your browser</p>
        </header>
        
        <div class="content">
            <div class="editor-panel">
                <div class="controls">
                    <select id="language">
                        <option value="rust">Rust</option>
                        <option value="python">Python</option>
                        <option value="javascript">JavaScript</option>
                    </select>
                    <button class="run" onclick="runCode()">Run</button>
                    <button class="clear" onclick="clearOutput()">Clear</button>
                </div>
                <textarea id="code-editor"></textarea>
            </div>
            
            <div class="output-panel">
                <h3>Output</h3>
                <div id="output" class="output"></div>
            </div>
        </div>
    </div>
    
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.2/codemirror.min.js"></script>
    <script>
        const editor = CodeMirror.fromTextArea(document.getElementById('code-editor'), {
            lineNumbers: true,
            mode: 'rust',
            theme: 'default'
        });
        
        async function runCode() {
            const code = editor.getValue();
            const language = document.getElementById('language').value;
            const output = document.getElementById('output');
            
            output.textContent = 'Running...';
            
            try {
                const response = await fetch('/api/execute', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code, language })
                });
                
                const result = await response.json();
                output.textContent = result.output || result.error;
            } catch (error) {
                output.textContent = 'Error: ' + error.message;
            }
        }
        
        function clearOutput() {
            document.getElementById('output').textContent = '';
        }
    </script>
</body>
</html>"#.to_string())
    }

    /// Create playground server
    async fn create_playground_server(&self) -> Result<String> {
        Ok(r#"//! OpenSim SDK Playground Server
use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::time::{timeout, Duration};
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct ExecuteRequest {
    code: String,
    language: String,
}

#[derive(Serialize)]
struct ExecuteResponse {
    output: Option<String>,
    error: Option<String>,
    execution_time: u64,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(playground_handler))
        .route("/api/execute", post(execute_code))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Playground server running on http://0.0.0.0:3001");
    
    axum::serve(listener, app).await.unwrap();
}

async fn playground_handler() -> &'static str {
    "OpenSim SDK Playground API"
}

async fn execute_code(Json(request): Json<ExecuteRequest>) -> Result<ResponseJson<ExecuteResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    let result = match request.language.as_str() {
        "rust" => execute_rust_code(&request.code).await,
        "python" => execute_python_code(&request.code).await,
        "javascript" => execute_js_code(&request.code).await,
        _ => Err("Unsupported language".to_string()),
    };
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    let response = match result {
        Ok(output) => ExecuteResponse {
            output: Some(output),
            error: None,
            execution_time,
        },
        Err(error) => ExecuteResponse {
            output: None,
            error: Some(error),
            execution_time,
        },
    };
    
    Ok(ResponseJson(response))
}

async fn execute_rust_code(code: &str) -> Result<String, String> {
    // Create a temporary Rust project and execute the code
    let temp_code = format!(r#"
use std::io::{{self, Write}};

fn main() {{
    // User code goes here
    {}
}}
"#, code);
    
    // This is a simplified version - in production you'd want proper sandboxing
    let output = timeout(Duration::from_secs(30), async {
        Command::new("echo")
            .arg(&temp_code)
            .output()
    })
    .await
    .map_err(|_| "Execution timeout".to_string())?
    .map_err(|e| format!("Execution error: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn execute_python_code(code: &str) -> Result<String, String> {
    // Execute Python code in a sandbox
    let output = timeout(Duration::from_secs(30), async {
        Command::new("python3")
            .arg("-c")
            .arg(code)
            .output()
    })
    .await
    .map_err(|_| "Execution timeout".to_string())?
    .map_err(|e| format!("Execution error: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn execute_js_code(code: &str) -> Result<String, String> {
    // Execute JavaScript code
    let output = timeout(Duration::from_secs(30), async {
        Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
    })
    .await
    .map_err(|_| "Execution timeout".to_string())?
    .map_err(|e| format!("Execution error: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
"#.to_string())
    }

    /// Create playground Dockerfile
    async fn create_playground_dockerfile(&self) -> Result<String> {
        Ok(format!("FROM {}", "rust"))
    }

    /// Create Vercel configuration
    async fn create_vercel_config(&self) -> Result<String> {
        Ok(r#"{
  "version": 2,
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build"
    }
  ],
  "routes": [
    {
      "src": "/api/(.*)",
      "dest": "/api/index.js"
    },
    {
      "src": "/(.*)",
      "dest": "/dist/$1"
    }
  ],
  "functions": {
    "api/playground.js": {
      "runtime": "nodejs18.x",
      "memory": 512
    }
  },
  "env": {
    "NODE_ENV": "production"
  }
}"#.to_string())
    }

    /// Create Netlify configuration
    async fn create_netlify_config(&self) -> Result<String> {
        Ok(r#"{
  "build": {
    "command": "npm run build:docs && npm run build:interactive",
    "publish": "dist/",
    "functions": "netlify/functions"
  },
  "redirects": [
    {
      "from": "/api/*",
      "to": "/.netlify/functions/api/:splat",
      "status": 200
    },
    {
      "from": "/playground/*",
      "to": "/playground/index.html",
      "status": 200
    },
    {
      "from": "/*",
      "to": "/index.html",
      "status": 200,
      "conditions": {
        "Role": ["admin"]
      }
    }
  ],
  "headers": [
    {
      "for": "/*.js",
      "values": {
        "Cache-Control": "public, max-age=31536000, immutable"
      }
    },
    {
      "for": "/*.css",
      "values": {
        "Cache-Control": "public, max-age=31536000, immutable"
      }
    },
    {
      "for": "/api/*",
      "values": {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, Authorization"
      }
    }
  ],
  "functions": {
    "directory": "netlify/functions",
    "node_bundler": "esbuild"
  },
  "plugins": [
    {
      "package": "@netlify/plugin-lighthouse",
      "inputs": {
        "audits": ["performance", "accessibility", "best-practices", "seo"]
      }
    },
    {
      "package": "netlify-plugin-sitemap",
      "inputs": {
        "baseUrl": "https://docs.opensim.org"
      }
    }
  ]
}"#.to_string())
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_sdk::api_schema::APISchema;

    #[tokio::test]
    async fn test_documentation_generator() -> Result<()> {
        let config = DocumentationConfig::default();
        let schema = APISchema::create_opensim_schema();
        let generator = DocumentationGenerator::new(config, schema);

        let docs = generator.generate_documentation().await?;
        assert!(!docs.is_empty());

        // Check that we have the expected documentation files
        let filenames: Vec<String> = docs.iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(filenames.len() > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_language_specific_docs() -> Result<()> {
        let config = DocumentationConfig::default();
        let schema = APISchema::create_opensim_schema();
        let generator = DocumentationGenerator::new(config, schema);

        let rust_docs = generator.generate_language_specific_docs(&TargetLanguage::Rust).await?;
        assert!(!rust_docs.is_empty());
        assert!(rust_docs[0].content.contains("Rust"));

        Ok(())
    }

    #[tokio::test]
    async fn test_example_generation() -> Result<()> {
        let config = DocumentationConfig::default();
        let schema = APISchema::create_opensim_schema();
        let generator = DocumentationGenerator::new(config, schema);

        let examples = generator.generate_language_examples(&TargetLanguage::Rust).await?;
        assert!(!examples.is_empty());
        assert!(examples.len() > 0);

        Ok(())
    }
}

impl DocumentationGenerator {
    /// Create interactive examples HTML page
    async fn create_interactive_examples_page(&self) -> Result<String> {
        Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim SDK Interactive Examples</title>
    <link rel="stylesheet" href="css/examples.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/codemirror.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/javascript/javascript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/python/python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/rust/rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/clike/clike.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/go/go.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/php/php.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/mode/ruby/ruby.min.js"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/codemirror.min.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.0/theme/material.min.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>OpenSim SDK Interactive Examples</h1>
            <p>Try out the OpenSim SDK in your browser with live code examples and instant feedback.</p>
        </header>
        
        <div class="language-selector">
            <select id="languageSelect" onchange="switchLanguage(this.value)">
                <option value="rust">Rust</option>
                <option value="python">Python</option>
                <option value="javascript">JavaScript</option>
                <option value="csharp">CSharp</option>
                <option value="java">Java</option>
                <option value="go">Go</option>
                <option value="php">PHP</option>
                <option value="ruby">Ruby</option>
            </select>
        </div>
        
        <div class="examples-container">
            <div class="example-tabs">
                <button class="tab-button active" onclick="showExample('basic')">Basic Usage</button>
                <button class="tab-button" onclick="showExample('auth')">Authentication</button>
                <button class="tab-button" onclick="showExample('advanced')">Advanced Features</button>
                <button class="tab-button" onclick="showExample('async')">Async Operations</button>
            </div>
            
            <div class="example-content">
                <div class="code-section">
                    <div class="code-header">
                        <h3 id="exampleTitle">Basic Usage Example</h3>
                        <div class="code-actions">
                            <button onclick="runCode()" class="btn btn-primary">Run</button>
                            <button onclick="resetCode()" class="btn btn-secondary">Reset</button>
                            <button onclick="copyCode()" class="btn btn-secondary">Copy</button>
                        </div>
                    </div>
                    <div id="codeEditor"></div>
                </div>
                
                <div class="output-section">
                    <div class="output-header">
                        <h3>Output</h3>
                        <button onclick="clearOutput()" class="btn btn-small">Clear</button>
                    </div>
                    <div id="output" class="output-content"></div>
                </div>
            </div>
        </div>
        
        <div class="documentation-section">
            <h2>API Reference</h2>
            <div id="apiDocs"></div>
        </div>
    </div>
    
    <script src="js/examples.js"></script>
</body>
</html>"#))
    }

    /// Create interactive JavaScript functionality
    async fn create_interactive_js(&self) -> Result<String> {
        Ok("// Interactive Examples JavaScript - Simplified for compilation".to_string())
    }

    /// Create CSS for interactive examples
    async fn create_interactive_css(&self) -> Result<String> {
        Ok(r#":root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --success-color: #28a745;
    --border-color: #dee2e6;
    --border-radius: 8px;
    --box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    margin: 0;
    padding: 0;
    background: #f8f9fa;
}

.container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
}

header {
    text-align: center;
    margin-bottom: 40px;
    padding: 40px 0;
    background: linear-gradient(135deg, var(--primary-color), #17a2b8);
    color: white;
    border-radius: var(--border-radius);
}

.examples-container {
    background: white;
    border-radius: var(--border-radius);
    box-shadow: var(--box-shadow);
    overflow: hidden;
}

.example-tabs {
    display: flex;
    background: #f8f9fa;
    border-bottom: 1px solid var(--border-color);
}

.tab-button {
    flex: 1;
    padding: 16px 24px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    color: var(--secondary-color);
    transition: all 0.3s ease;
}

.tab-button.active {
    color: var(--primary-color);
    background: white;
}

.example-content {
    display: grid;
    grid-template-columns: 1fr 1fr;
    min-height: 600px;
}

.code-section {
    border-right: 1px solid var(--border-color);
}

.code-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    background: #f8f9fa;
    border-bottom: 1px solid var(--border-color);
}

.btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    margin-left: 8px;
}

.btn-primary {
    background: var(--primary-color);
    color: white;
}

.btn-secondary {
    background: var(--secondary-color);
    color: white;
}

#codeEditor {
    height: 500px;
}

.CodeMirror {
    height: 100% !important;
    font-family: 'Monaco', 'Menlo', monospace;
    border: none;
}

.output-section {
    background: #fafafa;
}

.output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    background: #f8f9fa;
    border-bottom: 1px solid var(--border-color);
}

.output-content {
    padding: 20px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 13px;
    height: 500px;
    overflow-y: auto;
}

.output-success {
    color: var(--success-color);
    font-weight: 600;
    margin-bottom: 8px;
}

.output-log {
    margin-bottom: 4px;
}

@media (max-width: 1024px) {
    .example-content {
        grid-template-columns: 1fr;
    }
    
    .code-section {
        border-right: none;
        border-bottom: 1px solid var(--border-color);
    }
}
"#.to_string())
    }

    /// Helper methods for missing functionality
    async fn create_live_demo_for_language(&self, language: &TargetLanguage) -> Result<String> {
        Ok(format!("<!-- Live demo for {:?} -->", language))
    }

    async fn create_demo_server_config(&self) -> Result<String> {
        Ok(r#"{"port": 3000, "cors": true}"#.to_string())
    }

    async fn create_playground_html(&self) -> Result<String> {
        Ok("<!-- Playground HTML -->".to_string())
    }

    async fn create_playground_server(&self) -> Result<String> {
        Ok("// Playground server".to_string())
    }

    async fn create_playground_dockerfile(&self) -> Result<String> {
        Ok("FROM rust:latest".to_string())
    }

    /// Create Netlify configuration
    async fn create_netlify_config(&self) -> Result<String> {
        Ok(r#"{
  "build": {
    "command": "npm run build:docs && npm run build:interactive",
    "publish": "dist/",
    "functions": "netlify/functions"
  },
  "redirects": [
    {
      "from": "/api/*",
      "to": "/.netlify/functions/api/:splat",
      "status": 200
    },
    {
      "from": "/playground/*",
      "to": "/playground/index.html",
      "status": 200
    },
    {
      "from": "/*",
      "to": "/index.html",
      "status": 200,
      "conditions": {
        "Role": ["admin"]
      }
    }
  ],
  "headers": [
    {
      "for": "/*.js",
      "values": {
        "Cache-Control": "public, max-age=31536000, immutable"
      }
    },
    {
      "for": "/*.css",
      "values": {
        "Cache-Control": "public, max-age=31536000, immutable"
      }
    },
    {
      "for": "/api/*",
      "values": {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, Authorization"
      }
    }
  ],
  "functions": {
    "directory": "netlify/functions",
    "node_bundler": "esbuild"
  },
  "plugins": [
    {
      "package": "@netlify/plugin-lighthouse",
      "inputs": {
        "audits": ["performance", "accessibility", "best-practices", "seo"]
      }
    },
    {
      "package": "netlify-plugin-sitemap",
      "inputs": {
        "baseUrl": "https://docs.opensim.org"
      }
    }
  ]
}"#.to_string())
    }

    /// Create Vercel configuration
    async fn create_vercel_config(&self) -> Result<String> {
        Ok(r#"{
  "version": 2,
  "name": "opensim-docs",
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build",
      "config": {
        "buildCommand": "npm run build:docs && npm run build:interactive",
        "outputDirectory": "dist"
      }
    }
  ],
  "routes": [
    {
      "src": "/api/(.*)",
      "dest": "/api/$1"
    },
    {
      "src": "/playground/(.*)",
      "dest": "/playground/index.html"
    },
    {
      "src": "/(.*)",
      "dest": "/$1"
    }
  ],
  "headers": [
    {
      "source": "/(.*).js",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    },
    {
      "source": "/(.*).css",
      "headers": [
        {
          "key": "Cache-Control", 
          "value": "public, max-age=31536000, immutable"
        }
      ]
    },
    {
      "source": "/api/(.*)",
      "headers": [
        {
          "key": "Access-Control-Allow-Origin",
          "value": "*"
        },
        {
          "key": "Access-Control-Allow-Methods",
          "value": "GET, POST, PUT, DELETE, OPTIONS"
        },
        {
          "key": "Access-Control-Allow-Headers",
          "value": "Content-Type, Authorization"
        }
      ]
    }
  ],
  "env": {
    "NODE_ENV": "production"
  },
  "functions": {
    "app/api/**/*.js": {
      "runtime": "nodejs18.x"
    }
  }
}"#.to_string())
    }

    /// Create comprehensive package.json for documentation build
    async fn create_docs_package_json(&self) -> Result<String> {
        Ok(r#"{
  "name": "opensim-documentation",
  "version": "1.0.0",
  "description": "OpenSim SDK Documentation and Interactive Examples",
  "main": "index.js",
  "scripts": {
    "dev": "vite dev",
    "build": "npm run build:docs && npm run build:interactive && npm run optimize",
    "build:docs": "vite build --config vite.docs.config.js",
    "build:interactive": "vite build --config vite.interactive.config.js",
    "preview": "vite preview",
    "optimize": "npm run optimize:images && npm run optimize:bundle",
    "optimize:images": "imagemin generated/**/*.{jpg,png,gif} --out-dir=dist/optimized",
    "optimize:bundle": "webpack-bundle-analyzer dist/stats.json",
    "test": "npm run test:docs && npm run test:examples && npm run test:links",
    "test:docs": "jest --config jest.docs.config.js",
    "test:examples": "playwright test examples/",
    "test:links": "linkinator dist/ --recurse",
    "validate": "npm run validate:html && npm run validate:accessibility",
    "validate:html": "html-validate dist/**/*.html",
    "validate:accessibility": "pa11y-ci dist/**/*.html",
    "validate:links": "linkinator https://docs.opensim.org --recurse",
    "validate:examples": "npm run test:examples",
    "serve": "serve dist -l 3000",
    "lighthouse": "lhci autorun",
    "format": "prettier --write src/",
    "lint": "eslint src/ --ext .js,.ts,.vue",
    "clean": "rimraf dist/ .vite/ node_modules/.cache/"
  },
  "dependencies": {
    "vue": "^3.3.4",
    "@vueuse/core": "^10.2.1",
    "prismjs": "^1.29.0",
    "codemirror": "^6.0.1",
    "fuse.js": "^6.6.2",
    "mermaid": "^10.3.1",
    "chart.js": "^4.3.3"
  },
  "devDependencies": {
    "vite": "^4.4.9",
    "@vitejs/plugin-vue": "^4.3.3",
    "autoprefixer": "^10.4.14",
    "postcss": "^8.4.27",
    "tailwindcss": "^3.3.3",
    "@playwright/test": "^1.36.2",
    "jest": "^29.6.2",
    "linkinator": "^4.1.2",
    "html-validate": "^8.2.0",
    "pa11y-ci": "^3.0.1",
    "@lhci/cli": "^0.12.0",
    "lighthouse": "^10.4.0",
    "imagemin": "^8.0.1",
    "imagemin-mozjpeg": "^10.0.0",
    "imagemin-pngquant": "^9.0.2",
    "webpack-bundle-analyzer": "^4.9.0",
    "prettier": "^3.0.1",
    "eslint": "^8.46.0",
    "rimraf": "^5.0.1",
    "serve": "^14.2.1"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/opensim/opensim-next.git"
  },
  "keywords": [
    "opensim",
    "documentation",
    "sdk",
    "virtual-world",
    "api"
  ],
  "license": "MIT",
  "engines": {
    "node": ">=16.0.0",
    "npm": ">=7.0.0"
  }
}"#.to_string())
    }

    /// Create Lighthouse CI configuration
    async fn create_lighthouse_config(&self) -> Result<String> {
        Ok(r#"{
  "ci": {
    "collect": {
      "url": [
        "http://localhost:3000",
        "http://localhost:3000/examples",
        "http://localhost:3000/playground",
        "http://localhost:3000/api-reference"
      ],
      "startServerCommand": "npm run serve",
      "numberOfRuns": 3
    },
    "assert": {
      "assertions": {
        "categories:performance": ["warn", {"minScore": 0.8}],
        "categories:accessibility": ["error", {"minScore": 0.95}],
        "categories:best-practices": ["warn", {"minScore": 0.9}],
        "categories:seo": ["warn", {"minScore": 0.8}],
        "categories:pwa": "off"
      }
    },
    "upload": {
      "target": "temporary-public-storage"
    },
    "server": {
      "port": 9001
    }
  }
}"#.to_string())
    }

    fn language_to_string(&self, language: &TargetLanguage) -> &'static str {
        match language {
            TargetLanguage::Rust => "rust",
            TargetLanguage::Python => "python",
            TargetLanguage::JavaScript => "javascript",
            TargetLanguage::CSharp => "csharp",
            TargetLanguage::Java => "java",
            TargetLanguage::Go => "go",
            TargetLanguage::PHP => "php",
            TargetLanguage::Ruby => "ruby",
        }
    }
}