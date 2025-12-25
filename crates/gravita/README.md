# Gravita

A modular 2D physics engine and game framework for Rust.

[![Crates.io](https://img.shields.io/crates/v/gravita.svg)](https://crates.io/crates/gravita)
[![Documentation](https://docs.rs/gravita/badge.svg)](https://docs.rs/gravita)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Math primitives**: `Vec2`, `AABB`, `Circle`, `Ray2D`, `Transform2D`
- **Physics simulation**: Rigid body dynamics, collision detection and response
- **Multiple integrators**: Semi-implicit Euler, Verlet
- **Collision detection**: Circle-Circle, AABB-AABB, Circle-AABB
- **Broad phase**: Spatial hash grid for O(n) collision culling
- **Rendering**: Simple CPU-based 2D drawing primitives
- **Game objects**: Pre-built entities (Stickman, Spaceship, Planet)

## Quick Start

```rust
use gravita::prelude::*;

fn main() {
    // Create a physics world
    let mut world = PhysicsWorld::new();

    // Add a dynamic ball
    let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0));
    let ball = RigidBody::new(0, shape)
        .with_position(Vec2::new(100.0, 100.0))
        .with_velocity(Vec2::new(50.0, -30.0));
    world.add_body(ball);

    // Simulation loop
    let dt = 1.0 / 60.0;
    loop {
        world.step(dt);
        // Render your game...
    }
}
```

## Feature Flags

```toml
[dependencies]
# Default: math + physics + renderer
gravita = "0.1"

# Minimal: just math
gravita = { version = "0.1", default-features = false, features = ["math"] }

# Everything
gravita = { version = "0.1", features = ["full"] }
```

| Feature | Default | Description |
|---------|---------|-------------|
| `math` | ✅ | 2D math primitives |
| `physics` | ✅ | Physics engine |
| `renderer` | ✅ | CPU-based 2D rendering |
| `collections` | ❌ | Pre-built game objects |
| `engine-core` | ❌ | Engine integration layer |
| `input` | ❌ | Input handling |
| `assets` | ❌ | Asset loading |
| `full` | ❌ | Enable all features |

## Individual Crates

You can also depend on specific crates directly:

```toml
[dependencies]
gravita-math = "0.1"
gravita-physics = "0.1"
```

## License

MIT License - see [LICENSE](LICENSE) for details.

