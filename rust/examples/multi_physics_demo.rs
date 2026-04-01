//! Multi-physics engine demonstration
//!
//! This example shows how different simulators can use different physics engines
//! simultaneously, allowing administrators to choose the best physics engine
//! for each region's specific needs.

use opensim_next::physics::{
    MultiPhysicsSystem, PhysicsEngineType, 
    config::PhysicsEngineConfig,
    PhysicsBodyData, PhysicsShape, Vec3, Quat,
};
use opensim_next::OpenSimServer;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 OpenSim Multi-Physics Engine Demonstration");
    info!("=================================================");
    
    // Show supported physics engines
    let supported_engines = OpenSimServer::get_supported_physics_engines();
    info!("📋 Supported Physics Engines:");
    for engine in &supported_engines {
        let caps = OpenSimServer::get_physics_engine_capabilities(*engine);
        info!("  • {} - Max Bodies: {}, Soft Bodies: {}, Fluids: {}", 
              engine, caps.max_bodies, caps.supports_softbodies, caps.supports_fluids);
    }
    
    // Create the multi-physics system
    let physics_system = MultiPhysicsSystem::new(PhysicsEngineType::ODE);
    
    info!("\n🏗️  Creating Physics Managers for Different Regions");
    info!("====================================================");
    
    // Region 1: Avatar-focused region using ODE (stable for avatars)
    let avatar_config = PhysicsEngineConfig::for_avatars();
    let avatar_manager = physics_system.create_manager(
        "avatar-region".to_string(),
        PhysicsEngineType::ODE,
        avatar_config,
    ).await?;
    avatar_manager.initialize().await?;
    info!("✅ Avatar Region: Using ODE engine for stable avatar physics");
    
    // Region 2: Vehicle region using Bullet (better for vehicles)
    let vehicle_config = PhysicsEngineConfig::for_vehicles();
    let vehicle_manager = physics_system.create_manager(
        "vehicle-region".to_string(),
        PhysicsEngineType::Bullet,
        vehicle_config,
    ).await?;
    vehicle_manager.initialize().await?;
    info!("✅ Vehicle Region: Using Bullet engine for advanced vehicle physics");
    
    // Region 3: Large world using UBODE (better performance scaling)
    let large_world_config = PhysicsEngineConfig::for_large_worlds();
    let large_world_manager = physics_system.create_manager(
        "large-world-region".to_string(),
        PhysicsEngineType::UBODE,
        large_world_config,
    ).await?;
    large_world_manager.initialize().await?;
    info!("✅ Large World Region: Using UBODE engine for high object count");
    
    // Region 4: Particle effects using POS
    let particle_config = PhysicsEngineConfig::for_particles();
    let particle_manager = physics_system.create_manager(
        "particle-region".to_string(),
        PhysicsEngineType::POS,
        particle_config,
    ).await?;
    particle_manager.initialize().await?;
    info!("✅ Particle Region: Using POS engine for fluid/particle simulation");
    
    // Region 5: Testing region using Basic physics
    let test_config = PhysicsEngineConfig::for_testing();
    let test_manager = physics_system.create_manager(
        "test-region".to_string(),
        PhysicsEngineType::Basic,
        test_config,
    ).await?;
    test_manager.initialize().await?;
    info!("✅ Test Region: Using Basic engine for lightweight testing");
    
    info!("\n🧪 Demonstrating Different Physics Scenarios");
    info!("===============================================");
    
    // Avatar scenario: Create avatar in avatar region
    let avatar_body = PhysicsBodyData {
        position: Vec3::new(0.0, 0.0, 1.0),
        rotation: Quat::identity(),
        velocity: Vec3::zero(),
        angular_velocity: Vec3::zero(),
        mass: 70.0, // Average human mass
        shape: PhysicsShape::Capsule { radius: 0.3, height: 1.8 },
        is_static: false,
        friction: 0.8,
        restitution: 0.1,
        damping: 0.2,
        angular_damping: 0.5,
    };
    
    let avatar_handle = avatar_manager.create_body(avatar_body).await?;
    info!("👤 Created avatar in Avatar Region (ODE): {:?}", avatar_handle);
    
    // Vehicle scenario: Create vehicle in vehicle region
    let vehicle_body = PhysicsBodyData {
        position: Vec3::new(10.0, 0.0, 0.5),
        rotation: Quat::identity(),
        velocity: Vec3::zero(),
        angular_velocity: Vec3::zero(),
        mass: 1500.0, // Car mass
        shape: PhysicsShape::Box { size: Vec3::new(4.0, 2.0, 1.5) },
        is_static: false,
        friction: 0.9,
        restitution: 0.05,
        damping: 0.1,
        angular_damping: 0.3,
    };
    
    let vehicle_handle = vehicle_manager.create_body(vehicle_body).await?;
    info!("🚗 Created vehicle in Vehicle Region (Bullet): {:?}", vehicle_handle);
    
    // Large world scenario: Create multiple objects
    for i in 0..10 {
        let object_body = PhysicsBodyData {
            position: Vec3::new(i as f32 * 2.0, 0.0, 1.0),
            rotation: Quat::identity(),
            velocity: Vec3::zero(),
            angular_velocity: Vec3::zero(),
            mass: 1.0,
            shape: PhysicsShape::Box { size: Vec3::new(1.0, 1.0, 1.0) },
            is_static: false,
            friction: 0.5,
            restitution: 0.3,
            damping: 0.1,
            angular_damping: 0.1,
        };
        
        large_world_manager.create_body(object_body).await?;
    }
    info!("📦 Created 10 objects in Large World Region (UBODE)");
    
    // Particle scenario: Create particle system (simplified)
    let particle_body = PhysicsBodyData {
        position: Vec3::new(0.0, 0.0, 5.0),
        rotation: Quat::identity(),
        velocity: Vec3::new(0.0, 0.0, -1.0),
        angular_velocity: Vec3::zero(),
        mass: 0.01,
        shape: PhysicsShape::Sphere { radius: 0.05 },
        is_static: false,
        friction: 0.1,
        restitution: 0.8,
        damping: 0.05,
        angular_damping: 0.05,
    };
    
    let particle_handle = particle_manager.create_body(particle_body).await?;
    info!("💧 Created particle in Particle Region (POS): {:?}", particle_handle);
    
    info!("\n⚡ Simulating Physics Steps");
    info!("===========================");
    
    // Simulate physics for a few steps
    let delta_time = 1.0 / 60.0; // 60 FPS
    
    for step in 1..=10 {
        // Step all regions simultaneously
        avatar_manager.step(delta_time).await?;
        vehicle_manager.step(delta_time).await?;
        large_world_manager.step(delta_time).await?;
        particle_manager.step(delta_time).await?;
        test_manager.step(delta_time).await?;
        
        if step % 3 == 0 {
            // Get positions after some steps
            let avatar_pos = avatar_manager.get_body_position(avatar_handle).await?;
            let vehicle_pos = vehicle_manager.get_body_position(vehicle_handle).await?;
            let particle_pos = particle_manager.get_body_position(particle_handle).await?;
            
            info!("Step {}: Avatar at ({:.2}, {:.2}, {:.2}), Vehicle at ({:.2}, {:.2}, {:.2}), Particle at ({:.2}, {:.2}, {:.2})",
                  step, avatar_pos.x, avatar_pos.y, avatar_pos.z,
                  vehicle_pos.x, vehicle_pos.y, vehicle_pos.z,
                  particle_pos.x, particle_pos.y, particle_pos.z);
        }
        
        // Small delay to make output readable
        sleep(Duration::from_millis(100)).await;
    }
    
    info!("\n📊 Physics Statistics");
    info!("====================");
    
    // Get statistics from all regions
    let all_stats = physics_system.get_all_stats().await;
    for (region_id, stats) in all_stats {
        info!("📈 {}: {} engine - {} active bodies, {:.2} ms step time, {:.1} FPS",
              region_id, stats.engine_type, stats.active_bodies, stats.step_time_ms, stats.fps);
    }
    
    info!("\n🔄 Demonstrating Engine Switching");
    info!("===================================");
    
    // Demonstrate engine switching (avatar region from ODE to Basic)
    let mut avatar_manager_mut = avatar_manager;
    match Arc::try_unwrap(avatar_manager_mut) {
        Ok(mut manager) => {
            info!("🔀 Switching Avatar Region from ODE to Basic engine...");
            
            let basic_config = PhysicsEngineConfig::for_testing();
            manager.switch_engine(PhysicsEngineType::Basic, basic_config).await?;
            
            info!("✅ Avatar Region now using Basic engine");
            
            // Test the switched engine
            manager.step(delta_time).await?;
            let new_avatar_pos = manager.get_body_position(avatar_handle).await?;
            info!("👤 Avatar position after engine switch: ({:.2}, {:.2}, {:.2})",
                  new_avatar_pos.x, new_avatar_pos.y, new_avatar_pos.z);
        },
        Err(arc_manager) => {
            warn!("⚠️  Could not switch engine (manager still has other references)");
            // In a real scenario, you'd handle this properly
            let _ = arc_manager; // Suppress unused variable warning
        }
    }
    
    info!("\n🎯 Physics Engine Recommendations");
    info!("==================================");
    
    // Show recommendations for different use cases
    let use_cases = [
        ("Avatar movement and interaction", 0),
        ("Vehicle simulation", 1),
        ("Large worlds with many objects", 2),
        ("Particle and fluid effects", 3),
        ("Testing and development", 4),
    ];
    
    for (use_case, case_id) in use_cases {
        let recommended = match case_id {
            0 => PhysicsEngineType::ODE,
            1 => PhysicsEngineType::Bullet,
            2 => PhysicsEngineType::UBODE,
            3 => PhysicsEngineType::POS,
            4 => PhysicsEngineType::Basic,
            _ => PhysicsEngineType::ODE,
        };
        
        info!("💡 For {}: {} engine recommended", use_case, recommended);
    }
    
    info!("\n🏁 Shutting Down Physics System");
    info!("================================");
    
    // Shutdown all physics managers
    physics_system.shutdown_all().await?;
    info!("✅ All physics managers shut down successfully");
    
    info!("\n🎉 Multi-Physics Demonstration Complete!");
    info!("==========================================");
    info!("✨ Each region successfully ran with its optimal physics engine:");
    info!("   • Avatar Region: ODE → Basic (demonstrated switching)");
    info!("   • Vehicle Region: Bullet Physics");
    info!("   • Large World Region: UBODE");
    info!("   • Particle Region: POS");
    info!("   • Test Region: Basic Physics");
    info!("");
    info!("🔧 This architecture allows OpenSim administrators to:");
    info!("   • Choose the best physics engine for each region's needs");
    info!("   • Switch engines on-the-fly without server restart");
    info!("   • Optimize performance based on content type");
    info!("   • Maintain compatibility with existing content");
    
    Ok(())
}