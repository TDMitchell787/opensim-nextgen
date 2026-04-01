/// Phase 64.2 Integration Test: AvatarService with LoginService
/// Tests avatar appearance creation during login flow

use opensim_next::database::{DatabaseManager, multi_backend::{DatabaseConnection, MultiDatabaseConfig}};
use opensim_next::services::AvatarService;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_avatar_service_create_default_appearance() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Connect to PostgreSQL database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://opensim@localhost/opensim_db".to_string());

    let mut db_config = MultiDatabaseConfig::default();
    db_config.connection_string = db_url;

    let conn = DatabaseConnection::new(&db_config).await?;
    let conn = Arc::new(conn);

    // Create DatabaseManager
    let db_manager = Arc::new(DatabaseManager::with_connection(conn).await?);

    // Create AvatarService (matches main.rs:1298-1303)
    let avatar_service = Arc::new(AvatarService::new(db_manager.clone()));

    // Don McLean's UUID from POSTGRES_ACCESS.md
    let don_mclean_uuid = Uuid::parse_str("516b14df-ba9d-4d35-8b22-65419c8befdb")?;

    println!("[TEST] Testing avatar appearance creation for Don McLean");
    println!("[TEST] UUID: {}", don_mclean_uuid);

    // Call get_or_create_default_appearance (matches login_service.rs:172)
    let avatar = avatar_service.get_or_create_default_appearance(don_mclean_uuid).await?;

    println!("[TEST] ✅ Avatar appearance loaded/created successfully");
    println!("[TEST] AvatarType: {}", avatar.avatar_type);
    println!("[TEST] Data keys: {}", avatar.data.len());

    // Verify expected fields
    assert_eq!(avatar.avatar_type, 1, "Avatar type should be 1 (SL avatar)");

    assert!(avatar.data.contains_key("Serial"), "Should have Serial");
    assert_eq!(avatar.data.get("Serial"), Some(&"1".to_string()));

    assert!(avatar.data.contains_key("AvatarHeight"), "Should have AvatarHeight");
    assert_eq!(avatar.data.get("AvatarHeight"), Some(&"1.771488".to_string()));

    assert!(avatar.data.contains_key("BodyItem"), "Should have BodyItem");
    assert!(avatar.data.contains_key("BodyAsset"), "Should have BodyAsset");
    assert!(avatar.data.contains_key("SkinItem"), "Should have SkinItem");
    assert!(avatar.data.contains_key("SkinAsset"), "Should have SkinAsset");
    assert!(avatar.data.contains_key("HairItem"), "Should have HairItem");
    assert!(avatar.data.contains_key("HairAsset"), "Should have HairAsset");
    assert!(avatar.data.contains_key("EyesItem"), "Should have EyesItem");
    assert!(avatar.data.contains_key("EyesAsset"), "Should have EyesAsset");

    assert!(avatar.data.contains_key("VisualParams"), "Should have VisualParams");
    let visual_params = avatar.data.get("VisualParams").unwrap();
    let param_count = visual_params.split(',').count();
    assert_eq!(param_count, 218, "Should have 218 visual parameters");

    // Verify wearables format
    assert!(avatar.data.contains_key("Wearable 0:0"), "Should have Wearable 0:0 (Shape)");
    assert!(avatar.data.contains_key("Wearable 1:0"), "Should have Wearable 1:0 (Skin)");
    assert!(avatar.data.contains_key("Wearable 2:0"), "Should have Wearable 2:0 (Hair)");
    assert!(avatar.data.contains_key("Wearable 3:0"), "Should have Wearable 3:0 (Eyes)");

    println!("[TEST] ✅ All avatar data fields verified");

    // Verify data persisted to database
    println!("[TEST] Verifying database persistence...");
    let reloaded_avatar = avatar_service.get_avatar(don_mclean_uuid).await?;

    assert_eq!(reloaded_avatar.avatar_type, 1);
    assert_eq!(reloaded_avatar.data.len(), avatar.data.len());
    assert_eq!(reloaded_avatar.data.get("Serial"), Some(&"1".to_string()));
    assert_eq!(reloaded_avatar.data.get("AvatarHeight"), Some(&"1.771488".to_string()));

    println!("[TEST] ✅ Avatar data successfully persisted to PostgreSQL");
    println!("[TEST] ✅ Phase 64.2 Integration Test PASSED");

    Ok(())
}
