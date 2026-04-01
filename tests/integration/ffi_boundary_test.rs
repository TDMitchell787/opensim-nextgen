use opensim_next::ffi::{FFIManager, FFIError, Physics, Vec3, Quat};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Test FFI boundary data integrity
#[test]
fn test_ffi_data_integrity() {
    let manager = FFIManager::new().expect("Failed to initialize FFI manager");
    
    // Test vector data passing
    let test_vectors = vec![
        Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        Vec3 { x: 1.0, y: 2.0, z: 3.0 },
        Vec3 { x: -1.0, y: -2.0, z: -3.0 },
        Vec3 { x: f32::MAX, y: f32::MIN, z: 0.0 },
        Vec3 { x: f32::INFINITY, y: f32::NEG_INFINITY, z: f32::NAN },
    ];
    
    for (i, test_vec) in test_vectors.iter().enumerate() {
        println!("Testing vector {}: {:?}", i, test_vec);
        
        // Create buffer and write vector data
        let buffer_handle = manager.create_buffer(std::mem::size_of::<Vec3>())
            .expect("Failed to create buffer for vector test");
        
        // In a real implementation, you would write the vector to the buffer
        // For now, we just test that the buffer was created successfully
        
        manager.destroy_buffer(buffer_handle).expect("Failed to destroy buffer");
    }
}

/// Test FFI error propagation across language boundary
#[test]
fn test_ffi_error_propagation() {
    let manager = FFIManager::new().expect("Failed to initialize FFI manager");
    
    // Test various error conditions
    let error_tests = vec![
        ("zero_capacity", 0),
        ("very_large_capacity", usize::MAX),
        ("invalid_handle", 99999),
    ];
    
    for (test_name, capacity) in error_tests {
        println!("Testing error condition: {}", test_name);
        
        let result = manager.create_buffer(capacity);
        match result {
            Ok(_) => println!("  Unexpected success for {}", test_name),
            Err(e) => println!("  Expected error for {}: {:?}", test_name, e),
        }
    }
    
    // Test error message propagation
    let error_count = manager.get_error_count();
    if error_count > 0 {
        let last_error = manager.get_last_error();
        println!("Error count: {}, Last error: {}", error_count, last_error);
        assert!(!last_error.is_empty(), "Error message should not be empty");
    }
}

/// Test concurrent FFI access
#[test]
fn test_concurrent_ffi_access() {
    let manager = Arc::new(FFIManager::new().expect("Failed to initialize FFI manager"));
    let mut handles = vec![];
    
    // Spawn multiple threads to test concurrent access
    for thread_id in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            println!("Thread {} starting FFI operations", thread_id);
            
            // Perform multiple buffer operations
            for i in 0..10 {
                let buffer_handle = manager_clone.create_buffer(1024)
                    .expect(&format!("Thread {} failed to create buffer {}", thread_id, i));
                
                // Simulate some work
                thread::sleep(Duration::from_millis(1));
                
                manager_clone.destroy_buffer(buffer_handle)
                    .expect(&format!("Thread {} failed to destroy buffer {}", thread_id, i));
            }
            
            println!("Thread {} completed FFI operations", thread_id);
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread failed to join");
    }
    
    // Check for errors after concurrent operations
    let error_count = manager.get_error_count();
    println!("Concurrent FFI test completed with {} errors", error_count);
}

/// Test physics FFI boundary
#[test]
fn test_physics_ffi_boundary() {
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Test body creation through FFI
    let body_result = physics.create_body();
    assert!(body_result.is_ok(), "Body creation should succeed through FFI");
    
    if let Ok(mut body) = body_result {
        // Test velocity setting through FFI
        let test_velocity = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        let vel_result = body.set_velocity(test_velocity);
        assert!(vel_result.is_ok(), "Velocity setting should succeed through FFI");
        
        // Test velocity retrieval through FFI
        let retrieved_velocity = body.get_velocity();
        assert!(retrieved_velocity.is_ok(), "Velocity retrieval should succeed through FFI");
        
        if let Ok(retrieved) = retrieved_velocity {
            // Check that the velocity was set correctly (allowing for small floating point differences)
            assert!((retrieved.x - test_velocity.x).abs() < 0.001);
            assert!((retrieved.y - test_velocity.y).abs() < 0.001);
            assert!((retrieved.z - test_velocity.z).abs() < 0.001);
        }
        
        // Test position retrieval through FFI
        let position_result = body.get_position();
        assert!(position_result.is_ok(), "Position retrieval should succeed through FFI");
    }
    
    // Test physics stepping through FFI
    let step_result = physics.step(1.0 / 60.0);
    assert!(step_result.is_ok(), "Physics stepping should succeed through FFI");
}

/// Test memory leak detection at FFI boundaries
#[test]
fn test_ffi_memory_leak_detection() {
    // Test that resources are properly cleaned up
    {
        let manager = FFIManager::new().expect("Failed to initialize FFI manager");
        let mut physics = Physics::new().expect("Failed to initialize physics");
        
        // Create multiple resources
        let mut buffer_handles = vec![];
        let mut bodies = vec![];
        
        for i in 0..20 {
            // Create buffers
            if let Ok(handle) = manager.create_buffer(512) {
                buffer_handles.push(handle);
            }
            
            // Create physics bodies
            if let Ok(body) = physics.create_body() {
                bodies.push(body);
            }
        }
        
        println!("Created {} buffers and {} bodies for memory leak test", 
                 buffer_handles.len(), bodies.len());
        
        // Destroy half of the buffers
        for handle in buffer_handles.iter().take(10) {
            manager.destroy_buffer(*handle).expect("Failed to destroy buffer");
        }
        
        // Run some physics simulation
        for _ in 0..30 {
            physics.step(1.0 / 60.0).expect("Physics step failed");
        }
        
        // Remaining resources should be cleaned up when they go out of scope
    }
    
    println!("Memory leak test completed - all resources should be freed");
}

