#ifndef OPENSIM_PHYSICS_H
#define OPENSIM_PHYSICS_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct PhysicsWorld PhysicsWorld;
typedef struct PhysicsBody PhysicsBody;

// Vector types
typedef struct {
    float x, y, z;
} Vec3;

typedef struct {
    float x, y, z, w;
} Quat;

// Physics world management
PhysicsWorld* physics_world_create(void);
void physics_world_destroy(PhysicsWorld* world);
bool physics_world_step(PhysicsWorld* world, float delta_time);

// Body management
PhysicsBody* physics_body_create(PhysicsWorld* world, Vec3 position, Quat rotation);
void physics_body_destroy(PhysicsBody* body);
void physics_body_set_position(PhysicsBody* body, Vec3 position);
void physics_body_set_rotation(PhysicsBody* body, Quat rotation);
void physics_body_get_position(const PhysicsBody* body, Vec3* out_position);
void physics_body_get_rotation(const PhysicsBody* body, Quat* out_rotation);
void physics_body_apply_force(PhysicsBody* body, Vec3 force);

// Memory management
void physics_memory_init(size_t arena_size);
void physics_memory_deinit(void);

#ifdef __cplusplus
}
#endif

#endif // OPENSIM_PHYSICS_H 