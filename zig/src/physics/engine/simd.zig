const std = @import("std");

/// SIMD-optimized physics calculations for better performance
pub const PhysicsSIMD = struct {
    /// SIMD vector type for 4 f32 values
    pub const Vec4 = @Vector(4, f32);
    /// SIMD vector type for 3 f32 values (with padding)
    pub const Vec3 = @Vector(4, f32);

    /// Batch process physics updates using SIMD
    /// Processes 4 physics bodies at once for better performance
    pub fn batchUpdatePositions(
        positions: []align(16) Vec3,
        velocities: []align(16) Vec3,
        forces: []align(16) Vec3,
        masses: []align(16) f32,
        delta_time: f32,
    ) void {
        const dt = @splat(4, delta_time);
        const dt_squared = @splat(4, delta_time * delta_time * 0.5);

        var i: usize = 0;
        const simd_count = positions.len / 4 * 4;

        // Process 4 bodies at a time using SIMD
        while (i < simd_count) : (i += 4) {
            const pos = @Vector(4, Vec3){
                positions[i + 0],
                positions[i + 1],
                positions[i + 2],
                positions[i + 3],
            };

            const vel = @Vector(4, Vec3){
                velocities[i + 0],
                velocities[i + 1],
                velocities[i + 2],
                velocities[i + 3],
            };

            const force = @Vector(4, Vec3){
                forces[i + 0],
                forces[i + 1],
                forces[i + 2],
                forces[i + 3],
            };

            const mass = @Vector(4, f32){
                masses[i + 0],
                masses[i + 1],
                masses[i + 2],
                masses[i + 3],
            };

            // Physics calculations: F = ma, v = v0 + at, x = x0 + v0*t + 0.5*a*t^2
            const inv_mass = @select(f32, mass > @splat(4, @as(f32, 0)), @splat(4, @as(f32, 1)) / mass, @splat(4, @as(f32, 0)));
            const acceleration = force * @splat(4, @as(Vec3, @splat(4, inv_mass)));
            const new_velocity = vel + acceleration * dt;
            const new_position = pos + vel * dt + acceleration * dt_squared;

            // Store results
            positions[i + 0] = new_position[0];
            positions[i + 1] = new_position[1];
            positions[i + 2] = new_position[2];
            positions[i + 3] = new_position[3];

            velocities[i + 0] = new_velocity[0];
            velocities[i + 1] = new_velocity[1];
            velocities[i + 2] = new_velocity[2];
            velocities[i + 3] = new_velocity[3];
        }

        // Process remaining bodies
        while (i < positions.len) : (i += 1) {
            const inv_mass = if (masses[i] > 0) 1.0 / masses[i] else 0.0;
            const acceleration = forces[i] * @splat(4, inv_mass);
            const new_velocity = velocities[i] + acceleration * @splat(4, delta_time);
            const new_position = positions[i] + velocities[i] * @splat(4, delta_time) + 
                               acceleration * @splat(4, delta_time * delta_time * 0.5);

            positions[i] = new_position;
            velocities[i] = new_velocity;
        }
    }

    /// SIMD-optimized collision detection for multiple bodies
    pub fn batchCollisionDetection(
        positions: []align(16) Vec3,
        radii: []align(16) f32,
        collision_pairs: []CollisionPair,
    ) usize {
        var collision_count: usize = 0;
        const max_collisions = collision_pairs.len;

        // Use SIMD for broad phase collision detection
        var i: usize = 0;
        while (i < positions.len) : (i += 4) {
            const end_i = @min(i + 4, positions.len);
            const pos_i = @Vector(4, Vec3){
                positions[i + 0],
                positions[i + 1],
                positions[i + 2],
                positions[i + 3],
            };

            const rad_i = @Vector(4, f32){
                radii[i + 0],
                radii[i + 1],
                radii[i + 2],
                radii[i + 3],
            };

            var j: usize = i + 4;
            while (j < positions.len) : (j += 4) {
                const end_j = @min(j + 4, positions.len);
                const pos_j = @Vector(4, Vec3){
                    positions[j + 0],
                    positions[j + 1],
                    positions[j + 2],
                    positions[j + 3],
                };

                const rad_j = @Vector(4, f32){
                    radii[j + 0],
                    radii[j + 1],
                    radii[j + 2],
                    radii[j + 3],
                };

                // Check collisions between these groups
                for (0..end_i - i) |ii| {
                    for (0..end_j - j) |jj| {
                        if (collision_count >= max_collisions) break;

                        const distance = @sqrt(@reduce(.Add, (pos_i[ii] - pos_j[jj]) * (pos_i[ii] - pos_j[jj])));
                        const min_distance = rad_i[ii] + rad_j[jj];

                        if (distance < min_distance) {
                            collision_pairs[collision_count] = CollisionPair{
                                .body_a = i + ii,
                                .body_b = j + jj,
                                .penetration = min_distance - distance,
                            };
                            collision_count += 1;
                        }
                    }
                }
            }
        }

        return collision_count;
    }

    /// SIMD-optimized gravity application
    pub fn applyGravity(
        velocities: []align(16) Vec3,
        masses: []align(16) f32,
        gravity: Vec3,
        delta_time: f32,
    ) void {
        const gravity_dt = gravity * @splat(4, delta_time);

        var i: usize = 0;
        const simd_count = velocities.len / 4 * 4;

        // Process 4 bodies at a time
        while (i < simd_count) : (i += 4) {
            const mass = @Vector(4, f32){
                masses[i + 0],
                masses[i + 1],
                masses[i + 2],
                masses[i + 3],
            };

            // Only apply gravity to non-zero mass bodies
            const should_apply = mass > @splat(4, @as(f32, 0));
            const gravity_effect = @select(Vec3, should_apply, gravity_dt, @splat(4, @as(Vec3, @splat(4, @as(f32, 0)))));

            velocities[i + 0] += gravity_effect[0];
            velocities[i + 1] += gravity_effect[1];
            velocities[i + 2] += gravity_effect[2];
            velocities[i + 3] += gravity_effect[3];
        }

        // Process remaining bodies
        while (i < velocities.len) : (i += 1) {
            if (masses[i] > 0) {
                velocities[i] += gravity * @splat(4, delta_time);
            }
        }
    }
};

