use std::time::Duration;
use tokio::time::sleep;
use opensim_next::login_session::{CircuitCodeRegistry, LoginSession};
use opensim_next::opensim_compatibility::protocols::LLUDPClientStack;

#[tokio::test]
async fn test_circuit_code_registration_and_validation() {
    println!("Testing circuit code registration and validation...");

    let registry = CircuitCodeRegistry::new();

    let circuit_code = 12345_u32;
    let session = LoginSession {
        circuit_code,
        session_id: "test-session-123".to_string(),
        agent_id: "test-agent-456".to_string(),
        first_name: "test".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now(),
        is_xmlrpc_session: false,
    };

    println!("Registering circuit code {} for user {} {}",
        circuit_code, session.first_name, session.last_name);

    registry.register_login(session.clone()).await;

    println!("Validating circuit code {}...", circuit_code);
    let validated_session = registry.validate_circuit_code(circuit_code).await;

    assert!(validated_session.is_some(), "Circuit code validation should succeed");
    let validated = validated_session.unwrap();

    assert_eq!(validated.circuit_code, circuit_code);
    assert_eq!(validated.session_id, "test-session-123");
    assert_eq!(validated.agent_id, "test-agent-456");
    assert_eq!(validated.first_name, "test");
    assert_eq!(validated.last_name, "user");

    println!("✓ Circuit code validation successful!");

    println!("Testing circuit code removal after use...");
    let removed_session = registry.remove_circuit_code(circuit_code).await;
    assert!(removed_session.is_some(), "Circuit code should be removed");

    let invalid_validation = registry.validate_circuit_code(circuit_code).await;
    assert!(invalid_validation.is_none(), "Circuit code should no longer be valid after removal");

    println!("✓ Circuit code removal successful!");
}

#[tokio::test]
async fn test_circuit_code_expiration() {
    println!("Testing circuit code expiration...");

    let registry = CircuitCodeRegistry::new();

    let circuit_code = 99999_u32;
    let expired_session = LoginSession {
        circuit_code,
        session_id: "expired-session".to_string(),
        agent_id: "expired-agent".to_string(),
        first_name: "expired".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now() - Duration::from_secs(400),
        is_xmlrpc_session: false,
    };

    registry.register_login(expired_session).await;

    let validation_result = registry.validate_circuit_code(circuit_code).await;
    assert!(validation_result.is_none(), "Expired circuit code should not validate");

    println!("✓ Circuit code expiration test successful!");
}

#[tokio::test]
async fn test_invalid_circuit_code() {
    println!("Testing invalid circuit code validation...");

    let registry = CircuitCodeRegistry::new();

    let invalid_code = 88888_u32;
    let validation_result = registry.validate_circuit_code(invalid_code).await;

    assert!(validation_result.is_none(), "Invalid circuit code should not validate");

    println!("✓ Invalid circuit code test successful!");
}

#[tokio::test]
async fn test_multiple_concurrent_logins() {
    println!("Testing multiple concurrent login sessions...");

    let registry = CircuitCodeRegistry::new();

    let sessions = vec![
        LoginSession {
            circuit_code: 11111,
            session_id: "session-1".to_string(),
            agent_id: "agent-1".to_string(),
            first_name: "user1".to_string(),
            last_name: "test".to_string(),
            created_at: std::time::Instant::now(),
            is_xmlrpc_session: false,
        },
        LoginSession {
            circuit_code: 22222,
            session_id: "session-2".to_string(),
            agent_id: "agent-2".to_string(),
            first_name: "user2".to_string(),
            last_name: "test".to_string(),
            created_at: std::time::Instant::now(),
            is_xmlrpc_session: false,
        },
        LoginSession {
            circuit_code: 33333,
            session_id: "session-3".to_string(),
            agent_id: "agent-3".to_string(),
            first_name: "user3".to_string(),
            last_name: "test".to_string(),
            created_at: std::time::Instant::now(),
            is_xmlrpc_session: false,
        },
    ];

    for session in &sessions {
        registry.register_login(session.clone()).await;
        println!("Registered circuit code {} for user {}",
            session.circuit_code, session.first_name);
    }

    for session in &sessions {
        let validated = registry.validate_circuit_code(session.circuit_code).await;
        assert!(validated.is_some(), "All registered circuit codes should validate");
        let validated_session = validated.unwrap();
        assert_eq!(validated_session.first_name, session.first_name);
        println!("✓ Validated circuit code {} for user {}",
            session.circuit_code, session.first_name);
    }

    println!("✓ Multiple concurrent login test successful!");
}

#[test]
fn test_unique_circuit_code_generation() {
    println!("Testing unique circuit code generation...");

    let mut generated_codes = std::collections::HashSet::new();
    let mut duplicates = 0;

    for i in 0..1000 {
        let circuit_code = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32) ^ (rand::random::<u32>() & 0xFFFF) ^ (i as u32);

        if !generated_codes.insert(circuit_code) {
            duplicates += 1;
        }

        std::thread::sleep(std::time::Duration::from_nanos(1));
    }

    println!("✓ Generated {} unique circuit codes with {} duplicates",
        generated_codes.len(), duplicates);

    assert!(duplicates < 50, "Too many duplicate circuit codes: {}", duplicates);
    assert!(generated_codes.len() > 950, "Not enough unique codes generated: {}", generated_codes.len());

    println!("✓ Circuit code generation test successful!");
}

