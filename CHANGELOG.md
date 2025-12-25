# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2025-12-25

🎉 **Initial Release** — Gravita is a modular 2D physics engine and game framework for Rust.

### gravita (umbrella crate)

- Re-exports all Gravita crates under one umbrella
- Feature flags for selective imports (`math`, `physics`, `renderer`, `full`)
- Convenient `prelude` module for common imports

### gravita-math

- `Vec2` struct with comprehensive 2D vector operations
- `Vec2::from_angle()` for creating unit vectors from angles (stable rotation)
- AABB (Axis-Aligned Bounding Box) with intersection and containment tests
- `Circle` struct with collision detection
- `Ray2D` for raycasting against AABB and Circle
- `Transform2D` for position, rotation, and scale
- Utility functions: `lerp`, `remap`, `smooth_step`, `clamp`
- Mathematical constants: `PI`, `TAU`, `E`

### gravita-physics

- `RigidBody` with mass, velocity, forces, and collision shapes
- `BodyType`: Static, Kinematic, Dynamic
- `PhysicsWorld` container for simulation
- Collision detection: Circle-Circle, AABB-AABB, Circle-AABB
- `SimpleCollisionDetector` (O(n²) brute force)
- `SpatialHashDetector` (O(n) broad phase optimization)
- `SemiImplicitEuler` and `Verlet` integrators
- Force generators: Gravity, Drag, Spring, Buoyancy, Field forces
- Collision response with restitution and friction
- Collision events with impulse magnitude

### gravita-renderer

- `draw_circle()` for filled circles with Bresenham-like algorithm
- `draw_line()` using DDA algorithm
- `clear()` to fill the frame buffer
- `draw_axes()` for debug visualization
- RGB color support via `[u8; 4]` format

### gravita-collections

- `Stickman` animated character with walking and jumping
- `Spaceship` with thrust, rotation, and damping
- `Planet` gravitational body visualization

### gravita-engine-core

- Core engine integration layer (WIP)

### gravita-input

- Input abstraction layer (WIP)

### gravita-assets

- Asset loading utilities (WIP)

### Examples

- `bouncing-balls`: Physics demo with dynamic ball spawning
- `gravity-arena`: Orbital mechanics with spaceship controls
- `rotating-rod`: Angular momentum demonstration
- `stickman-walk`: Animated character with walking/jumping
- `tetris`: Classic Tetris implementation
- `froggy-jump`: WASM-compatible Doodle Jump-style game

---

[0.1.0]: https://github.com/cantoramann/gravita/releases/tag/v0.1.0
