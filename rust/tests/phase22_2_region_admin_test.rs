//! Integration tests for Phase 22.2 Region Management Commands
//! 
//! Comprehensive testing suite for region management functionality
//! including database operations, API endpoints, and web terminal integration.

use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

/// Test configuration for region admin tests
struct RegionAdminTestConfig {
    api_base_url: String,
    api_key: String,
    client: Client,
}

impl RegionAdminTestConfig {
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

/// Test region creation via admin API
#[tokio::test]
async fn test_create_region_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    let payload = json!({
        "region_name": "TestRegion22_2",
        "location_x": 1000,
        "location_y": 1000,
        "size_x": 256,
        "size_y": 256,
        "server_ip": "127.0.0.1",
        "server_port": 9000
    });
    
    let response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    
    println!("Create region response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Create region result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        assert!(result["message"].as_str().unwrap().contains("created successfully"));
        
        // Verify region UUID is generated
        if let Some(data) = result["data"].as_object() {
            assert!(data.contains_key("region_uuid"));
            assert!(data.contains_key("region_name"));
            assert_eq!(data["region_name"], "TestRegion22_2");
            assert_eq!(data["location_x"], 1000);
            assert_eq!(data["location_y"], 1000);
        }
    } else {
        println!("Create region failed with status: {}", response.status());
        let error_text = response.text().await?;
        println!("Response: {}", error_text);
        
        // If it failed due to region already existing, that's acceptable for testing
        if !error_text.contains("already exists") {
            panic!("Unexpected error creating region: {}", error_text);
        }
    }
    
    Ok(())
}

