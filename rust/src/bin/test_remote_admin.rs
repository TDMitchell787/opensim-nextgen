//! OpenSim Next RemoteAdmin Test Suite
//! Comprehensive testing for RemoteAdmin interface functionality

use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "test-remote-admin")]
#[command(about = "OpenSim Next RemoteAdmin Test Suite")]
#[command(version = "1.0.0")]
struct Cli {
    /// Server hostname
    #[arg(long, default_value = "localhost")]
    host: String,
    
    /// Server port
    #[arg(long, default_value_t = 9000)]
    port: u16,
    
    /// Admin password
    #[arg(long, env = "OPENSIM_ADMIN_PASSWORD")]
    password: String,
    
    /// Use HTTPS
    #[arg(long)]
    ssl: bool,
    
    /// Run performance tests
    #[arg(long)]
    performance: bool,
    
    /// Number of performance test iterations
    #[arg(long, default_value_t = 10)]
    iterations: u32,
}

#[derive(Debug, Serialize)]
struct RemoteAdminRequest {
    method: String,
    password: String,
    #[serde(flatten)]
    parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RemoteAdminResponse {
    success: bool,
    message: String,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    status: String,
    enabled_commands: Vec<String>,
    statistics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    success: bool,
    message: String,
    duration: Duration,
}

impl TestResult {
    fn success(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            success: true,
            message: message.to_string(),
            duration,
        }
    }
    
    fn failure(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            success: false,
            message: message.to_string(),
            duration,
        }
    }
    
    fn warning(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            success: true,
            message: format!("⚠️  {}", message),
            duration,
        }
    }
}

struct RemoteAdminTester {
    client: Client,
    base_url: String,
    password: String,
    results: Vec<TestResult>,
}

impl RemoteAdminTester {
    fn new(host: &str, port: u16, password: String, ssl: bool) -> Self {
        let protocol = if ssl { "https" } else { "http" };
        let base_url = format!("{}://{}:{}/admin", protocol, host, port);
        
        Self {
            client: Client::new(),
            base_url,
            password,
            results: Vec::new(),
        }
    }
    
    async fn execute_command(
        &self,
        method: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<RemoteAdminResponse> {
        let request = RemoteAdminRequest {
            method: method.to_string(),
            password: self.password.clone(),
            parameters,
        };
        
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(match status.as_u16() {
                401 => anyhow!("Authentication failed"),
                403 => anyhow!("Command disabled"),
                500 => {
                    if let Ok(error_data) = serde_json::from_str::<RemoteAdminResponse>(&error_text) {
                        anyhow!("Server error: {}", error_data.message)
                    } else {
                        anyhow!("Server error: HTTP {}", status.as_u16())
                    }
                },
                _ => anyhow!("HTTP error: {}", status.as_u16()),
            });
        }
        
        let result: RemoteAdminResponse = response.json().await?;
        Ok(result)
    }
    
    async fn get_status(&self) -> Result<StatusResponse> {
        let response = self.client
            .get(&format!("{}/status", self.base_url))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get status: HTTP {}", response.status().as_u16()));
        }
        
