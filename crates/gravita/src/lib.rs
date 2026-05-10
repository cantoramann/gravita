//! # Gravita
//!
//! A modular 2D physics engine and game framework for Rust.
//!
//! Gravita provides everything you need to build 2D games and physics simulations:
//!
//! - **Math primitives**: Vectors, transforms, shapes (Aabb, Circle, Ray)
//! - **Physics simulation**: Rigid body dynamics, collision detection and response
//! - **Rendering utilities**: Simple CPU-based 2D drawing primitives
//! - **Game objects**: Pre-built entities like Stickman, Spaceship, Planet
//!
//! ## Quick Start
//!
//! ```rust
//! use gravita::prelude::*;
//!
//! // Create a physics world
//! let mut world = PhysicsWorld::new();
//!
//! // Add a dynamic circle
//! let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0));
//! let ball = RigidBody::new(0, shape)
//!     .with_position(Vec2::new(100.0, 100.0))
//!     .with_velocity(Vec2::new(50.0, -30.0));
//! world.add_body(ball);
//!
//! // Step the simulation
//! world.step(1.0 / 60.0);
//! ```
//!
//! ## Feature Flags
//!
//! Enable only what you need:
//!
//! ```toml
//! [dependencies]
//! gravita = { version = "0.1", default-features = false, features = ["math", "physics"] }
//! ```
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `math` | ✅ | 2D math primitives (Vec2, Aabb, Circle, etc.) |
//! | `physics` | ✅ | Physics engine (RigidBody, PhysicsWorld, etc.) |
//! | `renderer` | ✅ | CPU-based 2D rendering utilities |
//! | `collections` | ❌ | Pre-built game objects (Stickman, Spaceship) |
//! | `engine-core` | ❌ | Core engine integration layer |
//! | `input` | ❌ | Input handling abstraction |
//! | `assets` | ❌ | Asset loading utilities |
//! | `full` | ❌ | Enable all features |
//!
//! ## Crate Structure
//!
//! Gravita is composed of several focused crates:
//!
//! - [`gravita-math`](https://docs.rs/gravita-math) - Math primitives
//! - [`gravita-physics`](https://docs.rs/gravita-physics) - Physics simulation
//! - [`gravita-renderer`](https://docs.rs/gravita-renderer) - Rendering utilities
//! - [`gravita-collections`](https://docs.rs/gravita-collections) - Game objects
//! - [`gravita-engine-core`](https://docs.rs/gravita-engine-core) - Engine core
//! - [`gravita-input`](https://docs.rs/gravita-input) - Input handling
//! - [`gravita-assets`](https://docs.rs/gravita-assets) - Asset loading

#![warn(missing_docs)]

// ─── Re-exports ──────────────────────────────────────────────────────────────

#[cfg(feature = "assets")]
pub use gravita_assets as assets;
#[cfg(feature = "collections")]
pub use gravita_collections as collections;
#[cfg(feature = "engine-core")]
pub use gravita_engine_core as engine_core;
#[cfg(feature = "input")]
pub use gravita_input as input;
#[cfg(feature = "math")]
pub use gravita_math as math;
#[cfg(feature = "physics")]
pub use gravita_physics as physics;
#[cfg(feature = "physics-3d")]
pub use gravita_physics_3d as physics_3d;
#[cfg(feature = "renderer")]
pub use gravita_renderer as renderer;
#[cfg(feature = "renderer-3d")]
pub use gravita_renderer_3d as renderer_3d;

// ─── Prelude ─────────────────────────────────────────────────────────────────

/// Convenient re-exports for common use cases.
///
/// ```rust
/// use gravita::prelude::*;
/// ```
pub mod prelude {
    // Math types
    // Collections types
    #[cfg(feature = "collections")]
    pub use gravita_collections::{Planet, Spaceship, Stickman};
    #[cfg(feature = "math")]
    pub use gravita_math::{
        Aabb, Aabb3, Circle, PI, Quat, Ray2D, Ray3D, RayHit3D, Sphere, TAU, Transform2D,
        Transform3D, Vec2, Vec3, Vector,
    };
    // Physics types
    #[cfg(feature = "physics")]
    pub use gravita_physics::{
        BodyType, CollisionShape, PhysicsWorld, RigidBody,
        collision::{CollisionDetector, Contact, SimpleCollisionDetector, SpatialHashDetector},
        integrator::{Integrator, SemiImplicitEuler, Verlet},
    };
    // Physics 3D types
    #[cfg(feature = "physics-3d")]
    pub use gravita_physics_3d::{
        BodyType as BodyType3D, CollisionShape as CollisionShape3D, Contact as Contact3D,
        PhysicsWorld as PhysicsWorld3D, RigidBody as RigidBody3D,
        SemiImplicitEuler as SemiImplicitEuler3D, SimpleCollisionDetector as SimpleCollisionDetector3D,
    };
    // Renderer types
    #[cfg(feature = "renderer")]
    pub use gravita_renderer::{clear, draw_axes, draw_circle, draw_line};
    // Renderer 3D types — typically the entry point users want is `run` +
    // the `App3D` trait. The full surface is available via `gravita::renderer_3d`.
    #[cfg(feature = "renderer-3d")]
    pub use gravita_renderer_3d::{
        App3D, Camera as Camera3D, Instance as Instance3D, Mesh as Mesh3D,
        MeshHandle as MeshHandle3D, Renderer3D, WindowConfig as WindowConfig3D,
        run as run_3d,
    };
}
