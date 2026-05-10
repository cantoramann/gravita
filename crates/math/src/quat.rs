// math/src/quat.rs

//! Unit quaternions for 3D rotation.
//!
//! Stored as `(x, y, z, w)` with `w` the scalar component. Most operations
//! assume `self` is unit length — use [`Quat::normalize`] after composing many
//! rotations to keep drift bounded.

use std::ops::Mul;

use crate::vector3::Vec3;

/// Unit quaternion: `xi + yj + zk + w`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quat {
    /// Vector i component.
    pub x: f32,
    /// Vector j component.
    pub y: f32,
    /// Vector k component.
    pub z: f32,
    /// Scalar component.
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    /// Identity rotation `(0, 0, 0, 1)`.
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// Construct from raw components (caller is responsible for unit length).
    #[inline]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Build a rotation of `angle` radians around `axis`. The axis must be
    /// unit length for the result to be a valid rotation.
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half = angle * 0.5;
        let s = half.sin();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: half.cos(),
        }
    }

    /// Build from intrinsic Tait–Bryan angles: yaw around `Y`, pitch around `X`,
    /// roll around `Z`. Applied right-to-left, i.e. yaw first, then pitch,
    /// then roll on the resulting orientation.
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        let qy = Self::from_axis_angle(Vec3::Y, yaw);
        let qx = Self::from_axis_angle(Vec3::X, pitch);
        let qz = Self::from_axis_angle(Vec3::Z, roll);
        qz * qx * qy
    }

    /// Squared length of the quaternion `(x²+y²+z²+w²)`.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x.mul_add(
            self.x,
            self.y
                .mul_add(self.y, self.z.mul_add(self.z, self.w * self.w)),
        )
    }

    /// Euclidean length.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Re-normalize. Numerical drift accumulates after long chains of
    /// multiplications — call this periodically.
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::from_xyzw(self.x / len, self.y / len, self.z / len, self.w / len)
        } else {
            Self::IDENTITY
        }
    }

    /// Conjugate `(−x, −y, −z, w)`. For unit quaternions this is the inverse.
    #[inline]
    pub fn conjugate(&self) -> Self {
        Self::from_xyzw(-self.x, -self.y, -self.z, self.w)
    }

    /// Inverse rotation. For unit quaternions equals the conjugate.
    pub fn inverse(&self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::IDENTITY;
        }
        let inv = 1.0 / len_sq;
        Self::from_xyzw(-self.x * inv, -self.y * inv, -self.z * inv, self.w * inv)
    }

    /// Rotate `v` by this quaternion. Uses the standard `q * (0,v) * q⁻¹`
    /// expansion, simplified.
    pub fn rotate_vec(&self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let s = self.w;
        // v' = 2 * dot(u, v) * u + (s² − u·u) * v + 2 * s * cross(u, v)
        let two_dot = 2.0 * u.dot(v);
        let s_sq_minus_uu = s.mul_add(s, -u.dot(u));
        u * two_dot + v * s_sq_minus_uu + u.cross(v) * (2.0 * s)
    }

    /// Build the shortest-arc rotation that takes unit vector `from` onto
    /// unit vector `to`. Returns identity if `from == to`.
    pub fn from_rotation_arc(from: Vec3, to: Vec3) -> Self {
        let d = from.dot(to);
        if d >= 1.0 - 1e-6 {
            return Self::IDENTITY;
        }
        if d <= -1.0 + 1e-6 {
            // 180° rotation — pick any axis orthogonal to `from`.
            let axis = if from.x.abs() > from.z.abs() {
                Vec3::new(-from.y, from.x, 0.0)
            } else {
                Vec3::new(0.0, -from.z, from.y)
            }
            .normalize();
            return Self::from_axis_angle(axis, std::f32::consts::PI);
        }
        let s = ((1.0 + d) * 2.0).sqrt();
        let c = from.cross(to) * (1.0 / s);
        Self::from_xyzw(c.x, c.y, c.z, s * 0.5).normalize()
    }
}

impl Mul for Quat {
    type Output = Self;

