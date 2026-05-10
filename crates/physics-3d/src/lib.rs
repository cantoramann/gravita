//! 3D rigid body physics for the Gravita engine.
//!
//! Mirrors the structure of [`gravita_physics`] but works in 3D using
//! [`Vec3`](gravita_math::Vec3) and [`Quat`](gravita_math::Quat) for state.
//!
//! # What's included
//!
//! - [`RigidBody`] with quaternion rotation and a diagonal inertia tensor.
//! - [`CollisionShape::Sphere`] and [`CollisionShape::Aabb`].
//! - Narrow phase: sphere-sphere, sphere-AABB, AABB-AABB.
//! - O(N²) brute-force [`SimpleCollisionDetector`].
//! - [`SemiImplicitEuler`] integrator with stable quaternion update.
//! - [`PhysicsWorld`] step pipeline matching the 2D crate's API shape.
//!
//! # Limitations (v0.1)
//!
//! - No broad phase yet — port the spatial hash from `gravita_physics` when
//!   needed.
//! - AABBs are world-axis-aligned (no OBB / rotated boxes).
//! - No constraints / joints / friction impulses (normal impulses only).

#![warn(missing_docs)]

/// 3D rigid body and collision shape types.
pub mod body;
/// 3D collision detection (narrow phase + simple detector).
pub mod collision;
/// Numerical integrators with quaternion-based rotation.
pub mod integrator;
/// Physics world container.
pub mod world;

pub use body::{BodyType, CollisionShape, RigidBody};
pub use collision::{Contact, SimpleCollisionDetector};
pub use integrator::{Integrator, SemiImplicitEuler};
pub use world::PhysicsWorld;