/// Collision pair structure for SIMD collision detection
pub const CollisionPair = struct {
    body_a: usize,
    body_b: usize,
    penetration: f32,
};

test "SIMD physics calculations" {
    const testing = std.testing;

    // Test data
    var positions = [_]PhysicsSIMD.Vec3{
        @splat(4, @as(f32, 0)), // Position 1
        @splat(4, @as(f32, 1)), // Position 2
        @splat(4, @as(f32, 2)), // Position 3
        @splat(4, @as(f32, 3)), // Position 4
    };

    var velocities = [_]PhysicsSIMD.Vec3{
        @splat(4, @as(f32, 1)), // Velocity 1
        @splat(4, @as(f32, 2)), // Velocity 2
        @splat(4, @as(f32, 3)), // Velocity 3
        @splat(4, @as(f32, 4)), // Velocity 4
    };

    var forces = [_]PhysicsSIMD.Vec3{
        @splat(4, @as(f32, 0.1)), // Force 1
        @splat(4, @as(f32, 0.2)), // Force 2
        @splat(4, @as(f32, 0.3)), // Force 3
        @splat(4, @as(f32, 0.4)), // Force 4
    };

    var masses = [_]f32{ 1.0, 2.0, 3.0, 4.0 };

    // Run SIMD physics update
    PhysicsSIMD.batchUpdatePositions(&positions, &velocities, &forces, &masses, 0.016);

    // Verify that positions have changed (basic validation)
    for (positions) |pos| {
        try testing.expect(pos[0] != 0 or pos[1] != 0 or pos[2] != 0 or pos[3] != 0);
    }
} 