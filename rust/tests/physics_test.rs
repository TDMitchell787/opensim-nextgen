use opensim_next::ffi::{Physics, Vec3};

#[test]
fn test_physics_body_movement() {
    let mut physics = Physics::new().expect("Failed to initialize physics");
    let mut body = physics.create_body().expect("Failed to create body");
    body.set_velocity(Vec3 {
        x: 0.0,
        y: 0.0,
        z: 5.0,
    })
    .expect("Failed to set velocity");
    let initial_pos = body.get_position().expect("Failed to get initial position");
    for _ in 0..10 {
        physics.step(0.02).expect("Failed to step physics");
    }
    let final_pos = body.get_position().expect("Failed to get final position");
    println!("Initial z: {}, Final z: {}", initial_pos.z, final_pos.z);
    assert!(final_pos.z > initial_pos.z, "Body should have moved up");
}

#[test]
fn test_physics_body_velocity_set_get() {
    let mut physics = Physics::new().expect("Failed to initialize physics");
    let mut body = physics.create_body().expect("Failed to create body");
    let vel = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    body.set_velocity(vel).expect("Failed to set velocity");
    let got = body.get_velocity().expect("Failed to get velocity");
    assert!((got.x - 1.0).abs() < 0.01);
    assert!((got.y - 2.0).abs() < 0.01);
    assert!((got.z - 3.0).abs() < 0.01);
}
