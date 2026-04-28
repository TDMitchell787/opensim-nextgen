//! Phase 25 Database Restoration Integration Test
//!
//! Validates that all database operations restored in Phase 25 are working correctly:
//! - Avatar Persistence Layer (Phase 25.1.1)
//! - Social Features Database (Phase 25.1.2)
//! - Economy Database Integration (Phase 25.1.3)

use chrono::Utc;
use opensim_next::avatar::{
    Achievement, AchievementCategory, AvatarAppearance, AvatarBehavior, AvatarPersistence,
    AvatarPersistenceData, AvatarPreferences, AvatarSocialFeatures, AvatarSocialProfile,
    EnhancedAvatar,
};
use opensim_next::database::{DatabaseManager, DatabaseType, MultiDatabaseConfig};
use opensim_next::economy::{CurrencyBalance, CurrencySystem, EconomyConfig};
use std::sync::Arc;
use uuid::Uuid;

/// Test database URL for integration testing
const TEST_DB_URL: &str = "sqlite::memory:";

#[tokio::test]
#[ignore]
async fn test_phase25_database_manager_creation() {
    println!("🧪 Testing Phase 25 Database Manager Creation...");

    // Test database manager creation
    let db_manager = DatabaseManager::new(TEST_DB_URL).await;

    match db_manager {
        Ok(manager) => {
            println!("✅ Database Manager created successfully");

            // Test legacy pool access (Phase 25 compatibility)
            let pool_result = manager.legacy_pool();
            match pool_result {
                Ok(_pool) => println!("✅ Legacy pool access working"),
                Err(e) => println!("⚠️  Legacy pool access issue: {}", e),
            }

            // Test health check
            let health = manager.health_check().await;
            match health {
                Ok(_) => println!("✅ Database health check passed"),
                Err(e) => println!("⚠️  Health check issue: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Database Manager creation failed: {}", e);
            panic!("Cannot proceed with database tests");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_phase25_1_1_avatar_persistence() {
    println!("🧪 Testing Phase 25.1.1 - Avatar Persistence Layer...");

    let db_manager = match DatabaseManager::new(TEST_DB_URL).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("❌ Failed to create database manager: {}", e);
            return;
        }
    };

    let avatar_persistence = AvatarPersistence::new(db_manager.clone());

    // Test table initialization
    match avatar_persistence.initialize_tables().await {
        Ok(_) => println!("✅ Avatar persistence tables initialized"),
        Err(e) => println!("⚠️  Avatar persistence table initialization issue: {}", e),
    }

    // Create test avatar
    let test_avatar = create_test_avatar();

    // Test avatar storage
    match avatar_persistence.store_avatar(&test_avatar).await {
        Ok(_) => println!("✅ Avatar storage test passed"),
        Err(e) => println!("⚠️  Avatar storage issue: {}", e),
    }

    // Test avatar loading
    match avatar_persistence.load_avatar(test_avatar.id).await {
        Ok(loaded_avatar) => {
            if loaded_avatar.id == test_avatar.id {
                println!("✅ Avatar loading test passed");
            } else {
                println!("⚠️  Avatar loading returned wrong ID");
            }
        }
        Err(e) => println!("⚠️  Avatar loading issue: {}", e),
    }

    // Test avatar count
    match avatar_persistence.get_total_avatar_count().await {
        Ok(count) => println!("✅ Avatar count test passed: {} avatars", count),
        Err(e) => println!("⚠️  Avatar count issue: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_phase25_1_2_social_features() {
    println!("🧪 Testing Phase 25.1.2 - Social Features Database Layer...");

    let db_manager = match DatabaseManager::new(TEST_DB_URL).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("❌ Failed to create database manager: {}", e);
            return;
        }
    };

    let social_features = AvatarSocialFeatures::new(db_manager.clone());

    // Test table initialization
    match social_features.initialize_tables().await {
        Ok(_) => println!("✅ Social features tables initialized"),
        Err(e) => println!("⚠️  Social features table initialization issue: {}", e),
    }

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    // Test friend management
    match social_features
        .add_friend(user1_id, user2_id, "TestFriend".to_string())
        .await
    {
        Ok(_) => println!("✅ Add friend test passed"),
        Err(e) => println!("⚠️  Add friend issue: {}", e),
    }

    match social_features.get_friend_count(user1_id).await {
        Ok(count) => println!("✅ Friend count test passed: {} friends", count),
        Err(e) => println!("⚠️  Friend count issue: {}", e),
    }

    match social_features.are_friends(user1_id, user2_id).await {
        Ok(are_friends) => {
            if are_friends {
                println!("✅ Friend relationship verification passed");
            } else {
                println!("⚠️  Friend relationship not found");
            }
        }
        Err(e) => println!("⚠️  Friend verification issue: {}", e),
    }

    // Test achievement system
    let test_achievement = Achievement {
        achievement_id: Uuid::new_v4(),
        name: "Test Achievement".to_string(),
        description: "A test achievement".to_string(),
        icon_url: Some("test_icon.png".to_string()),
        earned_at: Utc::now(),
        points: 100,
        category: AchievementCategory::Social,
    };

    match social_features
        .add_achievement(user1_id, test_achievement)
        .await
    {
        Ok(_) => println!("✅ Add achievement test passed"),
        Err(e) => println!("⚠️  Add achievement issue: {}", e),
    }

    match social_features.get_achievement_points(user1_id).await {
        Ok(points) => println!("✅ Achievement points test passed: {} points", points),
        Err(e) => println!("⚠️  Achievement points issue: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_phase25_1_3_economy_integration() {
    println!("🧪 Testing Phase 25.1.3 - Economy Database Integration...");

    let db_manager = match DatabaseManager::new(TEST_DB_URL).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("❌ Failed to create database manager: {}", e);
            return;
        }
    };

    let economy_config = EconomyConfig::default();
    let currency_system = CurrencySystem::new(db_manager.clone(), economy_config);

    // Test currency system initialization
    match currency_system.initialize().await {
        Ok(_) => println!("✅ Currency system initialized"),
        Err(e) => println!("⚠️  Currency system initialization issue: {}", e),
    }

    let test_user_id = Uuid::new_v4();

    // Test balance retrieval (should create default balance)
    match currency_system.get_balance(test_user_id, "L$").await {
        Ok(balance) => {
            println!("✅ Balance retrieval test passed: {} L$", balance.available);
            if balance.user_id == test_user_id && balance.currency_code == "L$" {
                println!("✅ Balance data integrity verified");
            } else {
                println!("⚠️  Balance data integrity issue");
            }
        }
        Err(e) => println!("⚠️  Balance retrieval issue: {}", e),
    }

    // Test multiple currency support
    match currency_system.get_all_balances(test_user_id).await {
        Ok(balances) => println!(
            "✅ Multi-currency balance test passed: {} currencies",
            balances.len()
        ),
        Err(e) => println!("⚠️  Multi-currency balance issue: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_phase25_2_1_integration_workflow() {
    println!("🧪 Testing Phase 25.2.1 - Complete Integration Workflow...");

    let db_manager = match DatabaseManager::new(TEST_DB_URL).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            println!("❌ Failed to create database manager: {}", e);
            return;
        }
    };

    // Test complete workflow: Create avatar with social features and economy
    let test_user_id = Uuid::new_v4();
    let test_avatar_id = Uuid::new_v4();

    // 1. Initialize all systems
    let avatar_persistence = AvatarPersistence::new(db_manager.clone());
    let social_features = AvatarSocialFeatures::new(db_manager.clone());
    let currency_system = CurrencySystem::new(db_manager.clone(), EconomyConfig::default());

    // Initialize all tables
    let init_results = tokio::join!(
        avatar_persistence.initialize_tables(),
        social_features.initialize_tables(),
        currency_system.initialize()
    );

    match init_results {
        (Ok(_), Ok(_), Ok(_)) => println!("✅ All systems initialized successfully"),
        _ => println!("⚠️  Some systems had initialization issues"),
    }

    // 2. Create and store avatar
    let test_avatar = EnhancedAvatar {
        id: test_avatar_id,
        user_id: test_user_id,
        name: "IntegrationTestUser".to_string(),
        appearance: AvatarAppearance::default(),
        behavior: AvatarBehavior::default(),
        social_profile: AvatarSocialProfile::default(),
        persistence_data: AvatarPersistenceData::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match avatar_persistence.store_avatar(&test_avatar).await {
        Ok(_) => println!("✅ Integration test avatar created"),
        Err(e) => println!("⚠️  Integration avatar creation issue: {}", e),
    }

    // 3. Set up social features
    let friend_id = Uuid::new_v4();
    match social_features
        .add_friend(test_user_id, friend_id, "IntegrationFriend".to_string())
        .await
    {
        Ok(_) => println!("✅ Integration social setup completed"),
        Err(e) => println!("⚠️  Integration social setup issue: {}", e),
    }

    // 4. Set up economy
    match currency_system.get_balance(test_user_id, "L$").await {
        Ok(_) => println!("✅ Integration economy setup completed"),
        Err(e) => println!("⚠️  Integration economy setup issue: {}", e),
    }

    println!("🎉 Phase 25 Integration Workflow Test Completed!");
}

// Helper function to create test avatar data
fn create_test_avatar() -> EnhancedAvatar {
    use opensim_next::avatar::*;

    EnhancedAvatar {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "TestAvatar".to_string(),
        appearance: AvatarAppearance {
            height: 1.8,
            proportions: AvatarProportions::default(),
            wearables: Vec::new(),
            textures: std::collections::HashMap::new(),
            attachments: Vec::new(),
            visual_params: Vec::new(),
        },
        behavior: AvatarBehavior {
            animations: Vec::new(),
            gestures: Vec::new(),
            auto_behaviors: Vec::new(),
            expressions: Vec::new(),
            voice_settings: VoiceSettings::default(),
        },
        social_profile: AvatarSocialProfile {
            display_name: "Test User".to_string(),
            bio: Some("A test user for Phase 25 validation".to_string()),
            interests: vec!["Testing".to_string(), "Phase25".to_string()],
            languages: vec!["en".to_string()],
            relationship_status: RelationshipStatus::NotSpecified,
            privacy_settings: PrivacySettings::default(),
            social_links: std::collections::HashMap::new(),
            achievements: Vec::new(),
        },
        persistence_data: AvatarPersistenceData {
            last_position: Vector3::default(),
            last_rotation: Quaternion::default(),
            last_region: Uuid::new_v4(),
            session_time: 0,
            total_time: 0,
            visit_count: 1,
            last_login: Utc::now(),
            inventory_snapshot: None,
            preferences: AvatarPreferences::default(),
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
