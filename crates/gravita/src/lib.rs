//! # Gravita
//!
//! A small, readable 2D + 3D physics engine and game framework for Rust.
//!
//! The umbrella crate re-exports every other gravita crate behind a cargo
//! feature so consumers compile only what they need.
//!
//! ## Quick Start (2D)
//!
//! ```rust
//! use gravita::prelude::*;
//!
//! let mut world = PhysicsWorld::new();
//! let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0));
//! let ball = RigidBody::new(0, shape)
//!     .with_position(Vec2::new(100.0, 100.0))
//!     .with_velocity(Vec2::new(50.0, -30.0));
//! world.add_body(ball);
//! world.step(1.0 / 60.0);
//! ```
//!
//! ## Quick Start (3D)
//!
//! ```toml
//! gravita = { version = "0.1", default-features = false, features = ["math", "physics-3d", "renderer-3d"] }
//! ```
//!
//! ```ignore
//! use gravita::prelude::*;
//!
//! let mut world = PhysicsWorld3D::new();
//! world.set_gravity(Vec3::new(0.0, -9.81, 0.0));
//! let ball = RigidBody3D::new(0, CollisionShape3D::Sphere(Sphere::new(Vec3::ZERO, 0.5)))
//!     .with_position(Vec3::new(0.0, 5.0, 0.0));
//! world.add_body(ball);
//! world.step(1.0 / 60.0);
//! ```
//!
//! ## Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `math` | yes | 2D + 3D math (Vec2, Vec3, Quat, Aabb, Aabb3, Sphere, Obb, ...) |
//! | `physics` | yes | 2D rigid-body sim |
//! | `physics-3d` | no | 3D rigid-body sim |
//! | `renderer` | yes | 2D CPU framebuffer rasterizer |
//! | `renderer-3d` | no | wgpu 3D renderer + winit runner |
//! | `collections` | no | Pre-built 2D characters (Stickman, Spaceship, Planet) |
//! | `input` | no | Input snapshot (keyboard + mouse + cursor) |
//! | `full` | no | All features above |
//!
//! ## Crate map
//!
//! - [`gravita-math`](https://docs.rs/gravita-math) — math primitives
//! - [`gravita-physics`](https://docs.rs/gravita-physics) — 2D physics
//! - [`gravita-physics-3d`](https://docs.rs/gravita-physics-3d) — 3D physics
//! - [`gravita-renderer`](https://docs.rs/gravita-renderer) — 2D rasterizer
//! - [`gravita-renderer-3d`](https://docs.rs/gravita-renderer-3d) — wgpu 3D renderer
//! - [`gravita-collections`](https://docs.rs/gravita-collections) — example game objects
//! - [`gravita-input`](https://docs.rs/gravita-input) — input state snapshot

#![warn(missing_docs)]

// ─── Re-exports ──────────────────────────────────────────────────────────────

#[cfg(feature = "collections")]
pub use gravita_collections as collections;
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
    #[cfg(feature = "collections")]
    pub use gravita_collections::{Planet, Spaceship, Stickman};
    #[cfg(feature = "math")]
    pub use gravita_math::{
        Aabb, Aabb3, Circle, Obb, PI, Quat, Ray2D, Ray3D, RayHit3D, Sphere, TAU, Transform2D,
        Transform3D, Vec2, Vec3, Vector,
    };
    #[cfg(feature = "physics")]
    pub use gravita_physics::{
        BodyType, CollisionShape, PhysicsWorld, RigidBody,
        collision::{CollisionDetector, Contact, SimpleCollisionDetector, SpatialHashDetector},
        integrator::{Integrator, SemiImplicitEuler, Verlet},
    };
    #[cfg(feature = "physics-3d")]
    pub use gravita_physics_3d::{
        BodyType as BodyType3D, CollisionShape as CollisionShape3D, Contact as Contact3D,
        PhysicsWorld as PhysicsWorld3D, RigidBody as RigidBody3D,
        SemiImplicitEuler as SemiImplicitEuler3D,
        SimpleCollisionDetector as SimpleCollisionDetector3D,
    };
    #[cfg(feature = "renderer")]
    pub use gravita_renderer::{clear, draw_axes, draw_circle, draw_line};
    #[cfg(feature = "renderer-3d")]
    pub use gravita_renderer_3d::{
        App3D, Camera as Camera3D, Instance as Instance3D, Mesh as Mesh3D,
        MeshHandle as MeshHandle3D, Renderer3D, WindowConfig as WindowConfig3D, run as run_3d,
    };
}