    /// Hamilton product: `self * other` applies `other` then `self`.
    #[allow(clippy::suboptimal_flops)] // Hamilton product reads more naturally as 4 multiply-add chains.
    fn mul(self, rhs: Self) -> Self {
        Self::from_xyzw(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.rotate_vec(v)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    #[test]
    fn identity_rotation_is_identity() {
        let q = Quat::IDENTITY;
        assert!(vec_approx(q.rotate_vec(Vec3::X), Vec3::X));
        assert!(vec_approx(q.rotate_vec(Vec3::Y), Vec3::Y));
        assert!(vec_approx(q.rotate_vec(Vec3::Z), Vec3::Z));
    }

    #[test]
    fn identity_has_unit_length() {
        assert!(approx(Quat::IDENTITY.length(), 1.0));
    }

    #[test]
    fn axis_angle_90_around_z_rotates_x_to_y() {
        let q = Quat::from_axis_angle(Vec3::Z, FRAC_PI_2);
        let r = q.rotate_vec(Vec3::X);
        assert!(vec_approx(r, Vec3::Y), "got {r:?}");
    }

    #[test]
    fn axis_angle_180_around_y_flips_x() {
        let q = Quat::from_axis_angle(Vec3::Y, PI);
        assert!(vec_approx(q.rotate_vec(Vec3::X), -Vec3::X));
    }

    #[test]
    fn axis_angle_returns_unit_quaternion() {
        let q = Quat::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalize(), 1.234);
        assert!(approx(q.length(), 1.0));
    }

    #[test]
    fn composition_yields_same_as_sequential_rotations() {
        // Rotate +90° around Z then +90° around X.
        let qz = Quat::from_axis_angle(Vec3::Z, FRAC_PI_2);
        let qx = Quat::from_axis_angle(Vec3::X, FRAC_PI_2);
        let combined = qx * qz;

        let v = Vec3::X;
        let sequential = qx.rotate_vec(qz.rotate_vec(v));
        let composed = combined.rotate_vec(v);
        assert!(
            vec_approx(sequential, composed),
            "sequential {sequential:?} != composed {composed:?}"
        );
    }

    #[test]
    fn inverse_undoes_rotation() {
        let q = Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalize(), 1.234);
        let v = Vec3::new(0.5, -2.0, 7.3);
        let rotated = q.rotate_vec(v);
        let restored = q.inverse().rotate_vec(rotated);
        assert!(vec_approx(restored, v), "got {restored:?} expected {v:?}");
    }

    #[test]
    fn conjugate_equals_inverse_for_unit() {
        let q = Quat::from_axis_angle(Vec3::Z, FRAC_PI_4);
        let c = q.conjugate();
        let i = q.inverse();
        assert!(approx(c.x, i.x) && approx(c.y, i.y) && approx(c.z, i.z) && approx(c.w, i.w));
    }

    #[test]
    fn rotation_arc_x_to_y() {
        let q = Quat::from_rotation_arc(Vec3::X, Vec3::Y);
        assert!(vec_approx(q.rotate_vec(Vec3::X), Vec3::Y));
    }

    #[test]
    fn rotation_arc_identity_when_same() {
        let q = Quat::from_rotation_arc(Vec3::X, Vec3::X);
        assert!(vec_approx(q.rotate_vec(Vec3::Y), Vec3::Y));
    }

    #[test]
    fn rotation_arc_opposite_finds_180() {
        let q = Quat::from_rotation_arc(Vec3::X, -Vec3::X);
        assert!(vec_approx(q.rotate_vec(Vec3::X), -Vec3::X));
    }

    #[test]
    fn euler_yaw_90_rotates_x_to_neg_z() {
        // Yaw of +90° around Y (right-handed) should take +X to -Z.
        let q = Quat::from_euler(FRAC_PI_2, 0.0, 0.0);
        let r = q.rotate_vec(Vec3::X);
        assert!(vec_approx(r, Vec3::FORWARD), "got {r:?}");
    }

    #[test]
    fn mul_vec_dispatches_to_rotate() {
        let q = Quat::from_axis_angle(Vec3::Z, FRAC_PI_2);
        let direct = q.rotate_vec(Vec3::X);
        let via_op = q * Vec3::X;
        assert!(vec_approx(direct, via_op));
    }

    #[test]
    fn normalize_recovers_unit_length_after_drift() {
        let mut q = Quat::from_xyzw(0.1, 0.2, 0.3, 0.4); // not unit
        q = q.normalize();
        assert!(approx(q.length(), 1.0));
    }
}
