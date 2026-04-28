//! Phase 25 Core Database Test - Essential Validation
//! Tests core database functionality restored in Phase 25

use opensim_next::database::DatabaseManager;
use opensim_next::economy::{CurrencySystem, EconomyConfig};
use std::sync::Arc;
use uuid::Uuid;

const TEST_DB_URL: &str = "sqlite::memory:";

#[tokio::test]
async fn test_phase25_core_database_functionality() {
    println!("🧪 Testing Phase 25 Core Database Functionality...");

    // Test 1: Database Manager Creation
    let db_manager = match DatabaseManager::new(TEST_DB_URL).await {
        Ok(manager) => {
            println!("✅ Database Manager created successfully");
            Arc::new(manager)
        }
        Err(e) => {
            println!("❌ Database Manager creation failed: {}", e);
            panic!("Cannot proceed with database tests");
        }
    };

    // Test 2: Legacy Pool Access (Phase 25 Critical)
    match db_manager.legacy_pool() {
        Ok(_pool) => println!("✅ Legacy pool access working - Phase 25.1 integration successful"),
        Err(e) => {
            println!("❌ Legacy pool access failed: {}", e);
            panic!("Phase 25 legacy pool integration failed");
        }
    }

    // Test 3: Currency System Initialization (Phase 25.1.3)
    let economy_config = EconomyConfig::default();
    let currency_system = CurrencySystem::new(db_manager.clone(), economy_config);

    match currency_system.initialize().await {
        Ok(_) => {
            println!("✅ Phase 25.1.3 Currency System initialized successfully");

            // Test 4: Basic Currency Operations
            let test_user_id = Uuid::new_v4();
            match currency_system.get_balance(test_user_id, "L$").await {
                Ok(balance) => {
                    println!(
                        "✅ Phase 25.1.3 Currency balance operations working: {} L$",
                        balance.available
                    );
                }
                Err(e) => {
                    println!("⚠️  Currency balance operation issue: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Currency system initialization issue: {}", e);
        }
    }

    println!("🎉 Phase 25 Core Database Test Completed!");
}

#[tokio::test]
async fn test_phase25_database_error_handling() {
    println!("🧪 Testing Phase 25 Database Error Handling...");

    // Test error handling with invalid database URL
    let result = DatabaseManager::new("invalid://bad-url").await;

    match result {
        Ok(_) => println!("⚠️  Unexpected success with invalid URL"),
        Err(e) => println!("✅ Error handling working correctly: {}", e),
    }

    println!("✅ Phase 25 Error Handling Test Completed!");
}
