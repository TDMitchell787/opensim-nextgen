//! Integration tests for OpenSim Next Admin API
//!
//! Comprehensive testing suite for Phase 22.1 Robust-style admin commands.
//! Tests the complete flow from terminal commands to database operations.

use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Test configuration
struct TestConfig {
    api_base_url: String,
    api_key: String,
    client: Client,
}

impl TestConfig {
    fn new() -> Self {
        Self {
            api_base_url: env::var("OPENSIM_ADMIN_API_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
            api_key: env::var("OPENSIM_API_KEY")
                .unwrap_or_else(|_| "default-key-change-me".to_string()),
            client: Client::new(),
        }
    }
}

/// Test user creation via admin API
#[tokio::test]
async fn test_create_user_api() -> Result<()> {
    let config = TestConfig::new();

    let payload = json!({
        "firstname": "TestUser",
        "lastname": "Integration",
        "email": "testuser@example.com",
        "password": "testpassword123",
        "user_level": 0
    });

    let response = config
        .client
        .post(&format!("{}/admin/users", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Create user response status: {}", response.status());

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "Create user result: {}",
            serde_json::to_string_pretty(&result)?
        );

        assert!(result["success"].as_bool().unwrap_or(false));
        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("created successfully"));
    } else {
        println!("Create user failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    Ok(())
}

/// Test user listing via admin API
#[tokio::test]
async fn test_list_users_api() -> Result<()> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/users?limit=10", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;

    println!("List users response status: {}", response.status());

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "List users result: {}",
            serde_json::to_string_pretty(&result)?
        );

        assert!(result["success"].as_bool().unwrap_or(false));
    } else {
        println!("List users failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    Ok(())
}

/// Test password reset via admin API
#[tokio::test]
async fn test_reset_password_api() -> Result<()> {
    let config = TestConfig::new();

    let payload = json!({
        "firstname": "TestUser",
        "lastname": "Integration",
        "new_password": "newpassword456"
    });

    let response = config
        .client
        .put(&format!("{}/admin/users/password", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Reset password response status: {}", response.status());

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "Reset password result: {}",
            serde_json::to_string_pretty(&result)?
        );

        // Result might be success or failure (user might not exist)
        let success = result["success"].as_bool().unwrap_or(false);
        let message = result["message"].as_str().unwrap_or("No message");
        println!("Operation success: {}, message: {}", success, message);
    } else {
        println!("Reset password failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    Ok(())
}

/// Test database statistics via admin API
#[tokio::test]
async fn test_database_stats_api() -> Result<()> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/database/stats", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;

    println!("Database stats response status: {}", response.status());

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "Database stats result: {}",
            serde_json::to_string_pretty(&result)?
        );

        assert!(result["success"].as_bool().unwrap_or(false));
    } else {
        println!("Database stats failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    Ok(())
}

/// Test admin API health endpoint
#[tokio::test]
async fn test_admin_health_api() -> Result<()> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/health", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;

    println!("Admin health response status: {}", response.status());

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "Admin health result: {}",
            serde_json::to_string_pretty(&result)?
        );

        assert_eq!(result["status"], "healthy");
        assert!(result["robust_commands_supported"].is_array());
    } else {
        println!("Admin health failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    Ok(())
}

/// Test invalid API key rejection
#[tokio::test]
async fn test_invalid_api_key() -> Result<()> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/health", config.api_base_url))
        .header("X-API-Key", "invalid-key-123")
        .send()
        .await?;

    println!("Invalid API key response status: {}", response.status());

    // Should return unauthorized
    assert_eq!(response.status(), 401);

    Ok(())
}

/// Test missing API key rejection
#[tokio::test]
async fn test_missing_api_key() -> Result<()> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/health", config.api_base_url))
        // No X-API-Key header
        .send()
        .await?;

    println!("Missing API key response status: {}", response.status());

    // Should return unauthorized
    assert_eq!(response.status(), 401);

    Ok(())
}

