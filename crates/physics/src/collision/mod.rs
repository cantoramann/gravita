// physics/src/collision/mod.rs

//! Collision detection pipeline for the physics engine.
//!
//! Collision detection is split into two phases:
//! - **Broad phase**: Quickly finds potentially colliding pairs using spatial partitioning
//! - **Narrow phase**: Precisely tests each pair and generates contact information

/// Broad phase collision detection using spatial partitioning.
pub mod broad_phase;
/// Contact and collision event data structures.
pub mod contact;
/// High-level collision detector trait and implementations.
pub mod detector;
/// Narrow phase collision tests between specific shape pairs.
pub mod narrow_phase;

pub use broad_phase::{BroadPhase, SpatialHashGrid};
pub use contact::{CollisionEvent, Contact};
pub use detector::{CollisionDetector, SimpleCollisionDetector, SpatialHashDetector};
pub use narrow_phase::{test_aabb_aabb, test_circle_aabb, test_circle_circle};
