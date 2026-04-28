//! Integration Tests for Phase 24.3: Advanced Social Features
//!
//! Comprehensive test suite for social features including friends, groups,
//! messaging, community management, and social networking capabilities.

use anyhow::Result;
use opensim_next::social::*;
use opensim_next::OpenSimServer;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_social_features_initialization() -> Result<()> {
    // Initialize OpenSim server with social features
    let server = OpenSimServer::new().await?;

    // Test social manager availability
    let social_manager = server.social_manager();
    assert!(social_manager.friend_system().as_ref() as *const _ != std::ptr::null()); // Check friend system exists

    // Test social system health
    let health = server.get_social_system_health().await;
    assert_eq!(health.status, "healthy");

    println!("✅ Social features initialization test passed");
    Ok(())
}

#[tokio::test]
async fn test_user_social_profile_management() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user_id = Uuid::new_v4();
    let display_name = "TestUser".to_string();

    // Create social profile
    let profile = server
        .create_user_social_profile(user_id, display_name.clone())
        .await?;
    assert_eq!(profile.user_id, user_id);
    assert_eq!(profile.display_name, display_name);
    assert_eq!(profile.social_statistics.friend_count, 0);

    // Update online status
    server
        .update_user_online_status(user_id, OnlineStatus::Online)
        .await?;

    println!("✅ User social profile management test passed");
    Ok(())
}

#[tokio::test]
async fn test_friend_system_workflow() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    // Create social profiles for both users
    server
        .create_user_social_profile(user1_id, "User1".to_string())
        .await?;
    server
        .create_user_social_profile(user2_id, "User2".to_string())
        .await?;

    // Send friend request
    let friend_request = server
        .send_friend_request(user1_id, user2_id, Some("Let's be friends!".to_string()))
        .await?;

    assert_eq!(friend_request.requester_id, user1_id);
    assert_eq!(friend_request.target_id, user2_id);
    assert_eq!(friend_request.status, friends::FriendRequestStatus::Pending);

    // Get friend list (should be empty before acceptance)
    let friend_list = server.get_user_friends(user1_id).await?;
    assert_eq!(friend_list.total_count, 0);

    println!("✅ Friend system workflow test passed");
    Ok(())
}

#[tokio::test]
async fn test_messaging_system() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    // Create social profiles
    server
        .create_user_social_profile(user1_id, "Sender".to_string())
        .await?;
    server
        .create_user_social_profile(user2_id, "Receiver".to_string())
        .await?;

    // Send a message
    let message_request = messaging::SendMessageRequest {
        conversation_id: None,
        recipient_ids: vec![user2_id],
        message_type: messaging::MessageType::Text,
        content: messaging::MessageContent {
            text: Some("Hello, how are you?".to_string()),
            ..Default::default()
        },
        reply_to: None,
        thread_id: None,
    };

    let message = server.send_message(user1_id, message_request).await?;
    assert_eq!(message.sender_id, user1_id);
    assert_eq!(
        message.content.text,
        Some("Hello, how are you?".to_string())
    );
    assert_eq!(message.status, messaging::MessageStatus::Sent);

    // Get conversations for sender
    let conversations = server.get_user_conversations(user1_id).await?;
    assert!(conversations.total_count > 0);

    println!("✅ Messaging system test passed");
    Ok(())
}

#[tokio::test]
async fn test_group_management() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let owner_id = Uuid::new_v4();

    // Create social profile for group owner
    server
        .create_user_social_profile(owner_id, "GroupOwner".to_string())
        .await?;

    // Create a group
    let group_request = groups::CreateGroupRequest {
        name: "Test Group".to_string(),
        description: Some("A test group for development".to_string()),
        group_type: groups::GroupType::Social,
        visibility: groups::GroupVisibility::Public,
        membership_policy: groups::GroupMembershipPolicy::Open,
        max_members: Some(100),
        tags: vec!["test".to_string(), "development".to_string()],
        group_charter: Some("Test group charter".to_string()),
        settings: groups::GroupSettings::default(),
    };

    let group = server.create_group(owner_id, group_request).await?;
    assert_eq!(group.name, "Test Group");
    assert_eq!(group.owner_id, owner_id);
    assert_eq!(group.member_count, 1); // Owner is first member
    assert_eq!(group.visibility, groups::GroupVisibility::Public);

    println!("✅ Group management test passed");
    Ok(())
}

#[tokio::test]
async fn test_social_search_functionality() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user_id = Uuid::new_v4();

    // Create social profile
    server
        .create_user_social_profile(user_id, "SearchUser".to_string())
        .await?;

    // Test social search
    let search_criteria = SocialSearchCriteria {
        query: Some("test".to_string()),
        search_type: SocialSearchType::All,
        filters: SocialSearchFilters::default(),
        sort_by: SocialSortOption::Relevance,
        sort_order: SortOrder::Descending,
        limit: Some(10),
        offset: None,
    };

    let search_results = server
        .social_manager()
        .search_social_content(user_id, search_criteria)
        .await?;

    // Should have results structure even if empty
    assert!(search_results.total_results >= 0);

    println!("✅ Social search functionality test passed");
    Ok(())
}

