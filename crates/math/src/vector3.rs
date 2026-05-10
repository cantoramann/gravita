// math/src/vector3.rs

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::vector::Vector;

/// 3D vector with `f32` components.
///
/// Mirrors the [`Vec2`](crate::Vec2) API. For dimension-agnostic code,
/// implement against the [`Vector`] trait instead of concrete types.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vec3 {
    /// Zero vector `(0, 0, 0)`.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    /// One vector `(1, 1, 1)`.
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    /// Unit X `(1, 0, 0)`.
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    /// Unit Y `(0, 1, 0)`.
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    /// Unit Z `(0, 0, 1)`.
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    /// Up `(0, 1, 0)`.
    pub const UP: Self = Self::Y;
    /// Down `(0, -1, 0)`.
    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    /// Left `(-1, 0, 0)`.
    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    /// Right `(1, 0, 0)`.
    pub const RIGHT: Self = Self::X;
    /// Forward in right-handed Y-up: `(0, 0, -1)`.
    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };
    /// Back in right-handed Y-up: `(0, 0, 1)`.
    pub const BACK: Self = Self::Z;

    /// Construct a new vector from components.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// All components set to the same value.
    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// Dot product.
    #[inline]
    pub fn dot(&self, other: Self) -> f32 {
        self.x
            .mul_add(other.x, self.y.mul_add(other.y, self.z * other.z))
    }

    /// 3D cross product (right-handed).
    #[inline]
    pub fn cross(&self, other: Self) -> Self {
        Self::new(
            self.y.mul_add(other.z, -(self.z * other.y)),
            self.z.mul_add(other.x, -(self.x * other.z)),
            self.x.mul_add(other.y, -(self.y * other.x)),
        )
    }

    /// Squared length.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.dot(*self)
    }

    /// Euclidean length.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Unit-length copy. Returns [`Vec3::ZERO`] for the zero vector.
    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 { *self / len } else { Self::ZERO }
    }

    /// Euclidean distance to another point.
    #[inline]
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    /// Squared distance to another point.
    #[inline]
    pub fn distance_squared(&self, other: Self) -> f32 {
        (*self - other).length_squared()
    }

    /// Linear interpolation towards `other` by factor `t`.
    #[inline]
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        *self + (other - *self) * t
    }

    /// Reflect across `normal` (must be unit length).
    #[inline]
    pub fn reflect(&self, normal: Self) -> Self {
        *self - normal * (2.0 * self.dot(normal))
    }

    /// Per-component clamp.
    #[inline]
    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
            self.z.clamp(min.z, max.z),
        )
    }

    /// Per-component minimum.
    #[inline]
    pub fn min(&self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    /// Per-component maximum.
    #[inline]
    pub fn max(&self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    /// Project this vector onto `axis` (does not assume unit length).
    #[inline]
    pub fn project_onto(&self, axis: Self) -> Self {
        let axis_len_sq = axis.length_squared();
        if axis_len_sq == 0.0 {
            return Self::ZERO;
        }
        axis * (self.dot(axis) / axis_len_sq)
    }

    /// Component of this vector perpendicular to `axis`.
    #[inline]
    pub fn reject_from(&self, axis: Self) -> Self {
        *self - self.project_onto(axis)
    }
}

impl Vector for Vec3 {
    const ZERO: Self = Self::ZERO;

    fn dot(self, other: Self) -> f32 {
        Self::dot(&self, other)
    }
    fn length_squared(self) -> f32 {
        Self::length_squared(&self)
    }
}

// Operator implementations
impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline]
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl MulAssign<f32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl DivAssign<f32> for Vec3 {
    #[inline]
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(t: (f32, f32, f32)) -> Self {
        Self::new(t.0, t.1, t.2)
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(a: [f32; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    // -------------------------------------------------------------------------
    // Construction & constants
    // -------------------------------------------------------------------------

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec3::ONE, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(Vec3::X, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(Vec3::Y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(Vec3::Z, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(Vec3::UP, Vec3::Y);
        assert_eq!(Vec3::RIGHT, Vec3::X);
        assert_eq!(Vec3::FORWARD, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn splat_replicates_value() {
        assert_eq!(Vec3::splat(3.0), Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn from_tuple_and_array_round_trip() {
        let from_t: Vec3 = (1.0, 2.0, 3.0).into();
        let from_a: Vec3 = [1.0, 2.0, 3.0].into();
        assert_eq!(from_t, from_a);
        let back: [f32; 3] = from_t.into();
        assert_eq!(back, [1.0, 2.0, 3.0]);
    }

    // -------------------------------------------------------------------------
    // Arithmetic
    // -------------------------------------------------------------------------

    #[test]
    fn addition_subtraction() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn scalar_mul_both_sides() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * v, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn scalar_div() {
        assert_eq!(Vec3::new(2.0, 4.0, 6.0) / 2.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn negation() {
        assert_eq!(-Vec3::new(1.0, -2.0, 3.0), Vec3::new(-1.0, 2.0, -3.0));
    }

    #[test]
    fn assign_operators() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        v += Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v, Vec3::new(2.0, 3.0, 4.0));
        v -= Vec3::ONE;
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
        v *= 2.0;
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
        v /= 2.0;
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    // -------------------------------------------------------------------------
    // Geometry
    // -------------------------------------------------------------------------

    #[test]
    fn dot_basis_vectors() {
        assert_eq!(Vec3::X.dot(Vec3::Y), 0.0);
        assert_eq!(Vec3::X.dot(Vec3::X), 1.0);
        assert_eq!(Vec3::X.dot(-Vec3::X), -1.0);
    }

    #[test]
    fn cross_right_handed() {
        // i × j = k
        assert!(vec_approx(Vec3::X.cross(Vec3::Y), Vec3::Z));
        // j × k = i
        assert!(vec_approx(Vec3::Y.cross(Vec3::Z), Vec3::X));
        // k × i = j
        assert!(vec_approx(Vec3::Z.cross(Vec3::X), Vec3::Y));
    }

    #[test]
    fn cross_is_anticommutative() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!(vec_approx(a.cross(b), -b.cross(a)));
    }

    #[test]
    fn cross_of_parallel_is_zero() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = a * 5.0;
        assert!(vec_approx(a.cross(b), Vec3::ZERO));
    }

    #[test]
    fn length_345_triangle() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!(approx(v.length(), 5.0));
        let v = Vec3::new(2.0, 3.0, 6.0);
        // sqrt(4 + 9 + 36) = sqrt(49) = 7
        assert!(approx(v.length(), 7.0));
    }

    #[test]
    fn normalize_produces_unit_length() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert!(approx(v.normalize().length(), 1.0));
    }

    #[test]
    fn normalize_zero_returns_zero() {
        assert_eq!(Vec3::ZERO.normalize(), Vec3::ZERO);
    }

    #[test]
    fn distance_symmetry() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!(approx(a.distance(b), b.distance(a)));
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        let a = Vec3::ZERO;
        let b = Vec3::new(10.0, 20.0, -10.0);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, -5.0));
    }

    #[test]
    fn reflect_off_y_normal_flips_y() {
        let incident = Vec3::new(1.0, -1.0, 0.0);
        let reflected = incident.reflect(Vec3::Y);
        assert!(vec_approx(reflected, Vec3::new(1.0, 1.0, 0.0)));
    }

    #[test]
    fn clamp_per_component() {
        let v = Vec3::new(-5.0, 5.0, 15.0);
        let min = Vec3::splat(0.0);
        let max = Vec3::splat(10.0);
        assert_eq!(v.clamp(min, max), Vec3::new(0.0, 5.0, 10.0));
    }

    #[test]
    fn min_max_per_component() {
        let a = Vec3::new(1.0, 4.0, 5.0);
        let b = Vec3::new(3.0, 2.0, 5.0);
        assert_eq!(a.min(b), Vec3::new(1.0, 2.0, 5.0));
        assert_eq!(a.max(b), Vec3::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn projection_onto_axis() {
        // Projecting (3, 4, 0) onto X yields (3, 0, 0).
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!(vec_approx(
            v.project_onto(Vec3::X),
            Vec3::new(3.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn rejection_is_perpendicular_to_axis() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let r = v.reject_from(Vec3::X);
        assert!(approx(r.dot(Vec3::X), 0.0));
    }

    #[test]
    fn vector_trait_default_methods_compose() {
        // length() / normalize() should agree with the inherent impls.
        let v = Vec3::new(1.0, 2.0, 2.0); // length = 3
        let via_trait: f32 = <Vec3 as Vector>::length(v);
        assert!(approx(via_trait, 3.0));
        let n = <Vec3 as Vector>::normalize(v);
        assert!(approx(n.length(), 1.0));
    }
}
