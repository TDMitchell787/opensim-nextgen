//! Quick integration test to verify core functionality works

use opensim_next::physics::config::PhysicsEngineConfig;
use opensim_next::physics::{MultiPhysicsSystem, PhysicsEngineType};

#[tokio::test]
async fn test_multi_physics_system_creation() {
    // Test that we can create the multi-physics system
    let physics_system = MultiPhysicsSystem::new(PhysicsEngineType::ODE);

    // Test that we can list supported engines
    let engines = MultiPhysicsSystem::supported_engines();
    assert!(!engines.is_empty());
    assert!(engines.contains(&PhysicsEngineType::ODE));
    assert!(engines.contains(&PhysicsEngineType::Bullet));
    assert!(engines.contains(&PhysicsEngineType::POS));

    // Test engine capabilities
    let ode_caps = MultiPhysicsSystem::get_engine_capabilities(PhysicsEngineType::ODE);
    assert_eq!(ode_caps.max_bodies, 10000);
    assert!(ode_caps.supports_heightfields);

    let bullet_caps = MultiPhysicsSystem::get_engine_capabilities(PhysicsEngineType::Bullet);
    assert_eq!(bullet_caps.max_bodies, 50000);
    assert!(bullet_caps.supports_softbodies);

    println!("✅ Multi-physics system test passed!");
}

#[tokio::test]
async fn test_physics_engine_creation() {
    let physics_system = MultiPhysicsSystem::new(PhysicsEngineType::ODE);

    // Test creating a physics manager
    let config = PhysicsEngineConfig::default();
    let result = physics_system
        .create_manager("test-region".to_string(), PhysicsEngineType::ODE, config)
        .await;

    assert!(
        result.is_ok(),
        "Failed to create physics manager: {:?}",
        result.err()
    );

    let manager = result.unwrap();
    let engine_type = manager.engine_type();
    assert_eq!(engine_type, PhysicsEngineType::ODE);

    println!("✅ Physics engine creation test passed!");
}

#[test]
fn test_physics_types() {
    // Test basic physics types
    use opensim_next::physics::{PhysicsEngineType, Quat, Vec3};

    let vec = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(vec.x, 1.0);
    assert_eq!(vec.y, 2.0);
    assert_eq!(vec.z, 3.0);

    let quat = Quat::identity();
    assert_eq!(quat.w, 1.0);
    assert_eq!(quat.x, 0.0);

    let engine_type = PhysicsEngineType::default();
    assert_eq!(engine_type, PhysicsEngineType::ODE);

    println!("✅ Physics types test passed!");
}
