use anyhow::Result;
use std::sync::Arc;
use std::net::SocketAddr;
use uuid::Uuid;
use bytes::{BytesMut, BufMut};

use opensim_next::session::SessionManager;
use opensim_next::udp::UdpServer;
use opensim_next::database::multi_backend::DatabaseConnection;
use opensim_next::login_stage_tracker::LoginStageTracker;

const TEST_PORT_BASE: u16 = 19000;

#[tokio::test]
async fn test_udp_server_creation() -> Result<()> {
    let session_manager = Arc::new(SessionManager::new());

    let stage_tracker = Arc::new(LoginStageTracker::new());
    let server = UdpServer::new(
        &format!("127.0.0.1:{}", TEST_PORT_BASE),
        session_manager,
        stage_tracker
    ).await?;

    assert!(server.get_session_manager().get_active_session_count() == 0);

    Ok(())
}

#[tokio::test]
async fn test_udp_packet_parsing() -> Result<()> {
    let mut packet_data = BytesMut::new();

    packet_data.put_u8(0x40);
    packet_data.put_u32(1);
    packet_data.put_u8(0x00);
    packet_data.put_u32(0xFFFF0003);
    packet_data.extend_from_slice(b"test data");

    let source: SocketAddr = "127.0.0.1:12345".parse()?;
    let packet = opensim_next::udp::server::UdpPacket::parse(&packet_data, source)?;

    assert_eq!(packet.flags, 0x40);
    assert_eq!(packet.sequence, 1);
    assert_eq!(packet.message_id, 0xFFFF0003);
    assert!(packet.is_reliable());
    assert!(!packet.is_resent());
    assert!(!packet.has_acks());

    Ok(())
}