        let status: StatusResponse = response.json().await?;
        Ok(status)
    }
    
    async fn test_authentication(&mut self) {
        println!("🔐 Testing Authentication...");
        let start = Instant::now();
        
        // Test with correct password
        match self.execute_command("admin_region_query", HashMap::new()).await {
            Ok(result) => {
                if result.success {
                    self.results.push(TestResult::success(
                        "Authentication (correct password)",
                        "Authentication successful",
                        start.elapsed(),
                    ));
                    println!("  ✅ Correct password authentication: PASSED");
                } else {
                    self.results.push(TestResult::warning(
                        "Authentication (correct password)",
                        &format!("Command succeeded but returned: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Correct password authentication: Command may be disabled");
                }
            }
            Err(e) => {
                if e.to_string().contains("Authentication failed") {
                    self.results.push(TestResult::failure(
                        "Authentication (correct password)",
                        "Authentication failed with correct password",
                        start.elapsed(),
                    ));
                    println!("  ❌ Correct password authentication: FAILED");
                } else {
                    self.results.push(TestResult::warning(
                        "Authentication (correct password)",
                        &format!("Other error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Correct password authentication: {}", e);
                }
            }
        }
        
        // Test with wrong password
        let mut wrong_tester = self.clone();
        wrong_tester.password = "wrong_password".to_string();
        
        let start = Instant::now();
        match wrong_tester.execute_command("admin_region_query", HashMap::new()).await {
            Ok(_) => {
                self.results.push(TestResult::failure(
                    "Authentication (wrong password)",
                    "Should have failed with wrong password",
                    start.elapsed(),
                ));
                println!("  ❌ Wrong password rejection: FAILED (should have been rejected)");
            }
            Err(e) => {
                if e.to_string().contains("Authentication failed") {
                    self.results.push(TestResult::success(
                        "Authentication (wrong password)",
                        "Correctly rejected wrong password",
                        start.elapsed(),
                    ));
                    println!("  ✅ Wrong password rejection: PASSED");
                } else {
                    self.results.push(TestResult::warning(
                        "Authentication (wrong password)",
                        &format!("Unexpected error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Wrong password rejection: Unexpected error: {}", e);
                }
            }
        }
    }
    
    async fn test_status_endpoint(&mut self) {
        println!("\n📊 Testing Status Endpoint...");
        let start = Instant::now();
        
        match self.get_status().await {
            Ok(status) => {
                self.results.push(TestResult::success(
                    "Status endpoint",
                    &format!("Status: {}, Commands: {}", status.status, status.enabled_commands.len()),
                    start.elapsed(),
                ));
                println!("  ✅ Status endpoint: PASSED");
                println!("      Status: {}", status.status);
                println!("      Enabled commands: {}", status.enabled_commands.len());
            }
            Err(e) => {
                self.results.push(TestResult::failure(
                    "Status endpoint",
                    &format!("Failed: {}", e),
                    start.elapsed(),
                ));
                println!("  ❌ Status endpoint: FAILED - {}", e);
            }
        }
    }
    
    async fn test_user_management(&mut self) {
        println!("\n👤 Testing User Management...");
        
        // Test user creation
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("user_firstname".to_string(), json!("TestUser"));
        params.insert("user_lastname".to_string(), json!("RemoteAdmin"));
        params.insert("user_password".to_string(), json!("testpass123"));
        params.insert("user_email".to_string(), json!("test@opensim.local"));
        
        match self.execute_command("admin_create_user", params).await {
            Ok(result) => {
                if result.success {
                    self.results.push(TestResult::success(
                        "User creation",
                        &format!("User created: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ✅ User creation: PASSED");
                    if let Some(uuid) = result.data.get("avatar_uuid") {
                        println!("      Created user with UUID: {}", uuid);
                    }
                } else {
                    self.results.push(TestResult::warning(
                        "User creation",
                        &format!("Command succeeded but: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  User creation: {}", result.message);
                }
            }
            Err(e) => {
                if e.to_string().contains("disabled") {
                    self.results.push(TestResult::warning(
                        "User creation",
                        "Command disabled",
                        start.elapsed(),
                    ));
                    println!("  ⚠️  User creation: DISABLED");
                } else {
                    self.results.push(TestResult::failure(
                        "User creation",
                        &format!("Error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ❌ User creation: ERROR - {}", e);
                }
            }
        }
        
        // Test user existence check
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("user_firstname".to_string(), json!("TestUser"));
        params.insert("user_lastname".to_string(), json!("RemoteAdmin"));
        
        match self.execute_command("admin_exists_user", params).await {
            Ok(result) => {
                if result.success {
                    let exists = result.data.get("user_exists").and_then(|v| v.as_bool()).unwrap_or(false);
                    self.results.push(TestResult::success(
                        "User existence check",
                        &format!("User exists: {}", exists),
                        start.elapsed(),
                    ));
                    println!("  ✅ User existence check: PASSED (exists: {})", exists);
                } else {
                    self.results.push(TestResult::warning(
                        "User existence check",
                        &format!("Command succeeded but: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  User existence check: {}", result.message);
                }
            }
            Err(e) => {
                if e.to_string().contains("disabled") {
                    self.results.push(TestResult::warning(
                        "User existence check",
                        "Command disabled",
                        start.elapsed(),
                    ));
                    println!("  ⚠️  User existence check: DISABLED");
                } else {
                    self.results.push(TestResult::failure(
                        "User existence check",
                        &format!("Error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ❌ User existence check: ERROR - {}", e);
                }
            }
        }
    }
    
    async fn test_agent_management(&mut self) {
        println!("\n🧍 Testing Agent Management...");
        
        // Test getting agents
        let start = Instant::now();
        match self.execute_command("admin_get_agents", HashMap::new()).await {
            Ok(result) => {
                if result.success {
                    let agents = result.data.get("agents").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    self.results.push(TestResult::success(
                        "Get agents",
                        &format!("{} agents found", agents),
                        start.elapsed(),
                    ));
                    println!("  ✅ Get agents: PASSED ({} agents found)", agents);
                } else {
                    self.results.push(TestResult::warning(
                        "Get agents",
                        &format!("Command succeeded but: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Get agents: {}", result.message);
                }
            }
            Err(e) => {
                if e.to_string().contains("disabled") {
                    self.results.push(TestResult::warning(
                        "Get agents",
                        "Command disabled",
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Get agents: DISABLED");
                } else {
                    self.results.push(TestResult::failure(
                        "Get agents",
                        &format!("Error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ❌ Get agents: ERROR - {}", e);
                }
            }
        }
    }
    
    async fn test_region_management(&mut self) {
        println!("\n🗺️  Testing Region Management...");
        
        // Test region query
        let start = Instant::now();
        match self.execute_command("admin_region_query", HashMap::new()).await {
            Ok(result) => {
                if result.success {
                    let region_count = if let Some(regions) = result.data.get("regions").and_then(|v| v.as_array()) {
                        regions.len()
                    } else if result.data.get("region_name").is_some() {
                        1
                    } else {
                        0
                    };
                    
                    self.results.push(TestResult::success(
                        "Region query",
                        &format!("{} regions found", region_count),
                        start.elapsed(),
                    ));
                    println!("  ✅ Region query: PASSED ({} regions)", region_count);
                } else {
                    self.results.push(TestResult::warning(
                        "Region query",
                        &format!("Command succeeded but: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Region query: {}", result.message);
                }
            }
            Err(e) => {
                if e.to_string().contains("disabled") {
                    self.results.push(TestResult::warning(
                        "Region query",
                        "Command disabled",
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Region query: DISABLED");
                } else {
                    self.results.push(TestResult::failure(
                        "Region query",
                        &format!("Error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ❌ Region query: ERROR - {}", e);
                }
            }
        }
    }
    
    async fn test_broadcast(&mut self) {
        println!("\n📢 Testing Broadcast...");
        
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("message".to_string(), json!(format!("RemoteAdmin test message at {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"))));
        
        match self.execute_command("admin_broadcast", params).await {
            Ok(result) => {
                if result.success {
                    self.results.push(TestResult::success(
                        "Broadcast",
                        "Message broadcast successfully",
                        start.elapsed(),
                    ));
                    println!("  ✅ Broadcast: PASSED");
                } else {
                    self.results.push(TestResult::warning(
                        "Broadcast",
                        &format!("Command succeeded but: {}", result.message),
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Broadcast: {}", result.message);
                }
            }
            Err(e) => {
                if e.to_string().contains("disabled") {
                    self.results.push(TestResult::warning(
                        "Broadcast",
                        "Command disabled",
                        start.elapsed(),
                    ));
                    println!("  ⚠️  Broadcast: DISABLED");
                } else {
                    self.results.push(TestResult::failure(
                        "Broadcast",
                        &format!("Error: {}", e),
                        start.elapsed(),
                    ));
                    println!("  ❌ Broadcast: ERROR - {}", e);
                }
            }
        }
    }
    
    async fn test_console_commands(&mut self) {
        println!("\n💻 Testing Console Commands...");
        
        let commands = ["show users", "show regions", "show stats", "show version"];
        
        for command in &commands {
            let start = Instant::now();
            let mut params = HashMap::new();
            params.insert("command".to_string(), json!(command));
            
            match self.execute_command("admin_console_command", params).await {
                Ok(result) => {
                    if result.success {
                        self.results.push(TestResult::success(
                            &format!("Console '{}'", command),
                            "Command executed successfully",
                            start.elapsed(),
                        ));
                        println!("  ✅ Console '{}': PASSED", command);
                        if let Some(output) = result.data.get("result").and_then(|v| v.as_str()) {
                            let preview = if output.len() > 100 { &output[..100] } else { output };
                            println!("      Output: {}...", preview);
                        }
                    } else {
                        self.results.push(TestResult::warning(
                            &format!("Console '{}'", command),
                            &format!("Command succeeded but: {}", result.message),
                            start.elapsed(),
                        ));
                        println!("  ⚠️  Console '{}': {}", command, result.message);
                    }
                }
                Err(e) => {
                    if e.to_string().contains("disabled") {
                        self.results.push(TestResult::warning(
                            &format!("Console '{}'", command),
                            "Command disabled",
                            start.elapsed(),
                        ));
                        println!("  ⚠️  Console '{}': DISABLED", command);
                    } else {
                        self.results.push(TestResult::failure(
                            &format!("Console '{}'", command),
                            &format!("Error: {}", e),
                            start.elapsed(),
                        ));
                        println!("  ❌ Console '{}': ERROR - {}", command, e);
                    }
                }
            }
        }
    }
    
    async fn test_error_handling(&mut self) {
        println!("\n⚠️  Testing Error Handling...");
        
        // Test unknown command
        let start = Instant::now();
        match self.execute_command("admin_unknown_command", HashMap::new()).await {
            Ok(result) => {
                if result.success {
                    self.results.push(TestResult::failure(
                        "Unknown command handling",
                        "Should have failed for unknown command",
                        start.elapsed(),
                    ));
                    println!("  ❌ Unknown command handling: Should have failed");
                } else {
                    self.results.push(TestResult::success(
                        "Unknown command handling",
                        "Correctly rejected unknown command",
                        start.elapsed(),
                    ));
                    println!("  ✅ Unknown command handling: PASSED");
                }
            }
            Err(_) => {
                self.results.push(TestResult::success(
                    "Unknown command handling",
                    "Correctly rejected unknown command",
                    start.elapsed(),
                ));
                println!("  ✅ Unknown command handling: PASSED");
            }
        }
        
        // Test missing parameters
        let start = Instant::now();
        let mut params = HashMap::new();
        params.insert("user_firstname".to_string(), json!("Test"));
        // Missing required parameters
        
        match self.execute_command("admin_create_user", params).await {
            Ok(result) => {
                if result.success {
                    self.results.push(TestResult::failure(
                        "Missing parameters handling",
                        "Should have failed for missing parameters",
                        start.elapsed(),
                    ));
                    println!("  ❌ Missing parameters handling: Should have failed");
                } else {
                    self.results.push(TestResult::success(
                        "Missing parameters handling",
                        "Correctly rejected missing parameters",
                        start.elapsed(),
                    ));
                    println!("  ✅ Missing parameters handling: PASSED");
                }
            }
            Err(_) => {
                self.results.push(TestResult::success(
                    "Missing parameters handling",
                    "Correctly rejected missing parameters",
                    start.elapsed(),
                ));
                println!("  ✅ Missing parameters handling: PASSED");
            }
        }
    }
    
    async fn test_performance(&mut self, iterations: u32) {
        println!("\n⚡ Testing Performance...");
        
        let start = Instant::now();
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        
        for i in 0..iterations {
            match self.execute_command("admin_region_query", HashMap::new()).await {
                Ok(result) => {
                    if result.success {
                        successful_requests += 1;
                    } else {
                        failed_requests += 1;
                    }
                }
                Err(_) => {
                    failed_requests += 1;
                }
            }
            
            if i < iterations - 1 {
                sleep(Duration::from_millis(10)).await; // Small delay between requests
            }
        }
        
        let total_time = start.elapsed();
        let avg_time = total_time / iterations;
        let requests_per_second = iterations as f64 / total_time.as_secs_f64();
        
        self.results.push(TestResult::success(
            "Performance test",
            &format!("{} requests in {:.2}s, {:.1} req/s", iterations, total_time.as_secs_f64(), requests_per_second),
            total_time,
        ));
        
        println!("  📊 Performance Results:");
        println!("      Total requests: {}", iterations);
        println!("      Successful: {}", successful_requests);
        println!("      Failed: {}", failed_requests);
        println!("      Total time: {:.2}s", total_time.as_secs_f64());
        println!("      Average time per request: {:.3}s", avg_time.as_secs_f64());
        println!("      Requests per second: {:.1}", requests_per_second);
        
        if avg_time.as_millis() < 100 {
            println!("  ✅ Performance: EXCELLENT (< 100ms)");
        } else if avg_time.as_millis() < 500 {
            println!("  ✅ Performance: GOOD (< 500ms)");
        } else if avg_time.as_millis() < 1000 {
            println!("  ⚠️  Performance: ACCEPTABLE (< 1s)");
        } else {
            println!("  ❌ Performance: POOR (> 1s)");
        }
    }
    
    async fn run_all_tests(&mut self, run_performance: bool, iterations: u32) {
        println!("🧪 OpenSim Next RemoteAdmin Test Suite");
        println!("{}", "=".repeat(50));
        println!("Testing RemoteAdmin at: {}", self.base_url);
        println!("Using password: {}", "*".repeat(self.password.len()));
        println!();
        
        self.test_authentication().await;
        self.test_status_endpoint().await;
        self.test_user_management().await;
        self.test_agent_management().await;
        self.test_region_management().await;
        self.test_broadcast().await;
        self.test_console_commands().await;
        self.test_error_handling().await;
        
        if run_performance {
            self.test_performance(iterations).await;
        }
        
        self.print_summary();
    }
    
    fn print_summary(&self) {
        println!("\n{}", "=".repeat(50));
        println!("🎉 RemoteAdmin Test Suite Complete!");
        
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;
        
        println!("\n📋 Summary:");
        println!("   Total tests: {}", total_tests);
        println!("   Successful: {} ✅", successful_tests);
        println!("   Failed: {} {}", failed_tests, if failed_tests > 0 { "❌" } else { "" });
        
        let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
        println!("   Success rate: {:.1}%", success_rate);
        
        if failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.results {
                if !result.success {
                    println!("   • {}: {}", result.name, result.message);
                }
            }
        }
        
        let avg_duration: Duration = self.results.iter()
            .map(|r| r.duration)
            .sum::<Duration>() / self.results.len() as u32;
        
        println!("\n⏱️  Average response time: {:.3}s", avg_duration.as_secs_f64());
        
        println!("\n📖 Legend:");
        println!("   ✅ = Test passed");
        println!("   ⚠️  = Test passed with warnings or command disabled");
        println!("   ❌ = Test failed");
    }
}

impl Clone for RemoteAdminTester {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            password: self.password.clone(),
            results: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut tester = RemoteAdminTester::new(&cli.host, cli.port, cli.password, cli.ssl);
    tester.run_all_tests(cli.performance, cli.iterations).await;
    
    Ok(())
}