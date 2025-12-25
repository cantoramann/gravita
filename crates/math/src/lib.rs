// math/src/lib.rs

//! Minimal 2D math primitives for game development and physics simulation.
//!
//! This crate provides core mathematical types that are used throughout
//! the Gravita engine:
//!
//! - [`Vec2`] — 2D vector with common operations (add, scale, normalize, rotate, etc.)
//! - [`AABB`] — Axis-aligned bounding box for collision queries
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

pub use aabb::AABB;
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

    // =========================================================================
    // Floating-Point Precision Tests
    // =========================================================================

    /// Machine epsilon for f32 (~1.19e-7)
    const F32_EPSILON: f32 = f32::EPSILON;

    /// A more practical epsilon for game physics (~1e-5)
    #[allow(dead_code)]
    const PHYSICS_EPSILON: f32 = 1e-5;

    // -------------------------------------------------------------------------
    // Near-Zero Values
    // -------------------------------------------------------------------------

    #[test]
    fn precision_near_zero_addition() {
        let tiny = 1e-7_f32;
        let result = tiny + tiny;
        assert!((result - 2e-7).abs() < 1e-13, "Near-zero addition failed");
    }

    #[test]
    fn precision_near_zero_vector_normalize() {
        let v = Vec2::new(1e-6, 1e-6);
        let n = v.normalize();
        // Should still produce a unit vector or zero
        let len = n.length();
        assert!(
            len == 0.0 || (len - 1.0).abs() < 0.001,
            "Near-zero normalize failed: length = {len}"
        );
    }

    #[test]
    fn precision_denormalized_numbers() {
        // Test with denormalized (subnormal) numbers
        let denorm = f32::MIN_POSITIVE / 2.0; // This is subnormal
        let result = lerp(0.0, denorm, 0.5);
        // Should not produce NaN or infinity
        assert!(result.is_finite(), "Denormalized lerp produced non-finite");
    }

    #[test]
    fn precision_very_small_differences() {
        // Two values that differ by 1 ULP
        let a = 1.0_f32;
        let b = f32::from_bits(a.to_bits() + 1);
        let diff = (b - a).abs();
        assert!(diff < 1e-6, "1 ULP difference should be tiny");
        assert!(diff > 0.0, "1 ULP difference should be non-zero");
    }

    // -------------------------------------------------------------------------
    // Large Values
    // -------------------------------------------------------------------------

    #[test]
    fn precision_large_value_operations() {
        let large = 1e30_f32;
        let v = Vec2::new(large, large);
        let scaled = v * 2.0;
        assert_eq!(scaled.x, 2e30);
        assert_eq!(scaled.y, 2e30);
    }

    #[test]
    fn precision_large_small_addition() {
        // Adding very small to very large loses precision
        let large = 1e10_f32;
        let small = 1e-5_f32;
        let result = large + small;
        // Due to f32 precision limits, small value may be lost
        // This test documents the expected behavior
        assert!(
            (result - large).abs() < 1.0,
            "Large + small should approximately equal large"
        );
    }

    #[test]
    fn precision_large_vector_normalize() {
        // Very large vectors can overflow during length calculation
        // Test a moderately large vector that won't overflow
        let v = Vec2::new(1e10, 1e10);
        let n = v.normalize();
        let len = n.length();
        assert!(
            (len - 1.0).abs() < 0.01,
            "Large vector normalize should produce unit length: got {len}"
        );
    }

    #[test]
    fn precision_extreme_large_vector_overflows() {
        // Document that extremely large vectors overflow during normalization
        let v = Vec2::new(1e20, 1e20);
        let n = v.normalize();
        // This produces infinity during length calculation, which then produces 0/inf = 0
        // or can produce approximately 1.0 in some floating point implementations
        // This is expected behavior and users should clamp values before normalization
        let len = n.length();
        // Accept 0.0, ~1.0, NaN, or Infinity as all are valid degenerate outcomes
        assert!(
            len == 0.0 || len.is_nan() || len.is_infinite() || (len - 1.0).abs() < 0.01,
            "Extreme large vector normalization produces unexpected result: {len}"
        );
    }

    // -------------------------------------------------------------------------
    // Special Values (Infinity, NaN)
    // -------------------------------------------------------------------------

    #[test]
    fn precision_infinity_handling() {
        let inf = f32::INFINITY;
        let v = Vec2::new(inf, 0.0);
        // Operations with infinity should propagate correctly
        assert!(v.length().is_infinite());
    }

    #[test]
    fn precision_nan_propagation() {
        let nan = f32::NAN;
        let result = lerp(0.0, nan, 0.5);
        assert!(result.is_nan(), "NaN should propagate through lerp");
    }

    #[test]
    fn precision_zero_division_in_normalize() {
        let zero = Vec2::ZERO;
        let n = zero.normalize();
        // Our implementation should return ZERO, not NaN
        assert!(!n.x.is_nan(), "Zero normalize should not produce NaN");
        assert!(!n.y.is_nan(), "Zero normalize should not produce NaN");
    }

    // -------------------------------------------------------------------------
    // Accumulated Error
    // -------------------------------------------------------------------------

    #[test]
    fn precision_accumulated_rotation_error() {
        // Demonstrate that repeated rotation accumulates error
        let mut v = Vec2::new(1.0, 0.0);
        let step = PI / 180.0; // 1 degree

        for _ in 0..360 {
            v = v.rotate(step);
        }

        // After 360 degrees, should be close to original (but has error)
        let error_rotated = (v - Vec2::new(1.0, 0.0)).length();
        assert!(
            error_rotated < 0.01,
            "Accumulated rotation error too large: {error_rotated}"
        );

        // Compare with the proper approach: Vec2::from_angle
        let total_angle = step * 360.0;
        let from_angle = Vec2::from_angle(total_angle);
        let error_from_angle = (from_angle - Vec2::new(1.0, 0.0)).length();

        // from_angle should be significantly more accurate (at least 10x better)
        assert!(
            error_from_angle < error_rotated * 0.1,
            "from_angle ({error_from_angle}) should be much more accurate than rotate \
             ({error_rotated})"
        );
    }

    #[test]
    fn precision_accumulated_lerp_error() {
        // Repeated lerping should maintain precision
        let mut value = 0.0_f32;
        let target = 1.0_f32;

        // 1000 small steps toward target
        for _ in 0..1000 {
            value = lerp(value, target, 0.01);
        }

        // Should be very close to 1.0 (but not exactly due to asymptotic approach)
        assert!(
            (value - target).abs() < 0.001,
            "Accumulated lerp: expected ~1.0, got {value}"
        );
    }

    #[test]
    fn precision_accumulated_addition() {
        // Sum many small values
        let small = 0.1_f32;
        let mut sum = 0.0_f32;
        for _ in 0..10 {
            sum += small;
        }

        // 0.1 * 10 should equal 1.0, but floating point disagrees
        let error = (sum - 1.0).abs();
        assert!(
            error < 1e-6,
            "10 * 0.1 should be ~1.0, got {sum} (error: {error})"
        );
    }

    // -------------------------------------------------------------------------
    // Trigonometric Precision
    // -------------------------------------------------------------------------

    #[test]
    fn precision_rotation_cardinal_angles() {
        let v = Vec2::new(1.0, 0.0);

        // 90 degrees
        let rot90 = v.rotate(PI / 2.0);
        assert!(
            (rot90.x).abs() < PHYSICS_EPSILON,
            "90° rotation x should be 0, got {}",
            rot90.x
        );
        assert!(
            (rot90.y - 1.0).abs() < PHYSICS_EPSILON,
            "90° rotation y should be 1, got {}",
            rot90.y
        );

        // 180 degrees
        let rot180 = v.rotate(PI);
        assert!(
            (rot180.x + 1.0).abs() < PHYSICS_EPSILON,
            "180° rotation x should be -1, got {}",
            rot180.x
        );
        assert!(
            (rot180.y).abs() < PHYSICS_EPSILON,
            "180° rotation y should be 0, got {}",
            rot180.y
        );

        // 270 degrees
        let rot270 = v.rotate(3.0 * PI / 2.0);
        assert!(
            (rot270.x).abs() < PHYSICS_EPSILON,
            "270° rotation x should be 0, got {}",
            rot270.x
        );
        assert!(
            (rot270.y + 1.0).abs() < PHYSICS_EPSILON,
            "270° rotation y should be -1, got {}",
            rot270.y
        );
    }

    #[test]
    fn precision_angle_calculation() {
        // Test angle() method for cardinal directions
        assert!((Vec2::RIGHT.angle() - 0.0).abs() < PHYSICS_EPSILON);
        assert!((Vec2::UP.angle() - PI / 2.0).abs() < PHYSICS_EPSILON);
        assert!((Vec2::DOWN.angle() + PI / 2.0).abs() < PHYSICS_EPSILON);
        // LEFT angle is PI or -PI depending on implementation
        let left_angle = Vec2::LEFT.angle();
        assert!(
            (left_angle.abs() - PI).abs() < PHYSICS_EPSILON,
            "LEFT angle should be ±π, got {left_angle}"
        );
    }

    #[test]
    fn precision_sin_cos_identity() {
        // sin²(θ) + cos²(θ) = 1 for any angle
        for i in 0..36 {
            let angle = (i as f32) * PI / 18.0; // 0 to 2π in 10° steps
            let sin_sq = angle.sin().powi(2);
            let cos_sq = angle.cos().powi(2);
            let sum = sin_sq + cos_sq;
            assert!(
                (sum - 1.0).abs() < F32_EPSILON * 10.0,
                "sin²+cos² should be 1 at {}°, got {}",
                i * 10,
                sum
            );
        }
    }

    // -------------------------------------------------------------------------
    // Geometric Precision
    // -------------------------------------------------------------------------

    #[test]
    fn precision_dot_product_orthogonal() {
        // Perfectly orthogonal vectors should have zero dot product
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(a.dot(b), 0.0, "Orthogonal dot product should be exactly 0");
    }

    #[test]
    fn precision_cross_product_parallel() {
        // Parallel vectors should have zero cross product
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(6.0, 8.0); // 2 * a
        assert!(
            a.cross(b).abs() < PHYSICS_EPSILON,
            "Parallel cross product should be ~0, got {}",
            a.cross(b)
        );
    }

    #[test]
    fn precision_normalize_unit_vectors() {
        // Normalizing a unit vector should stay unit
        let unit = Vec2::new(0.6, 0.8); // Already unit length
        let normalized = unit.normalize();
        assert!(
            (normalized.length() - 1.0).abs() < PHYSICS_EPSILON,
            "Normalized unit vector should stay unit"
        );
    }

    #[test]
    fn precision_distance_symmetry() {
        let a = Vec2::new(1.234567, 8.901234);
        let b = Vec2::new(5.678901, 2.345678);

        let d1 = a.distance(b);
        let d2 = b.distance(a);

        assert_eq!(d1, d2, "Distance should be symmetric");
    }

    // -------------------------------------------------------------------------
    // Platform Consistency Tests
    // -------------------------------------------------------------------------

    #[test]
    fn platform_f32_properties() {
        // Verify f32 properties are consistent across platforms
        assert_eq!(f32::RADIX, 2, "f32 should use base 2");
        assert_eq!(f32::MANTISSA_DIGITS, 24, "f32 should have 24 mantissa bits");
        assert_eq!(f32::MAX_EXP, 128, "f32 max exponent should be 128");
        assert_eq!(f32::MIN_EXP, -125, "f32 min exponent should be -125");
    }

    #[test]
    fn platform_byte_order() {
        // Test that our bit manipulation works correctly
        let val: f32 = 1.0;
        let bits = val.to_bits();
        let restored = f32::from_bits(bits);
        assert_eq!(
            val, restored,
            "to_bits/from_bits round-trip should preserve value"
        );
    }

    #[test]
    fn platform_ieee754_special_values() {
        // Verify IEEE 754 special value handling
        assert!(f32::NAN.is_nan());
        assert!(f32::INFINITY.is_infinite());
        assert!(f32::NEG_INFINITY.is_infinite());
        assert!(0.0_f32.is_finite());
        assert!(f32::MIN_POSITIVE.is_normal());
        assert!((f32::MIN_POSITIVE / 2.0).is_subnormal());
    }

    #[test]
    fn platform_rounding_mode() {
        // Rust's f32::round() uses "round half away from zero" (not banker's rounding)
        let a = 0.5_f32;
        let b = 1.5_f32;
        let c = 2.5_f32;

        // Document actual Rust rounding behavior
        assert_eq!(a.round(), 1.0, "0.5 rounds to 1 (away from zero)");
        assert_eq!(b.round(), 2.0, "1.5 rounds to 2");
        assert_eq!(c.round(), 3.0, "2.5 rounds to 3 (away from zero)");
    }

    #[test]
    fn platform_deterministic_sqrt() {
        // sqrt should produce identical results
        let values: [f32; 9] = [1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 100.0, 0.5, 0.25];
        for &v in &values {
            let sqrt1 = v.sqrt();
            let sqrt2 = v.sqrt();
            assert_eq!(
                sqrt1.to_bits(),
                sqrt2.to_bits(),
                "sqrt({v}) should be deterministic"
            );
        }
    }

    #[test]
    fn platform_deterministic_trig() {
        // Trigonometric functions should be deterministic
        let angles = [0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI];
        for &a in &angles {
            let sin1 = a.sin();
            let sin2 = a.sin();
            let cos1 = a.cos();
            let cos2 = a.cos();

            assert_eq!(
                sin1.to_bits(),
                sin2.to_bits(),
                "sin({a}) should be deterministic"
            );
            assert_eq!(
                cos1.to_bits(),
                cos2.to_bits(),
                "cos({a}) should be deterministic"
            );
        }
    }
}