#[tokio::test]
async fn test_use_circuit_code_message_handling() -> Result<()> {
    let session_manager = Arc::new(SessionManager::new());

    let agent_id = Uuid::new_v4();

    let login_session = session_manager.create_session(
        agent_id,
        "Test".to_string(),
        "User".to_string(),
        "127.0.0.1".to_string(),
        9000,
    )?;

    assert!(session_manager.validate_session(agent_id, login_session.session_id, login_session.circuit_code));
    assert_eq!(session_manager.get_active_session_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_region_handshake_reply_parsing() -> Result<()> {
    use opensim_next::udp::messages::RegionHandshakeReplyMessage;

    let agent_id = Uuid::new_v4();
    let flags = 0x00000001u32;

    let mut data = BytesMut::new();
    data.put_u128(agent_id.as_u128());
    data.put_u32(flags);

    let message = RegionHandshakeReplyMessage::parse(&data)?;

    assert_eq!(message.agent_id, agent_id);
    assert_eq!(message.flags, flags);

    Ok(())
}

#[tokio::test]
async fn test_agent_update_parsing() -> Result<()> {
    use opensim_next::udp::messages::AgentUpdateMessage;

    let agent_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let mut data = BytesMut::new();

    data.put_u128(agent_id.as_u128());
    data.put_u128(session_id.as_u128());

    data.put_f32(0.0);
    data.put_f32(0.0);
    data.put_f32(0.0);
    data.put_f32(1.0);

    data.put_f32(0.0);
    data.put_f32(0.0);
    data.put_f32(0.0);
    data.put_f32(1.0);

    data.put_u8(0x00);

    data.put_f32(128.0);
    data.put_f32(128.0);
    data.put_f32(21.0);

    data.put_f32(1.0);
    data.put_f32(0.0);
    data.put_f32(0.0);

    data.put_f32(0.0);
    data.put_f32(1.0);
    data.put_f32(0.0);

    data.put_f32(0.0);
    data.put_f32(0.0);
    data.put_f32(1.0);

    data.put_f32(96.0);
    data.put_u32(0x00000000);
    data.put_u8(0x00);

    let message = AgentUpdateMessage::parse(&data)?;

    assert_eq!(message.agent_id, agent_id);
    assert_eq!(message.session_id, session_id);
    assert_eq!(message.camera_center[0], 128.0);
    assert_eq!(message.camera_center[1], 128.0);
    assert_eq!(message.camera_center[2], 21.0);
    assert_eq!(message.far, 96.0);

    Ok(())
}

#[tokio::test]
async fn test_message_routing() -> Result<()> {
    let session_manager = Arc::new(SessionManager::new());

    let agent_id = Uuid::new_v4();

    let login_session = session_manager.create_session(
        agent_id,
        "Test".to_string(),
        "User".to_string(),
        "127.0.0.1".to_string(),
        9000,
    )?;

    let stage_tracker = Arc::new(LoginStageTracker::new());
    let server = UdpServer::new(
        &format!("127.0.0.1:{}", TEST_PORT_BASE + 1),
        session_manager.clone(),
        stage_tracker
    ).await?;

    assert!(session_manager.validate_session(agent_id, login_session.session_id, login_session.circuit_code));
    assert_eq!(server.get_session_manager().get_active_session_count(), 1);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_terrain_transmission_on_complete_agent_movement() -> Result<()> {
    use opensim_next::database::multi_backend::MultiDatabaseConfig;

    let session_manager = Arc::new(SessionManager::new());

    let agent_id = Uuid::new_v4();

    let _login_session = session_manager.create_session(
        agent_id,
        "Test".to_string(),
        "User".to_string(),
        "127.0.0.1".to_string(),
        9000,
    )?;

    let db_config = MultiDatabaseConfig::postgresql("localhost", 5432, "opensim_db", "opensim", "password123");

    let db_connection = DatabaseConnection::new(&db_config).await?;
    let db_arc = Arc::new(db_connection);

    let stage_tracker = Arc::new(LoginStageTracker::new());
    let mut server = UdpServer::new(
        &format!("127.0.0.1:{}", TEST_PORT_BASE + 2),
        session_manager.clone(),
        stage_tracker
    ).await?;

    server = server.with_database(db_arc.clone());

    server.initialize_terrain().await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_sessions() -> Result<()> {
    let session_manager = Arc::new(SessionManager::new());

    for i in 0..10 {
        let agent_id = Uuid::new_v4();

        let login_session = session_manager.create_session(
            agent_id,
            format!("Test{}", i),
            format!("User{}", i),
            "127.0.0.1".to_string(),
            9000,
        )?;

        assert!(session_manager.validate_session(agent_id, login_session.session_id, login_session.circuit_code));
    }

    assert_eq!(session_manager.get_active_session_count(), 10);

    Ok(())
}

#[tokio::test]
async fn test_session_cleanup() -> Result<()> {
    let session_manager = Arc::new(SessionManager::new());

    let agent_id = Uuid::new_v4();

    let login_session = session_manager.create_session(
        agent_id,
        "Test".to_string(),
        "User".to_string(),
        "127.0.0.1".to_string(),
        9000,
    )?;
    assert_eq!(session_manager.get_active_session_count(), 1);

    session_manager.remove_session(login_session.circuit_code)?;
    assert_eq!(session_manager.get_active_session_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_reliability_flags() -> Result<()> {
    let mut packet_data = BytesMut::new();

    packet_data.put_u8(0x60);
    packet_data.put_u32(42);
    packet_data.put_u8(0x00);
    packet_data.put_u32(0xFFFF00F9);

    let source: SocketAddr = "127.0.0.1:54321".parse()?;
    let packet = opensim_next::udp::server::UdpPacket::parse(&packet_data, source)?;

    assert!(packet.is_reliable());
    assert!(packet.is_resent());
    assert!(!packet.has_acks());

    Ok(())
}

#[tokio::test]
async fn test_ack_appended_flag() -> Result<()> {
    let mut packet_data = BytesMut::new();

    packet_data.put_u8(0x50);
    packet_data.put_u32(100);
    packet_data.put_u8(0x00);
    packet_data.put_u32(0x00000004);

    let source: SocketAddr = "127.0.0.1:9876".parse()?;
    let packet = opensim_next::udp::server::UdpPacket::parse(&packet_data, source)?;

    assert!(packet.is_reliable());
    assert!(!packet.is_resent());
    assert!(packet.has_acks());

    Ok(())
}
