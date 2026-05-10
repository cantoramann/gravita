// math/src/vector2.rs

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::vector::Vector;

/// Simple 2D vector type used throughout the engine.
///
/// This is intentionally minimal and dependency-free, but still supports
/// the most common vector operations needed for physics and gameplay code.

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec2 {
    /// X component of the vector.
    pub x: f32,
    /// Y component of the vector.
    pub y: f32,
}

impl Vec2 {
    /// Zero vector `(0, 0)`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// One vector `(1, 1)`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    /// Unit vector pointing up `(0, 1)`.
    pub const UP: Self = Self { x: 0.0, y: 1.0 };
    /// Unit vector pointing down `(0, -1)`.
    pub const DOWN: Self = Self { x: 0.0, y: -1.0 };
    /// Unit vector pointing left `(-1, 0)`.
    pub const LEFT: Self = Self { x: -1.0, y: 0.0 };
    /// Unit vector pointing right `(1, 0)`.
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0 };

    #[inline]
    /// Construct a new vector from components.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    /// Dot product between two vectors.
    pub fn dot(&self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }

    #[inline]
    /// 2D "cross product" returning the scalar z-component.
    pub fn cross(&self, other: Self) -> f32 {
        // 2D cross product returns a scalar (z-component of 3D cross)
        self.x.mul_add(other.y, -(self.y * other.x))
    }

    #[inline]
    /// Euclidean length (magnitude) of the vector.
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    /// Squared length of the vector (avoids a square root).
    pub fn length_squared(&self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }

    #[inline]
    /// Return a normalized (unit length) copy of this vector.
    ///
    /// If the vector is very close to zero, this returns [`Vec2::ZERO`].
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 { *self / len } else { Self::ZERO }
    }

    #[inline]
    /// Distance between this point and another.
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    #[inline]
    /// Squared distance between this point and another.
    pub fn distance_squared(&self, other: Self) -> f32 {
        (*self - other).length_squared()
    }

    #[inline]
    /// Linearly interpolate towards `other` by factor `t` in \[0, 1\].
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        *self + (other - *self) * t
    }

    #[inline]
    /// Reflect this vector around the given normal.
    pub fn reflect(&self, normal: Self) -> Self {
        *self - normal * 2.0 * self.dot(normal)
    }

    #[inline]
    /// Create a unit vector pointing in the given direction (angle in radians).
    ///
    /// This is the **preferred way** to get a direction vector when you have an angle,
    /// as it avoids accumulated floating-point error from repeated rotations.
    ///
    /// # Example
    ///
    /// ```
    /// use std::f32::consts::PI;
    ///
    /// use gravita_math::Vec2;
    ///
    /// // Instead of rotating a vector 360 times:
    /// // let mut dir = Vec2::RIGHT;
    /// // for _ in 0..360 { dir = dir.rotate(PI / 180.0); }  // Accumulates error!
    ///
    /// // Prefer storing angle and computing direction on demand:
    /// let angle = PI / 4.0; // 45 degrees
    /// let dir = Vec2::from_angle(angle);
    /// ```
    pub fn from_angle(angle: f32) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    #[inline]
    /// Rotate this vector by `angle` radians around the origin.
    ///
    /// **Note:** Repeated rotations accumulate floating-point error. For objects
    /// that rotate continuously, prefer storing the angle as a scalar and using
    /// [`Vec2::from_angle`] to compute the direction when needed.
    pub fn rotate(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(
            self.x.mul_add(cos, -(self.y * sin)),
            self.x.mul_add(sin, self.y * cos),
        )
    }

    #[inline]
    /// Get a perpendicular vector (rotated 90° counter-clockwise).
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    #[inline]
    /// Angle of this vector in radians, measured from the +X axis.
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    #[inline]
    /// Signed smallest angle between this vector and `other` in radians.
    pub fn angle_between(&self, other: Self) -> f32 {
        let dot = self.dot(other);
        let det = self.cross(other);
        det.atan2(dot)
    }

    #[inline]
    /// Clamp each component between the corresponding components of `min` and `max`.
    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }
}

impl Vector for Vec2 {
    const ZERO: Self = Self::ZERO;

    fn dot(self, other: Self) -> f32 {
        Self::dot(&self, other)
    }
    fn length_squared(self) -> f32 {
        Self::length_squared(&self)
    }
}

// Operator implementations
impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, vec: Vec2) -> Vec2 {
        Vec2::new(vec.x * self, vec.y * self)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

// Assignment operators
impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
    }
}

