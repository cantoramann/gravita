// physics/src/lib.rs

//! A modular 2D physics engine for real-time games and simulations.
//!
//! This crate provides rigid body dynamics with collision detection and resolution.
//! It is designed to be simple, predictable, and suitable for games.
//!
//! # Architecture
//!
//! The physics simulation is organized around these core concepts:
//!
//! - [`PhysicsWorld`] — Container that owns all bodies and steps the simulation
//! - [`RigidBody`] — A simulated object with position, velocity, and mass
//! - [`BodyType`] — Classification: Static, Kinematic, or Dynamic
//! - [`CollisionShape`] — Geometric primitive (Circle or Aabb)
//! - [`Contact`] — Collision information between two bodies
//!
//! # Simulation Loop
//!
//! The physics world uses a fixed-timestep approach:
//!
//! 1. Clear forces from the previous step
//! 2. Apply gravity and user forces
//! 3. Integrate velocities
//! 4. Detect collisions (broad phase → narrow phase)
//! 5. Resolve collisions (velocity + position correction)
//! 6. Integrate positions
//!
//! # Example
//!
//! ```
//! use gravita_math::{Aabb, Circle, Vec2};
//! use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody};
//!
//! let mut world = PhysicsWorld::new();
//! world.set_gravity(Vec2::new(0.0, -500.0));
//!
//! // Add a static ground
//! let ground = RigidBody::new(
//!     0,
//!     CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(800.0, 50.0))),
//! )
//! .with_type(BodyType::Static)
//! .with_position(Vec2::new(400.0, 25.0));
//! world.add_body(ground);
//!
//! // Add a dynamic ball
//! let ball = RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 20.0)))
//!     .with_position(Vec2::new(400.0, 300.0));
//! world.add_body(ball);
//!
//! // Step the simulation
//! world.step(1.0 / 60.0);
//! ```
//!
//! # Collision Events
//!
//! After each step, you can query collision events for game logic:
//!
//! ```ignore
//! for event in world.collision_events() {
//!     if event.impulse_magnitude > 100.0 {
//!         // Play impact sound, deal damage, etc.
//!     }
//! }
//! ```

#![warn(missing_docs)]

/// Rigid body types and collision shapes.
pub mod body;
/// Collision detection pipeline (broad phase, narrow phase, contacts).
pub mod collision;
/// Force generators and external influences.
pub mod forces;
/// Numerical integration schemes (Euler, Verlet).
pub mod integrator;
/// Physics world container and simulation stepping.
pub mod world;

pub use body::{BodyType, CollisionShape, RigidBody};
pub use collision::{CollisionDetector, CollisionEvent, Contact};
pub use world::PhysicsWorld;
