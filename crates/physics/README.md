# gravita-physics

A modular 2D physics engine for real-time games and simulations.

## Features

- Rigid body dynamics (linear + angular)
- Collision detection (Circle-Circle, Circle-AABB, AABB-AABB)
- Contact resolution with friction and restitution
- Pluggable integrators (Semi-implicit Euler, Verlet)
- Collision events for game logic

## Architecture

```
PhysicsWorld
├── bodies: Vec<RigidBody>
├── integrator: Box<dyn Integrator>
├── collision_detector: Box<dyn CollisionDetector>
└── contacts: Vec<Contact>
```

### Simulation Pipeline

1. Clear forces
2. Apply gravity
3. Apply user forces + damping
4. Integrate velocities
5. Detect collisions
6. Solve velocity constraints
7. Integrate positions
8. Solve position constraints (penetration correction)

## Usage

```rust
use gravita_physics::{PhysicsWorld, RigidBody, CollisionShape, BodyType};
use gravita_math::{Vec2, Circle, AABB};

// Create world with gravity
let mut world = PhysicsWorld::new();
world.set_gravity(Vec2::new(0.0, -500.0));

// Add static ground
let ground = RigidBody::new(0, CollisionShape::AABB(
    AABB::from_center_size(Vec2::ZERO, Vec2::new(800.0, 50.0))
))
.with_type(BodyType::Static)
.with_position(Vec2::new(400.0, 25.0));
world.add_body(ground);

// Add dynamic ball
let mut ball = RigidBody::new(0, CollisionShape::Circle(
    Circle::new(Vec2::ZERO, 20.0)
))
.with_position(Vec2::new(400.0, 300.0))
.with_density(1.0);
ball.restitution = 0.8; // Bouncy!
world.add_body(ball);

// Game loop
loop {
    world.step(1.0 / 60.0);
    
    // Handle collisions
    for event in world.get_collision_events() {
        if event.impulse_magnitude > 100.0 {
            // Play sound, deal damage, etc.
        }
    }
}
```

## Body Types

| Type | Moved by Physics | Moved by User | Use Case |
|------|------------------|---------------|----------|
| Static | ❌ | ❌ | Ground, walls |
| Kinematic | ❌ | ✅ | Moving platforms |
| Dynamic | ✅ | ✅ | Players, projectiles |

## License

MIT OR Apache-2.0