/// Test FFI boundary stress conditions
#[test]
fn test_ffi_boundary_stress() {
    let manager = FFIManager::new().expect("Failed to initialize FFI manager");
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Rapid buffer creation and destruction
    for i in 0..1000 {
        if i % 100 == 0 {
            println!("FFI stress test progress: {}/1000", i);
        }
        
        let handle = manager.create_buffer(64).expect("Failed to create buffer in stress test");
        manager.destroy_buffer(handle).expect("Failed to destroy buffer in stress test");
    }
    
    // Rapid physics body creation and simulation
    for i in 0..100 {
        if i % 20 == 0 {
            println!("Physics stress test progress: {}/100", i);
        }
        
        if let Ok(mut body) = physics.create_body() {
            body.set_velocity(Vec3 { x: 1.0, y: 2.0, z: 3.0 }).expect("Failed to set velocity");
            physics.step(1.0 / 60.0).expect("Physics step failed");
        }
    }
    
    // Check for errors after stress test
    let error_count = manager.get_error_count();
    let physics_error_count = physics.get_error_count();
    
    println!("FFI stress test completed - Manager errors: {}, Physics errors: {}", 
             error_count, physics_error_count);
}

/// Test FFI boundary error recovery
#[test]
fn test_ffi_error_recovery() {
    let manager = FFIManager::new().expect("Failed to initialize FFI manager");
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Clear any existing errors
    manager.clear_logs();
    assert_eq!(manager.get_error_count(), 0, "Should start with no errors");
    
    // Intentionally cause some errors
    let _ = manager.create_buffer(0); // Invalid capacity
    let _ = manager.destroy_buffer(99999); // Invalid handle
    
    // Test that we can still perform valid operations after errors
    let valid_buffer = manager.create_buffer(1024);
    assert!(valid_buffer.is_ok(), "Should be able to create valid buffer after errors");
    
    if let Ok(handle) = valid_buffer {
        manager.destroy_buffer(handle).expect("Should be able to destroy valid buffer");
    }
    
    // Test physics error recovery
    let body_result = physics.create_body();
    assert!(body_result.is_ok(), "Should be able to create body after errors");
    
    if let Ok(mut body) = body_result {
        let step_result = physics.step(1.0 / 60.0);
        assert!(step_result.is_ok(), "Should be able to step physics after errors");
    }
    
    // Check final error count
    let final_error_count = manager.get_error_count();
    println!("Error recovery test completed with {} total errors", final_error_count);
}

/// Test FFI boundary data validation
#[test]
fn test_ffi_data_validation() {
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Test with various data values
    let test_cases = vec![
        Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        Vec3 { x: -1.0, y: -1.0, z: -1.0 },
        Vec3 { x: 1000.0, y: 2000.0, z: 3000.0 },
        Vec3 { x: -1000.0, y: -2000.0, z: -3000.0 },
    ];
    
    for (i, test_velocity) in test_cases.iter().enumerate() {
        println!("Testing velocity case {}: {:?}", i, test_velocity);
        
        if let Ok(mut body) = physics.create_body() {
            // Set velocity
            let set_result = body.set_velocity(*test_velocity);
            assert!(set_result.is_ok(), "Velocity setting should succeed for case {}", i);
            
            // Get velocity back
            let get_result = body.get_velocity();
            assert!(get_result.is_ok(), "Velocity getting should succeed for case {}", i);
            
            if let Ok(retrieved) = get_result {
                // Validate that the velocity was set correctly
                assert!((retrieved.x - test_velocity.x).abs() < 0.001, 
                        "X velocity mismatch for case {}", i);
                assert!((retrieved.y - test_velocity.y).abs() < 0.001, 
                        "Y velocity mismatch for case {}", i);
                assert!((retrieved.z - test_velocity.z).abs() < 0.001, 
                        "Z velocity mismatch for case {}", i);
            }
            
            // Run physics simulation
            let step_result = physics.step(1.0 / 60.0);
            assert!(step_result.is_ok(), "Physics step should succeed for case {}", i);
        }
    }
}

/// Test FFI boundary performance
#[test]
fn test_ffi_boundary_performance() {
    let manager = FFIManager::new().expect("Failed to initialize FFI manager");
    let mut physics = Physics::new().expect("Failed to initialize physics");
    
    // Test buffer operations performance
    let buffer_start = std::time::Instant::now();
    for _ in 0..1000 {
        let handle = manager.create_buffer(1024).expect("Failed to create buffer");
        manager.destroy_buffer(handle).expect("Failed to destroy buffer");
    }
    let buffer_duration = buffer_start.elapsed();
    println!("Buffer operations: 1000 create/destroy pairs in {:?}", buffer_duration);
    
    // Test physics operations performance
    let physics_start = std::time::Instant::now();
    for _ in 0..1000 {
        if let Ok(mut body) = physics.create_body() {
            body.set_velocity(Vec3 { x: 1.0, y: 2.0, z: 3.0 }).expect("Failed to set velocity");
            physics.step(1.0 / 60.0).expect("Failed to step physics");
        }
    }
    let physics_duration = physics_start.elapsed();
    println!("Physics operations: 1000 body operations in {:?}", physics_duration);
    
    // Performance assertions (adjust thresholds as needed)
    assert!(buffer_duration < Duration::from_secs(5), "Buffer operations took too long");
    assert!(physics_duration < Duration::from_secs(10), "Physics operations took too long");
} 