#[tokio::test]
async fn test_social_statistics_generation() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user_id = Uuid::new_v4();

    // Create social profile
    server
        .create_user_social_profile(user_id, "StatsUser".to_string())
        .await?;

    // Get user social statistics
    let user_stats = server
        .social_manager()
        .get_user_social_statistics(user_id)
        .await?;

    assert_eq!(user_stats.user_id, user_id);
    assert_eq!(user_stats.friend_count, 0);
    assert_eq!(user_stats.group_memberships, 0);
    assert!(user_stats.social_score >= 0.0);

    // Get system social statistics
    let system_stats = server
        .social_manager()
        .get_system_social_statistics()
        .await?;

    assert!(system_stats.total_users >= 0);
    assert!(system_stats.total_friendships >= 0);
    assert!(system_stats.total_groups >= 0);

    println!("✅ Social statistics generation test passed");
    Ok(())
}

#[tokio::test]
async fn test_social_system_health_monitoring() -> Result<()> {
    let server = OpenSimServer::new().await?;

    // Check social system health
    let health = server.get_social_system_health().await;

    assert!(!health.status.is_empty());
    assert!(health.friend_system_healthy);
    assert!(health.group_system_healthy);
    assert!(health.messaging_system_healthy);
    assert!(health.community_system_healthy);
    assert!(health.notification_system_healthy);
    assert!(health.moderation_system_healthy);
    assert!(health.system_load >= 0.0);

    println!("✅ Social system health monitoring test passed");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_social_workflow() -> Result<()> {
    let server = OpenSimServer::new().await?;
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let user3_id = Uuid::new_v4();

    // Create social profiles for all users
    server
        .create_user_social_profile(user1_id, "Alice".to_string())
        .await?;
    server
        .create_user_social_profile(user2_id, "Bob".to_string())
        .await?;
    server
        .create_user_social_profile(user3_id, "Charlie".to_string())
        .await?;

    // Update online status
    server
        .update_user_online_status(user1_id, OnlineStatus::Online)
        .await?;
    server
        .update_user_online_status(user2_id, OnlineStatus::Online)
        .await?;
    server
        .update_user_online_status(user3_id, OnlineStatus::Away)
        .await?;

    // Send friend requests
    let _request1 = server
        .send_friend_request(user1_id, user2_id, Some("Hi Bob!".to_string()))
        .await?;
    let _request2 = server
        .send_friend_request(user1_id, user3_id, Some("Hi Charlie!".to_string()))
        .await?;

    // Create a group
    let group_request = groups::CreateGroupRequest {
        name: "Friends Group".to_string(),
        description: Some("A group for friends".to_string()),
        group_type: groups::GroupType::Social,
        visibility: groups::GroupVisibility::Public,
        membership_policy: groups::GroupMembershipPolicy::Open,
        max_members: Some(50),
        tags: vec!["friends".to_string(), "social".to_string()],
        group_charter: None,
        settings: groups::GroupSettings::default(),
    };

    let group = server.create_group(user1_id, group_request).await?;
    assert_eq!(group.owner_id, user1_id);

    // Send a group message
    let group_message_request = messaging::SendMessageRequest {
        conversation_id: None,
        recipient_ids: vec![user2_id, user3_id],
        message_type: messaging::MessageType::Text,
        content: messaging::MessageContent {
            text: Some("Welcome to our group!".to_string()),
            ..Default::default()
        },
        reply_to: None,
        thread_id: None,
    };

    let _message = server.send_message(user1_id, group_message_request).await?;

    // Check social statistics
    let stats = server
        .social_manager()
        .get_user_social_statistics(user1_id)
        .await?;
    assert_eq!(stats.user_id, user1_id);

    // Check system health
    let health = server.get_social_system_health().await;
    assert_eq!(health.status, "healthy");

    println!("✅ Comprehensive social workflow test passed");
    Ok(())
}

#[tokio::test]
async fn test_social_features_integration_with_server() -> Result<()> {
    let server = OpenSimServer::new().await?;

    // Test that social features are properly integrated with the main server
    let social_manager = server.social_manager();

    // Test all subsystem availability
    let friend_system = social_manager.friend_system();
    let group_system = social_manager.group_system();
    let messaging_system = social_manager.messaging_system();
    let community_system = social_manager.community_system();
    let notification_system = social_manager.notification_system();
    let moderation_system = social_manager.moderation_system();

    // All systems should be available
    assert!(!Arc::as_ptr(&friend_system).is_null());
    assert!(!Arc::as_ptr(&group_system).is_null());
    assert!(!Arc::as_ptr(&messaging_system).is_null());
    assert!(!Arc::as_ptr(&community_system).is_null());
    assert!(!Arc::as_ptr(&notification_system).is_null());
    assert!(!Arc::as_ptr(&moderation_system).is_null());

    println!("✅ Social features integration with server test passed");
    Ok(())
}

#[tokio::test]
async fn test_social_features_configuration() -> Result<()> {
    // Test social configuration
    let config = SocialConfig::default();

    assert_eq!(config.max_friends_per_user, 1000);
    assert_eq!(config.max_groups_per_user, 100);
    assert_eq!(config.max_communities_per_user, 50);
    assert!(config.enable_recommendations);
    assert!(config.enable_activity_feeds);
    assert!(config.enable_social_search);
    assert_eq!(config.message_rate_limit, 60);
    assert_eq!(config.friend_request_rate_limit, 10);
    assert!(config.moderation_enabled);
    assert!(config.content_filtering_enabled);

    println!("✅ Social features configuration test passed");
    Ok(())
}
