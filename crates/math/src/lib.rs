// math/src/lib.rs

//! Minimal 2D math primitives for game development and physics simulation.
//!
//! This crate provides core mathematical types that are used throughout
//! the Gravita engine:
//!
//! - [`Vec2`] — 2D vector with common operations (add, scale, normalize, rotate, etc.)
//! - [`Aabb`] — Axis-aligned bounding box for collision queries
//! - [`Circle`] — Circle primitive for collision detection
//! - [`Ray2D`] — Ray for raycasting and intersection tests
//! - [`Transform2D`] — Position + rotation transform
//!
//! # Design Goals
//!
//! - **Zero dependencies**: Core math should compile instantly
//! - **Inline everything**: Performance-critical code path
//! - **Simple API**: Prefer free functions and methods over traits
//!
//! # Examples
//!
//! ```
//! use gravita_math::{Vec2, clamp, lerp};
//!
//! let velocity = Vec2::new(100.0, 50.0);
//! let normalized = velocity.normalize();
//! let clamped = clamp(velocity.length(), 0.0, 200.0);
//! ```

#![warn(missing_docs)]

/// Axis-aligned bounding box for spatial queries and collision detection.
pub mod aabb;
/// Circle primitive for collision detection and rendering.
pub mod circle;
/// 2D ray for raycasting and intersection tests.
pub mod ray;
/// 2D transform combining position and rotation.
pub mod transform;
/// 2D vector type with comprehensive operations.
pub mod vector2;

// Re-export common math functions
pub use std::f32::consts::{PI, TAU};

pub use aabb::Aabb;
pub use circle::Circle;
pub use ray::{Ray2D, RayHit};
pub use transform::Transform2D;
pub use vector2::Vec2;

/// Linearly interpolate between two values
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a).mul_add(t, a)
}

/// Clamp a value between min and max
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Map a value from one range to another
#[inline]
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = (value - from_min) / (from_max - from_min);
    lerp(to_min, to_max, t)
}

/// Smooth step interpolation
#[inline]
pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * 2.0f32.mul_add(-t, 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // lerp
    // =========================================================================

    #[test]
    fn lerp_at_zero_returns_start() {
        assert_eq!(lerp(0.0, 100.0, 0.0), 0.0);
    }

    #[test]
    fn lerp_at_one_returns_end() {
        assert_eq!(lerp(0.0, 100.0, 1.0), 100.0);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        assert_eq!(lerp(0.0, 100.0, 0.5), 50.0);
    }

    #[test]
    fn lerp_with_negative_values() {
        assert_eq!(lerp(-100.0, 100.0, 0.5), 0.0);
    }

    #[test]
    fn lerp_extrapolates_beyond_one() {
        assert_eq!(lerp(0.0, 100.0, 2.0), 200.0);
    }

    // =========================================================================
    // clamp
    // =========================================================================

    #[test]
    fn clamp_value_within_range_unchanged() {
        assert_eq!(clamp(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn clamp_value_below_min_returns_min() {
        assert_eq!(clamp(-10.0, 0.0, 100.0), 0.0);
    }

    #[test]
    fn clamp_value_above_max_returns_max() {
        assert_eq!(clamp(150.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn clamp_at_boundary_returns_boundary() {
        assert_eq!(clamp(0.0, 0.0, 100.0), 0.0);
        assert_eq!(clamp(100.0, 0.0, 100.0), 100.0);
    }

    // =========================================================================
    // remap
    // =========================================================================

    #[test]
    fn remap_from_0_1_to_0_100() {
        assert_eq!(remap(0.5, 0.0, 1.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn remap_at_source_min_returns_target_min() {
        assert_eq!(remap(0.0, 0.0, 1.0, 100.0, 200.0), 100.0);
    }

    #[test]
    fn remap_at_source_max_returns_target_max() {
        assert_eq!(remap(1.0, 0.0, 1.0, 100.0, 200.0), 200.0);
    }

    #[test]
    fn remap_inverted_range() {
        // Map from [0, 100] to [100, 0] (inverted)
        assert_eq!(remap(25.0, 0.0, 100.0, 100.0, 0.0), 75.0);
    }

    // =========================================================================
    // smooth_step
    // =========================================================================

    #[test]
    fn smooth_step_at_edge0_returns_zero() {
        assert_eq!(smooth_step(0.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn smooth_step_at_edge1_returns_one() {
        assert_eq!(smooth_step(0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn smooth_step_at_midpoint() {
        assert_eq!(smooth_step(0.0, 1.0, 0.5), 0.5);
    }

    #[test]
    fn smooth_step_below_edge0_clamped_to_zero() {
        assert_eq!(smooth_step(0.0, 1.0, -0.5), 0.0);
    }

    #[test]
    fn smooth_step_above_edge1_clamped_to_one() {
        assert_eq!(smooth_step(0.0, 1.0, 1.5), 1.0);
    }

    #[test]
    fn smooth_step_has_zero_derivative_at_edges() {
        // Smooth step should have smooth transitions (derivative = 0 at edges)
        // Test by checking values near edges are close to 0 and 1
        let near_zero = smooth_step(0.0, 1.0, 0.01);
        let near_one = smooth_step(0.0, 1.0, 0.99);
        assert!(near_zero < 0.01);
        assert!(near_one > 0.99);
    }
}