// Convenience conversions
impl From<(f32, f32)> for Vec2 {
    fn from(tuple: (f32, f32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(array: [f32; 2]) -> Self {
        Self::new(array[0], array[1])
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // =========================================================================
    // Construction and Constants
    // =========================================================================

    #[test]
    fn construction_from_components() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn construction_from_tuple() {
        let v: Vec2 = (3.0, 4.0).into();
        assert_eq!(v, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn construction_from_array() {
        let v: Vec2 = [3.0, 4.0].into();
        assert_eq!(v, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn constants_are_correct() {
        assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
        assert_eq!(Vec2::ONE, Vec2::new(1.0, 1.0));
        assert_eq!(Vec2::UP, Vec2::new(0.0, 1.0));
        assert_eq!(Vec2::DOWN, Vec2::new(0.0, -1.0));
        assert_eq!(Vec2::LEFT, Vec2::new(-1.0, 0.0));
        assert_eq!(Vec2::RIGHT, Vec2::new(1.0, 0.0));
    }

    // =========================================================================
    // Arithmetic Operations
    // =========================================================================

    #[test]
    fn addition_of_two_vectors() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn subtraction_of_two_vectors() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        assert_eq!(a - b, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn scalar_multiplication_right() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v * 2.0, Vec2::new(6.0, 8.0));
    }

    #[test]
    fn scalar_multiplication_left() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(2.0 * v, Vec2::new(6.0, 8.0));
    }

    #[test]
    fn scalar_division() {
        let v = Vec2::new(6.0, 8.0);
        assert_eq!(v / 2.0, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn negation() {
        let v = Vec2::new(3.0, -4.0);
        assert_eq!(-v, Vec2::new(-3.0, 4.0));
    }

    #[test]
    fn add_assign() {
        let mut v = Vec2::new(1.0, 2.0);
        v += Vec2::new(3.0, 4.0);
        assert_eq!(v, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn sub_assign() {
        let mut v = Vec2::new(5.0, 6.0);
        v -= Vec2::new(3.0, 4.0);
        assert_eq!(v, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn mul_assign() {
        let mut v = Vec2::new(3.0, 4.0);
        v *= 2.0;
        assert_eq!(v, Vec2::new(6.0, 8.0));
    }

    #[test]
    fn div_assign() {
        let mut v = Vec2::new(6.0, 8.0);
        v /= 2.0;
        assert_eq!(v, Vec2::new(3.0, 4.0));
    }

    // =========================================================================
    // Length and Distance
    // =========================================================================

    #[test]
    fn length_of_345_triangle() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn length_squared_avoids_sqrt() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length_squared(), 25.0);
    }

    #[test]
    fn length_of_zero_vector() {
        assert_eq!(Vec2::ZERO.length(), 0.0);
    }

    #[test]
    fn distance_between_two_points() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.distance(b), 5.0);
    }

    #[test]
    fn distance_squared_between_points() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.distance_squared(b), 25.0);
    }

    // =========================================================================
    // Normalization
    // =========================================================================

    #[test]
    fn normalize_produces_unit_length() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!(approx_eq(n.length(), 1.0));
    }

    #[test]
    fn normalize_preserves_direction() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!(approx_eq(n.x, 0.6));
        assert!(approx_eq(n.y, 0.8));
    }

    #[test]
    fn normalize_zero_vector_returns_zero() {
        let n = Vec2::ZERO.normalize();
        assert_eq!(n, Vec2::ZERO);
    }

    // =========================================================================
    // Dot and Cross Products
    // =========================================================================

    #[test]
    fn dot_product_of_orthogonal_vectors_is_zero() {
        let a = Vec2::RIGHT;
        let b = Vec2::UP;
        assert_eq!(a.dot(b), 0.0);
    }

    #[test]
    fn dot_product_of_parallel_vectors() {
        let a = Vec2::new(2.0, 0.0);
        let b = Vec2::new(3.0, 0.0);
        assert_eq!(a.dot(b), 6.0);
    }

    #[test]
    fn dot_product_of_opposite_vectors_is_negative() {
        let a = Vec2::RIGHT;
        let b = Vec2::LEFT;
        assert_eq!(a.dot(b), -1.0);
    }

    #[test]
    fn cross_product_2d_returns_scalar() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        // Cross product of right-hand system: i × j = k (positive)
        assert_eq!(a.cross(b), 1.0);
    }

    #[test]
    fn cross_product_reversed_is_negative() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(b.cross(a), -1.0);
    }

    #[test]
    fn cross_product_of_parallel_vectors_is_zero() {
        let a = Vec2::new(2.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        assert_eq!(a.cross(b), 0.0);
    }

    // =========================================================================
    // Rotation and Angles
    // =========================================================================

    #[test]
    fn rotate_90_degrees_counterclockwise() {
        let v = Vec2::RIGHT;
        let rotated = v.rotate(FRAC_PI_2);
        assert!(vec_approx_eq(rotated, Vec2::UP));
    }

    #[test]
    fn rotate_180_degrees() {
        let v = Vec2::RIGHT;
        let rotated = v.rotate(PI);
        assert!(vec_approx_eq(rotated, Vec2::LEFT));
    }

    #[test]
    fn rotate_45_degrees() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = v.rotate(FRAC_PI_4);
        let expected = Vec2::new(0.5_f32.sqrt(), 0.5_f32.sqrt());
        assert!(vec_approx_eq(rotated, expected));
    }

    // =========================================================================
    // from_angle (Error-Free Direction Construction)
    // =========================================================================

    #[test]
    fn from_angle_zero_is_right() {
        let v = Vec2::from_angle(0.0);
        assert!(vec_approx_eq(v, Vec2::RIGHT));
    }

    #[test]
    fn from_angle_90_degrees_is_up() {
        let v = Vec2::from_angle(FRAC_PI_2);
        assert!(vec_approx_eq(v, Vec2::UP));
    }

    #[test]
    fn from_angle_180_degrees_is_left() {
        let v = Vec2::from_angle(PI);
        assert!(vec_approx_eq(v, Vec2::LEFT));
    }

    #[test]
    fn from_angle_270_degrees_is_down() {
        let v = Vec2::from_angle(3.0 * FRAC_PI_2);
        assert!(vec_approx_eq(v, Vec2::DOWN));
    }

    #[test]
    fn from_angle_produces_unit_vector() {
        // Test various angles
        for i in 0..36 {
            let angle = (i as f32) * PI / 18.0; // 0 to 2π in 10° steps
            let v = Vec2::from_angle(angle);
            assert!(
                approx_eq(v.length(), 1.0),
                "from_angle({}) should produce unit vector, got length {}",
                angle,
                v.length()
            );
        }
    }

    #[test]
    fn from_angle_roundtrip_with_angle() {
        // from_angle(θ).angle() should return θ (for θ in (-π, π))
        // Note: ±π represent the same direction (LEFT), so we avoid exact ±π
        let test_angles = [
            0.0,
            FRAC_PI_4,
            FRAC_PI_2,
            PI * 0.75,
            -FRAC_PI_4,
            -FRAC_PI_2,
            -PI * 0.75,
        ];
        for &angle in &test_angles {
            let v = Vec2::from_angle(angle);
            let recovered = v.angle();
            assert!(
                approx_eq(angle, recovered),
                "from_angle({angle}).angle() = {recovered}, expected {angle}"
            );
        }
    }

    #[test]
    fn from_angle_no_accumulated_error() {
        // Key test: from_angle avoids accumulated rotation error
        // Compare: rotating 360 times vs computing direction from accumulated angle

        let step = PI / 180.0; // 1 degree

        // Method 1: Accumulate rotations (has error)
        let mut rotated = Vec2::RIGHT;
        for _ in 0..360 {
            rotated = rotated.rotate(step);
        }

        // Method 2: Accumulate angle, compute direction once (no error)
        let accumulated_angle = step * 360.0; // = 2π
        let from_angle_result = Vec2::from_angle(accumulated_angle);

        // from_angle should be much closer to the expected result
        let expected = Vec2::RIGHT; // After 360°, should be back to RIGHT

        let error_rotated = (rotated - expected).length();
        let error_from_angle = (from_angle_result - expected).length();

        // from_angle error should be orders of magnitude smaller
        assert!(
            error_from_angle < error_rotated * 0.1,
            "from_angle error ({error_from_angle}) should be much smaller than rotation error \
             ({error_rotated})"
        );

        // from_angle should be very close to expected
        assert!(
            error_from_angle < 1e-5,
            "from_angle after 360° should have negligible error, got {error_from_angle}"
        );
    }

    #[test]
    fn angle_of_right_vector() {
        assert!(approx_eq(Vec2::RIGHT.angle(), 0.0));
    }

    #[test]
    fn angle_of_up_vector() {
        assert!(approx_eq(Vec2::UP.angle(), FRAC_PI_2));
    }

    #[test]
    fn angle_between_orthogonal_vectors() {
        let a = Vec2::RIGHT;
        let b = Vec2::UP;
        assert!(approx_eq(a.angle_between(b), FRAC_PI_2));
    }

    #[test]
    fn perpendicular_is_90_degrees_ccw() {
        let v = Vec2::RIGHT;
        let perp = v.perpendicular();
        assert_eq!(perp, Vec2::UP);
    }

    // =========================================================================
    // Reflection and Interpolation
    // =========================================================================

    #[test]
    fn reflect_horizontal_off_vertical_surface() {
        let incident = Vec2::new(1.0, -1.0).normalize();
        let normal = Vec2::UP;
        let reflected = incident.reflect(normal);
        let expected = Vec2::new(1.0, 1.0).normalize();
        assert!(vec_approx_eq(reflected, expected));
    }

    #[test]
    fn lerp_at_zero_returns_start() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 10.0);
        assert_eq!(a.lerp(b, 0.0), a);
    }

    #[test]
    fn lerp_at_one_returns_end() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 10.0);
        assert_eq!(a.lerp(b, 1.0), b);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 10.0);
        assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 5.0));
    }

    // =========================================================================
    // Clamping
    // =========================================================================

    #[test]
    fn clamp_within_bounds_unchanged() {
        let v = Vec2::new(5.0, 5.0);
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(10.0, 10.0);
        assert_eq!(v.clamp(min, max), v);
    }

    #[test]
    fn clamp_below_min_returns_min() {
        let v = Vec2::new(-5.0, -5.0);
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(10.0, 10.0);
        assert_eq!(v.clamp(min, max), min);
    }

    #[test]
    fn clamp_above_max_returns_max() {
        let v = Vec2::new(15.0, 15.0);
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(10.0, 10.0);
        assert_eq!(v.clamp(min, max), max);
    }
}