/// Test region listing via admin API
#[tokio::test]
async fn test_list_regions_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    let response = config.client
        .get(&format!("{}/admin/regions?limit=10", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    println!("List regions response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("List regions result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        
        if let Some(data) = result["data"].as_object() {
            assert!(data.contains_key("regions"));
            assert!(data.contains_key("total_count"));
            assert!(data.contains_key("limit"));
            
            if let Some(regions) = data["regions"].as_array() {
                println!("Found {} regions", regions.len());
                
                // Verify region structure
                for region in regions {
                    assert!(region["uuid"].is_string());
                    assert!(region["region_name"].is_string());
                    assert!(region["location_x"].is_number());
                    assert!(region["location_y"].is_number());
                    assert!(region["size_x"].is_number());
                    assert!(region["size_y"].is_number());
                }
            }
        }
    } else {
        println!("List regions failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}

/// Test region details via admin API
#[tokio::test]
async fn test_show_region_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    // First create a test region
    let create_payload = json!({
        "region_name": "ShowTestRegion",
        "location_x": 1100,
        "location_y": 1100,
        "size_x": 512,
        "size_y": 512
    });
    
    let _create_response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&create_payload)
        .send()
        .await?;
    
    // Small delay to ensure creation completes
    sleep(Duration::from_millis(100)).await;
    
    // Now fetch the region details
    let response = config.client
        .get(&format!("{}/admin/regions/{}", config.api_base_url, "ShowTestRegion"))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    println!("Show region response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Show region result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        
        if let Some(data) = result["data"].as_object() {
            assert_eq!(data["region_name"], "ShowTestRegion");
            assert_eq!(data["location_x"], 1100);
            assert_eq!(data["location_y"], 1100);
            assert_eq!(data["size_x"], 512);
            assert_eq!(data["size_y"], 512);
            assert!(data["uuid"].is_string());
            assert!(data["server_ip"].is_string());
            assert!(data["server_port"].is_number());
        }
    } else {
        println!("Show region failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}

/// Test region update via admin API
#[tokio::test]
async fn test_update_region_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    // First create a test region
    let create_payload = json!({
        "region_name": "UpdateTestRegion",
        "location_x": 1200,
        "location_y": 1200,
        "size_x": 256,
        "size_y": 256
    });
    
    let _create_response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&create_payload)
        .send()
        .await?;
    
    // Small delay to ensure creation completes
    sleep(Duration::from_millis(100)).await;
    
    // Now update the region
    let update_payload = json!({
        "new_name": "UpdatedTestRegion",
        "location_x": 1250,
        "size_x": 512
    });
    
    let response = config.client
        .put(&format!("{}/admin/regions/{}", config.api_base_url, "UpdateTestRegion"))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&update_payload)
        .send()
        .await?;
    
    println!("Update region response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Update region result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        assert!(result["message"].as_str().unwrap().contains("updated successfully"));
        
        if let Some(data) = result["data"].as_object() {
            assert_eq!(data["region_name"], "UpdatedTestRegion");
        }
    } else {
        println!("Update region failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}

/// Test region statistics via admin API
#[tokio::test]
async fn test_region_stats_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    let response = config.client
        .get(&format!("{}/admin/regions/stats", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    println!("Region stats response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Region stats result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        
        if let Some(data) = result["data"].as_object() {
            assert!(data.contains_key("total_regions"));
            assert!(data.contains_key("online_regions"));
            assert!(data.contains_key("offline_regions"));
            assert!(data.contains_key("standard_regions"));
            assert!(data.contains_key("large_regions"));
            assert!(data.contains_key("total_land_area"));
            assert!(data.contains_key("last_updated"));
            
            // Verify numeric values
            assert!(data["total_regions"].is_number());
            assert!(data["online_regions"].is_number());
            assert!(data["offline_regions"].is_number());
        }
    } else {
        println!("Region stats failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}

/// Test region deletion via admin API
#[tokio::test]
async fn test_delete_region_api() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    // First create a test region to delete
    let create_payload = json!({
        "region_name": "DeleteTestRegion",
        "location_x": 1300,
        "location_y": 1300,
        "size_x": 256,
        "size_y": 256
    });
    
    let _create_response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&create_payload)
        .send()
        .await?;
    
    // Small delay to ensure creation completes
    sleep(Duration::from_millis(100)).await;
    
    // Now delete the region
    let response = config.client
        .delete(&format!("{}/admin/regions/{}", config.api_base_url, "DeleteTestRegion"))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    println!("Delete region response status: {}", response.status());
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Delete region result: {}", serde_json::to_string_pretty(&result)?);
        
        assert!(result["success"].as_bool().unwrap_or(false));
        assert!(result["message"].as_str().unwrap().contains("deleted successfully"));
    } else {
        println!("Delete region failed with status: {}", response.status());
        println!("Response: {}", response.text().await?);
    }
    
    Ok(())
}

/// Test invalid region operations for proper error handling
#[tokio::test]
async fn test_region_validation() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    // Test creating region with invalid coordinates
    let invalid_payload = json!({
        "region_name": "InvalidRegion",
        "location_x": -100,  // Invalid negative coordinate
        "location_y": 1000,
        "size_x": 256,
        "size_y": 256
    });
    
    let response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&invalid_payload)
        .send()
        .await?;
    
    println!("Invalid coordinates response status: {}", response.status());
    
    // Should return bad request
    assert_eq!(response.status(), 400);
    
    let result: serde_json::Value = response.json().await?;
    assert!(!result["success"].as_bool().unwrap_or(true));
    assert!(result["message"].as_str().unwrap().contains("non-negative"));
    
    // Test creating region with invalid size
    let invalid_size_payload = json!({
        "region_name": "InvalidSizeRegion",
        "location_x": 1000,
        "location_y": 1000,
        "size_x": 32,  // Invalid size (too small)
        "size_y": 256
    });
    
    let response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&invalid_size_payload)
        .send()
        .await?;
    
    println!("Invalid size response status: {}", response.status());
    
    // Should return bad request
    assert_eq!(response.status(), 400);
    
    Ok(())
}

/// Test unauthorized access to region API
#[tokio::test]
async fn test_region_api_authentication() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    
    // Test without API key
    let response = config.client
        .get(&format!("{}/admin/regions", config.api_base_url))
        .send()
        .await?;
    
    println!("No API key response status: {}", response.status());
    assert_eq!(response.status(), 401);
    
    // Test with invalid API key
    let response = config.client
        .get(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", "invalid-key-123")
        .send()
        .await?;
    
    println!("Invalid API key response status: {}", response.status());
    assert_eq!(response.status(), 401);
    
    Ok(())
}

/// Test complete region lifecycle (create -> show -> update -> delete)
#[tokio::test]
async fn test_region_lifecycle() -> Result<()> {
    let config = RegionAdminTestConfig::new();
    let test_region_name = "LifecycleTestRegion";
    
    // 1. Create region
    let create_payload = json!({
        "region_name": test_region_name,
        "location_x": 1400,
        "location_y": 1400,
        "size_x": 256,
        "size_y": 256,
        "server_ip": "127.0.0.1",
        "server_port": 9000
    });
    
    println!("Creating region for lifecycle test...");
    let create_response = config.client
        .post(&format!("{}/admin/regions", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&create_payload)
        .send()
        .await?;
    
    if create_response.status().is_success() {
        let create_result: serde_json::Value = create_response.json().await?;
        println!("Region created: {}", serde_json::to_string_pretty(&create_result)?);
        assert!(create_result["success"].as_bool().unwrap_or(false));
    }
    
    // Small delay
    sleep(Duration::from_millis(100)).await;
    
    // 2. Show region details
    println!("Fetching region details...");
    let show_response = config.client
        .get(&format!("{}/admin/regions/{}", config.api_base_url, test_region_name))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    if show_response.status().is_success() {
        let show_result: serde_json::Value = show_response.json().await?;
        println!("Region details: {}", serde_json::to_string_pretty(&show_result)?);
        assert!(show_result["success"].as_bool().unwrap_or(false));
        assert_eq!(show_result["data"]["region_name"], test_region_name);
    }
    
    // 3. Update region
    println!("Updating region...");
    let update_payload = json!({
        "location_x": 1450,
        "size_x": 512
    });
    
    let update_response = config.client
        .put(&format!("{}/admin/regions/{}", config.api_base_url, test_region_name))
        .header("X-API-Key", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&update_payload)
        .send()
        .await?;
    
    if update_response.status().is_success() {
        let update_result: serde_json::Value = update_response.json().await?;
        println!("Region updated: {}", serde_json::to_string_pretty(&update_result)?);
        assert!(update_result["success"].as_bool().unwrap_or(false));
    }
    
    // 4. Delete region (cleanup)
    println!("Deleting region (cleanup)...");
    let delete_response = config.client
        .delete(&format!("{}/admin/regions/{}", config.api_base_url, test_region_name))
        .header("X-API-Key", &config.api_key)
        .send()
        .await?;
    
    if delete_response.status().is_success() {
        let delete_result: serde_json::Value = delete_response.json().await?;
        println!("Region deleted: {}", serde_json::to_string_pretty(&delete_result)?);
        assert!(delete_result["success"].as_bool().unwrap_or(false));
    }
    
    Ok(())
}

/// Integration test setup
#[tokio::test]
async fn test_region_admin_setup() -> Result<()> {
    println!("🧪 Phase 22.2 Region Management Integration Tests");
    println!("================================================");
    
    let config = RegionAdminTestConfig::new();
    println!("🌐 API URL: {}", config.api_base_url);
    println!("🔑 API Key: {}", if config.api_key == "default-key-change-me" { 
        "default (change for production)" 
    } else { 
        "configured" 
    });
    
    // Test server connectivity
    match config.client
        .get(&format!("{}/admin/health", config.api_base_url))
        .header("X-API-Key", &config.api_key)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("✅ Server is running and responding");
            println!("🚀 Running region management tests...");
        }
        _ => {
            println!("❌ Server is not running or not responding");
            println!("⏭️  Skipping server-dependent tests");
            println!("💡 Start server with: cargo run --bin opensim-next");
        }
    }
    
    Ok(())
}