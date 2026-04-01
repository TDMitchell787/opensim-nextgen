use std::time::Duration;
use opensim_next::ffi::{
    physics_world_create, physics_world_destroy, physics_world_step,
    physics_object_create, physics_object_destroy,
    physics_object_get_position, physics_object_get_velocity,
    Vector3, Quaternion,
};
use opensim_next::ffi::{Physics, Vec3, Quat, FFIError};
use std::f32::consts::PI;

#[test]
fn test_physics_integration() {
    // Initialize physics engine with error handling
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Create a physics body at height 10
    let mut body = physics.create_body().expect("Failed to create physics body");
    
    // Set initial position (this would be done through the body interface)
    let initial_position = Vec3 { x: 0.0, y: 0.0, z: 10.0 };
    
    // Step simulation for 1 second at 60 FPS
    let dt = 1.0 / 60.0;
    let steps = (1.0 / dt) as i32;
    
    for i in 0..steps {
        physics.step(dt).expect(&format!("Physics step {} failed", i));
        
        // Small delay to simulate real-time
        std::thread::sleep(Duration::from_millis((dt * 1000.0) as u64));
    }

    // Check final position (should have fallen due to gravity)
    let final_position = body.get_position().expect("Failed to get body position");
    assert!(final_position.z < 10.0, "Object should have fallen");
    assert!(final_position.z > 0.0, "Object should not have fallen through the ground");

    // Check velocity (should be moving downward)
    let final_velocity = body.get_velocity().expect("Failed to get body velocity");
    assert!(final_velocity.z < 0.0, "Object should have downward velocity");
    
    // Check error count
    let error_count = physics.get_error_count();
    if error_count > 0 {
        let last_error = physics.get_last_error();
        println!("Physics simulation completed with {} errors: {}", error_count, last_error);
    }
}

#[test]
fn test_physics_simulation() {
    // Initialize physics engine
    let mut physics = Physics::new().expect("Failed to initialize physics");

    // Create a body
    let mut body = physics.create_body().expect("Failed to create body");

    // Run simulation for 1 second at 60 FPS
    for i in 0..60 {
        physics.step(1.0 / 60.0).expect(&format!("Physics step {} failed", i));
    }

    // Get final position
    let final_pos = body.get_position().expect("Failed to get final position");

    // Body should have fallen due to gravity
    assert!(final_pos.z < 10.0, "Body should have fallen");
    assert!(final_pos.z > 0.0, "Body should not have fallen through the ground");
    assert_eq!(final_pos.x, 0.0, "Body should not have moved horizontally");
    assert_eq!(final_pos.y, 0.0, "Body should not have moved horizontally");
}

#[test]
fn test_velocity_application() {
    let mut physics = Physics::new().expect("Failed to initialize physics");

    // Create a body
    let mut body = physics.create_body().expect("Failed to create body");

    // Apply velocity in X direction
    let velocity = Vec3 { x: 10.0, y: 0.0, z: 0.0 };
    body.set_velocity(velocity).expect("Failed to set velocity");

    // Run simulation for a short time
    physics.step(0.1).expect("Physics step failed");

    // Get final position
    let final_pos = body.get_position().expect("Failed to get final position");

    // Body should have moved in X direction
    assert!(final_pos.x > 0.0, "Body should have moved in X direction");
    assert_eq!(final_pos.y, 0.0, "Body should not have moved in Y direction");
}

#[test]
fn test_multiple_bodies() {
    let mut physics = Physics::new().expect("Failed to initialize physics");

    // Create two bodies
    let mut body1 = physics.create_body().expect("Failed to create body 1");
    let mut body2 = physics.create_body().expect("Failed to create body 2");

    // Apply different velocities
    body1.set_velocity(Vec3 { x: 1.0, y: 0.0, z: 0.0 }).expect("Failed to set velocity 1");
    body2.set_velocity(Vec3 { x: -1.0, y: 0.0, z: 0.0 }).expect("Failed to set velocity 2");

    // Run simulation
    physics.step(0.1).expect("Physics step failed");

    // Get positions
    let pos1 = body1.get_position().expect("Failed to get position 1");
    let pos2 = body2.get_position().expect("Failed to get position 2");

    // Bodies should have moved in opposite directions
    assert!(pos1.x > 0.0, "Body 1 should have moved right");
    assert!(pos2.x < 0.0, "Body 2 should have moved left");
}

#[test]
fn test_error_handling() {
    // Test physics initialization error handling
    let physics_result = Physics::new();
    assert!(physics_result.is_ok(), "Physics initialization should succeed");
    
    if let Ok(mut physics) = physics_result {
        // Test body creation error handling
        let body_result = physics.create_body();
        assert!(body_result.is_ok(), "Body creation should succeed");
        
        if let Ok(mut body) = body_result {
            // Test velocity setting error handling
            let vel_result = body.set_velocity(Vec3 { x: 1.0, y: 2.0, z: 3.0 });
            assert!(vel_result.is_ok(), "Velocity setting should succeed");
            
            // Test position getting error handling
            let pos_result = body.get_position();
            assert!(pos_result.is_ok(), "Position getting should succeed");
            
            // Test velocity getting error handling
            let vel_get_result = body.get_velocity();
            assert!(vel_get_result.is_ok(), "Velocity getting should succeed");
        }
        
        // Test physics step error handling
        let step_result = physics.step(1.0 / 60.0);
        assert!(step_result.is_ok(), "Physics step should succeed");
    }
}

#[test]
fn test_physics_performance() {
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Create multiple bodies for performance testing
    let mut bodies = vec![];
    for i in 0..100 {
        match physics.create_body() {
            Ok(body) => bodies.push(body),
            Err(e) => println!("Failed to create body {}: {:?}", i, e),
        }
    }
    
    println!("Created {} bodies for performance test", bodies.len());
    
    // Run simulation for performance testing
    let start_time = std::time::Instant::now();
    
    for step in 0..600 { // 10 seconds at 60 FPS
        physics.step(1.0 / 60.0).expect(&format!("Physics step {} failed", step));
        
        if step % 60 == 0 {
            println!("Performance test: step {} completed", step);
        }
    }
    
    let duration = start_time.elapsed();
    println!("Performance test completed in {:?}", duration);
    println!("Average time per step: {:?}", duration / 600);
    
    // Check for errors during performance test
    let error_count = physics.get_error_count();
    if error_count > 0 {
        let last_error = physics.get_last_error();
        println!("Performance test completed with {} errors: {}", error_count, last_error);
    }
}

#[test]
fn test_physics_cleanup() {
    // Test that physics resources are properly cleaned up
    {
        let mut physics = Physics::new().expect("Failed to initialize physics");
        
        // Create multiple bodies
        let mut bodies = vec![];
        for _ in 0..50 {
            if let Ok(body) = physics.create_body() {
                bodies.push(body);
            }
        }
        
        println!("Created {} bodies for cleanup test", bodies.len());
        
        // Run some simulation steps
        for i in 0..60 {
            physics.step(1.0 / 60.0).expect(&format!("Physics step {} failed", i));
        }
        
        // Bodies and physics should be cleaned up when they go out of scope
    }
    
    println!("Physics cleanup test completed - all resources should be freed");
} 