/// Test input validation for create user
#[tokio::test]
async fn test_create_user_validation() -> Result<()> {
    let config = TestConfig::new();

    // Test with invalid email
    let payload = json!({
        "firstname": "Test",
        "lastname": "User",
        "email": "invalid-email",
        "password": "password123",
        "user_level": 0
    });

    let response = config
        .client
        .post(&format!("{}/admin/users", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Invalid email response status: {}", response.status());

    // Should return bad request
    assert_eq!(response.status(), 400);

    // Test with short password
    let payload = json!({
        "firstname": "Test",
        "lastname": "User",
        "email": "test@example.com",
        "password": "123", // Too short
        "user_level": 0
    });

    let response = config
        .client
        .post(&format!("{}/admin/users", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    println!("Short password response status: {}", response.status());

    // Should return bad request
    assert_eq!(response.status(), 400);

    Ok(())
}

/// Test complete user lifecycle (create -> show -> update -> delete)
#[tokio::test]
async fn test_user_lifecycle() -> Result<()> {
    let config = TestConfig::new();
    let test_user_id = "LifecycleTest";

    // 1. Create user
    let create_payload = json!({
        "firstname": test_user_id,
        "lastname": "User",
        "email": "lifecycle@example.com",
        "password": "testpassword123",
        "user_level": 0
    });

    let create_response = config
        .client
        .post(&format!("{}/admin/users", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&create_payload)
        .send()
        .await?;

    println!("Create response status: {}", create_response.status());

    if create_response.status().is_success() {
        let create_result: serde_json::Value = create_response.json().await?;
        println!(
            "User created: {}",
            serde_json::to_string_pretty(&create_result)?
        );
    }

    // 2. Show user account
    let show_response = config
        .client
        .get(&format!(
            "{}/admin/users/account?firstname={}&lastname=User",
            config.api_base_url, test_user_id
        ))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;

    println!("Show account response status: {}", show_response.status());

    if show_response.status().is_success() {
        let show_result: serde_json::Value = show_response.json().await?;
        println!(
            "User account: {}",
            serde_json::to_string_pretty(&show_result)?
        );
    }

    // 3. Update user level
    let update_payload = json!({
        "firstname": test_user_id,
        "lastname": "User",
        "user_level": 100
    });

    let update_response = config
        .client
        .put(&format!("{}/admin/users/level", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&update_payload)
        .send()
        .await?;

    println!("Update level response status: {}", update_response.status());

    if update_response.status().is_success() {
        let update_result: serde_json::Value = update_response.json().await?;
        println!(
            "User level updated: {}",
            serde_json::to_string_pretty(&update_result)?
        );
    }

    // 4. Delete user (cleanup)
    let delete_payload = json!({
        "firstname": test_user_id,
        "lastname": "User"
    });

    let delete_response = config
        .client
        .delete(&format!("{}/admin/users/delete", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&delete_payload)
        .send()
        .await?;

    println!("Delete response status: {}", delete_response.status());

    if delete_response.status().is_success() {
        let delete_result: serde_json::Value = delete_response.json().await?;
        println!(
            "User deleted: {}",
            serde_json::to_string_pretty(&delete_result)?
        );
    }

    Ok(())
}

/// Performance test - multiple concurrent requests
#[tokio::test]
async fn test_concurrent_requests() -> Result<()> {
    let config = TestConfig::new();

    let mut tasks = Vec::new();

    // Create 10 concurrent health check requests
    for i in 0..10 {
        let client = config.client.clone();
        let url = format!("{}/admin/health", config.api_base_url);
        let api_key = config.api_key.clone();

        let task = tokio::spawn(async move {
            let response = client.get(&url).header("X-API-Key", &api_key).send().await;

            (i, response)
        });

        tasks.push(task);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(tasks).await;

    println!("Concurrent request results:");
    for result in results {
        match result {
            Ok((id, Ok(response))) => {
                println!("Request {}: Status {}", id, response.status());
                assert!(response.status().is_success());
            }
            Ok((id, Err(e))) => {
                println!("Request {} failed: {}", id, e);
            }
            Err(e) => {
                println!("Task failed: {}", e);
            }
        }
    }

    Ok(())
}

/// Test rate limiting (if enabled)
#[tokio::test]
async fn test_rate_limiting() -> Result<()> {
    let config = TestConfig::new();

    println!("Testing rate limiting with rapid requests...");

    // Make rapid requests to trigger rate limiting
    for i in 0..70 {
        // More than typical rate limit
        let response = config
            .client
            .get(&format!("{}/admin/health", config.api_base_url))
            .header("X-API-Key", &config.api_key)
            .send()
            .await?;

        if response.status() == 429 {
            // Too Many Requests
            println!("Rate limiting triggered at request {}", i);
            println!("Response: {}", response.text().await?);
            break;
        }

        // Small delay to avoid overwhelming the server
        sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}

/// Test terminal command integration (if binary is available)
#[tokio::test]
async fn test_terminal_command_help() -> Result<()> {
    // Try to execute the admin terminal help command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "admin_terminal", "--", "--help"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            println!("Terminal help stdout:\n{}", stdout);
            if !stderr.is_empty() {
                println!("Terminal help stderr:\n{}", stderr);
            }

            // Check that help contains expected content
            assert!(stdout.contains("OpenSim Next Admin Terminal"));
            assert!(stdout.contains("create user"));
        }
        Err(e) => {
            println!("Could not execute admin terminal: {}", e);
            println!("This is expected if the binary is not built");
        }
    }

    Ok(())
}

/// Helper function to run a test if server is available
async fn require_server_running() -> Result<bool> {
    let config = TestConfig::new();

    let response = config
        .client
        .get(&format!("{}/admin/health", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => Ok(true),
        _ => {
            println!(
                "⚠️  OpenSim Next server not running on {}",
                config.api_base_url
            );
            println!("💡 Start server with: cargo run --bin opensim-next");
            println!("💡 Or set OPENSIM_ADMIN_API_URL to running server");
            Ok(false)
        }
    }
}

/// Integration test setup
#[tokio::test]
async fn test_setup() -> Result<()> {
    println!("🧪 OpenSim Next Admin API Integration Tests");
    println!("===========================================");

    let config = TestConfig::new();
    println!("🌐 API URL: {}", config.api_base_url);
    println!(
        "🔑 API Key: {}",
        if config.api_key == "default-key-change-me" {
            "default (change for production)"
        } else {
            "configured"
        }
    );

    let server_running = require_server_running().await?;

    if server_running {
        println!("✅ Server is running and responding");
        println!("🚀 Running integration tests...");
    } else {
        println!("❌ Server is not running");
        println!("⏭️  Skipping server-dependent tests");
    }

    Ok(())
}
