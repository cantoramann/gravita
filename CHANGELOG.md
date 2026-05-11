# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-05-10

A 3D-and-WASM release. The engine now has parallel 2D and 3D pipelines on top of a shared `gravita-math`, plus a handwritten JavaScript surface so the same physics runs in the browser without anyone touching Rust.

### Added

#### 3D math (`gravita-math`)

- `Vec3` (component-wise ops, dot, cross, normalize, lerp, distance helpers).
- `Quat` quaternion (Hamilton product, axis-angle, slerp, `from_euler` / `to_euler`, `rotate_vec3`).
- `Aabb3`, `Sphere`, `Obb` collision primitives; `Ray3D` raycasting.
- `Transform3D` (position + rotation + scale, parent-relative composition).
- `Vector` trait so dimension-agnostic helpers (`move_toward`, `attract`, …) work against both `Vec2` and `Vec3`.

#### 3D physics (`gravita-physics-3d`)

- `RigidBody` with quaternion rotation and 3D inertia tensor.
- `CollisionShape::{Sphere, Aabb, Obb}`; OBB–OBB uses the full 15-axis SAT.
- Spatial-hash broad phase (packed `u64` keys, 21 bits per axis).
- Coulomb friction cone (`|jt| ≤ μ·|jn|`) for both 2D and 3D solvers.
- Semi-implicit Euler integration with `q' = normalize(q + 0.5·ω·q·dt)`.

#### 3D rendering (`gravita-renderer-3d`)

- `wgpu` 0.19 instanced-mesh renderer with depth buffer and directional light.
- `App3D` trait plus a `winit::ApplicationHandler` runner so 3D examples stay small.

#### WebAssembly (`gravita-wasm`)

- Handwritten `wasm-bindgen` surface exposing `World2D` and `World3D` directly to JS / TS — plain numeric arguments, `Float32Array` returns. The crate is `#[cfg(target_arch = "wasm32")]`-gated, so native workspace builds compile it to an empty hull.
- `BodyKind` enum (`Dynamic` / `Kinematic` / `Static`) replaces the integer constants that auto-derived bindings would force.
- `wasm-pack build crates/wasm --target web --release` produces a ~75 KB `gravita_wasm_bg.wasm` with auto-generated TypeScript types.

#### Examples

- `cube-3d`: smoke test for the wgpu pipeline (spinning colored cube on a plane).
- `spheres-3d`: bouncing spheres driven by `gravita-physics-3d` and rendered by `gravita-renderer-3d`.

#### CI

- WASM build job: compiles `gravita-wasm` for `wasm32-unknown-unknown` and runs a `wasm-pack` smoke build on every push.

### Changed (breaking)

- **API: drop `get_` prefix** on getters across `PhysicsWorld` and `PhysicsWorld` (3D), per the Rust C-GETTER guideline: `get_bodies()` → `bodies()`, `get_body()` → `body()`, `get_body_mut()` → `body_mut()`, `get_collision_events()` → `collision_events()`.
- **`PhysicsWorld::disable_body`** no longer teleports the body to a magic offscreen position. `RigidBody` gains a real `pub enabled: bool` flag, and the broad/narrow phases skip disabled bodies. Pair with new `enable_body(id)`.
- **`CollisionEvent::relative_velocity`** is now populated with the pre-solve dot product against the contact normal (was always `0.0`).
- **`Aabb`** acronym casing — the type used to be `AABB`. The rename was applied workspace-wide for `clippy::upper_case_acronyms`.

### Removed

- Placeholder crates `gravita-engine-core` and `gravita-assets` (no implementation had landed; reserve the names later if needed).
- Workspace dependencies on `env_logger`, `log`, `approx`, `bytemuck`, `bytemuck_derive`. Replaced with stdlib equivalents (`eprintln!`, manual float comparison, plain `repr(C)` structs).
- Stale "magic offscreen" disable path in `PhysicsWorld` (see Changed).

### Internal

- `gravita-input` promoted from placeholder to a real crate. Holds the `Input` struct + `KeyCode` / `MouseButton` re-exports shared by the 2D `example-shim` and the 3D `runner` module.
- Tetris `draw_cell` 8-argument signature refactored to take a `CellDraw` struct.
- Workspace lint table in `Cargo.toml` expanded to cover the correctness / style / perf / nursery / restriction tiers; all crates now share one source of truth via `[lints] workspace = true`.
- New top-level `ARCHITECTURE.md` mapping the crate graph and per-frame pipelines, plus a `CLAUDE.md` so AI assistants can read themselves in.

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

[0.2.0]: https://github.com/cantoramann/gravita/releases/tag/v0.2.0
[0.1.0]: https://github.com/cantoramann/gravita/releases/tag/v0.1.0