#[tokio::test]
async fn test_full_login_flow_simulation() {
    println!("Testing full login flow simulation...");

    let circuit_registry = CircuitCodeRegistry::new();

    let circuit_code = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32) ^ (rand::random::<u32>() & 0xFFFF);

    let login_session = LoginSession {
        circuit_code,
        session_id: "test-session-456".to_string(),
        agent_id: "test-agent-789".to_string(),
        first_name: "firestorm".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now(),
        is_xmlrpc_session: false,
    };

    println!("Step 1: XML-RPC Login - Generated circuit code {} for user {} {}",
        circuit_code, login_session.first_name, login_session.last_name);

    circuit_registry.register_login(login_session.clone()).await;
    println!("Step 2: Login service registered circuit code with LLUDP server");

    println!("Step 3: Viewer received login response with circuit code {}", circuit_code);

    println!("Step 4: Viewer connecting to LLUDP server with UseCircuitCode...");

    let validation_result = circuit_registry.validate_circuit_code(circuit_code).await;
    assert!(validation_result.is_some(), "LLUDP server should validate circuit code successfully");

    let validated_session = validation_result.unwrap();
    assert_eq!(validated_session.circuit_code, circuit_code);
    assert_eq!(validated_session.first_name, "firestorm");
    assert_eq!(validated_session.last_name, "user");

    println!("Step 5: ✓ LLUDP server validated circuit code {} for user {} {}",
        circuit_code, validated_session.first_name, validated_session.last_name);

    let removed_session = circuit_registry.remove_circuit_code(circuit_code).await;
    assert!(removed_session.is_some(), "Circuit code should be removed after authentication");

    println!("Step 6: ✓ Circuit code removed after successful authentication");

    let second_validation = circuit_registry.validate_circuit_code(circuit_code).await;
    assert!(second_validation.is_none(), "Circuit code should not validate after being used");

    println!("Step 7: ✓ Circuit code cannot be reused");

    println!("🎉 Full login flow simulation successful!");
    println!("✓ This demonstrates the complete OpenSim login process:");
    println!("  1. XML-RPC login generates unique circuit code");
    println!("  2. Login service shares circuit code with LLUDP server");
    println!("  3. Viewer receives circuit code in login response");
    println!("  4. Viewer sends UseCircuitCode UDP message to simulator");
    println!("  5. LLUDP server validates circuit code and authenticates connection");
    println!("  6. Circuit code is consumed (one-time use for security)");
}

#[tokio::test]
async fn test_realistic_timing_scenarios() {
    println!("Testing realistic timing scenarios...");

    let registry = CircuitCodeRegistry::new();

    let circuit_code_1 = 11111_u32;
    let session_1 = LoginSession {
        circuit_code: circuit_code_1,
        session_id: "timing-test-1".to_string(),
        agent_id: "agent-1".to_string(),
        first_name: "quick".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now(),
        is_xmlrpc_session: false,
    };

    registry.register_login(session_1).await;

    let validation_1 = registry.validate_circuit_code(circuit_code_1).await;
    assert!(validation_1.is_some(), "Immediate circuit code use should succeed");
    registry.remove_circuit_code(circuit_code_1).await;

    println!("✓ Test 1: Immediate circuit code use - SUCCESS");

    let circuit_code_2 = 22222_u32;
    let session_2 = LoginSession {
        circuit_code: circuit_code_2,
        session_id: "timing-test-2".to_string(),
        agent_id: "agent-2".to_string(),
        first_name: "slow".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now(),
        is_xmlrpc_session: false,
    };

    registry.register_login(session_2).await;

    sleep(Duration::from_secs(30)).await;

    let validation_2 = registry.validate_circuit_code(circuit_code_2).await;
    assert!(validation_2.is_some(), "Circuit code should still be valid after 30 seconds");
    registry.remove_circuit_code(circuit_code_2).await;

    println!("✓ Test 2: 30-second delayed circuit code use - SUCCESS");

    let circuit_code_3 = 33333_u32;
    let session_3 = LoginSession {
        circuit_code: circuit_code_3,
        session_id: "timing-test-3".to_string(),
        agent_id: "agent-3".to_string(),
        first_name: "veryslow".to_string(),
        last_name: "user".to_string(),
        created_at: std::time::Instant::now(),
        is_xmlrpc_session: false,
    };

    registry.register_login(session_3).await;

    sleep(Duration::from_secs(120)).await;

    let validation_3 = registry.validate_circuit_code(circuit_code_3).await;
    assert!(validation_3.is_some(), "Circuit code should still be valid after 2 minutes");
    registry.remove_circuit_code(circuit_code_3).await;

    println!("✓ Test 3: 2-minute delayed circuit code use - SUCCESS");

    println!("✓ All timing scenario tests passed!");
}
