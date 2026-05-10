// math/src/transform3.rs

//! 3D affine transform: translate, rotate, scale.

use crate::{quat::Quat, vector3::Vec3};

/// Affine 3D transform expressed as Translate-Rotate-Scale.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform3D {
    /// World-space position.
    pub position: Vec3,
    /// Rotation as a unit quaternion.
    pub rotation: Quat,
    /// Per-axis scale. Use [`Vec3::ONE`] for no scaling.
    pub scale: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform3D {
    /// Identity (no translation, no rotation, unit scale).
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Translation-only transform.
    pub const fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Builder: override position.
    #[must_use]
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Builder: override rotation.
    #[must_use]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Builder: override scale.
    #[must_use]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Transform a point: `T(p) = position + rotation * (scale * p)`.
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        self.position + self.rotation.rotate_vec(self.scaled(p))
    }

    /// Transform a direction (ignores translation, but applies scale).
    pub fn transform_direction(&self, d: Vec3) -> Vec3 {
        self.rotation.rotate_vec(self.scaled(d))
    }

    /// Inverse transform of a world-space point back into local space.
    pub fn inverse_transform_point(&self, p: Vec3) -> Vec3 {
        let local = self.rotation.inverse().rotate_vec(p - self.position);
        self.unscaled(local)
    }

    /// Compose two transforms: `self ∘ child` applies `child` first then `self`.
    pub fn combine(&self, child: &Self) -> Self {
        Self {
            position: self.transform_point(child.position),
            rotation: self.rotation * child.rotation,
            scale: Vec3::new(
                self.scale.x * child.scale.x,
                self.scale.y * child.scale.y,
                self.scale.z * child.scale.z,
            ),
        }
    }

    /// Output a 4×4 column-major affine matrix (suitable for `wgpu`/`glam`
    /// consumers). Layout: `[col0, col1, col2, col3]` where each col is `[x, y, z, w]`.
    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        // R = rotation matrix from quaternion
        let q = self.rotation;
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;
        // R rows: i = (1-2(yy+zz), 2(xy-wz), 2(xz+wy))
        //         j = (2(xy+wz), 1-2(xx+zz), 2(yz-wx))
        //         k = (2(xz-wy), 2(yz+wx), 1-2(xx+yy))
        let r00 = (-2.0_f32).mul_add(yy + zz, 1.0);
        let r01 = 2.0 * (xy - wz);
        let r02 = 2.0 * (xz + wy);
        let r10 = 2.0 * (xy + wz);
        let r11 = (-2.0_f32).mul_add(xx + zz, 1.0);
        let r12 = 2.0 * (yz - wx);
        let r20 = 2.0 * (xz - wy);
        let r21 = 2.0 * (yz + wx);
        let r22 = (-2.0_f32).mul_add(xx + yy, 1.0);
        let s = self.scale;
        let p = self.position;
        // Column-major
        [
            [r00 * s.x, r10 * s.x, r20 * s.x, 0.0],
            [r01 * s.y, r11 * s.y, r21 * s.y, 0.0],
            [r02 * s.z, r12 * s.z, r22 * s.z, 0.0],
            [p.x, p.y, p.z, 1.0],
        ]
    }

    #[inline]
    fn scaled(&self, v: Vec3) -> Vec3 {
        Vec3::new(v.x * self.scale.x, v.y * self.scale.y, v.z * self.scale.z)
    }

    #[inline]
    fn unscaled(&self, v: Vec3) -> Vec3 {
        let inv = |s: f32, x: f32| if s.abs() > f32::EPSILON { x / s } else { 0.0 };
        Vec3::new(inv(self.scale.x, v.x), inv(self.scale.y, v.y), inv(self.scale.z, v.z))
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    #[test]
    fn identity_transforms_point_unchanged() {
        let t = Transform3D::IDENTITY;
        assert!(vec_approx(t.transform_point(Vec3::new(1.0, 2.0, 3.0)), Vec3::new(1.0, 2.0, 3.0)));
    }

    #[test]
    fn translate_only_adds_position() {
        let t = Transform3D::from_position(Vec3::new(10.0, 0.0, 0.0));
        assert!(vec_approx(t.transform_point(Vec3::ZERO), Vec3::new(10.0, 0.0, 0.0)));
    }

    #[test]
    fn rotation_only_rotates_point() {
        let t = Transform3D::IDENTITY.with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
        assert!(vec_approx(t.transform_point(Vec3::X), Vec3::Y));
    }

    #[test]
    fn scale_only_scales_components() {
        let t = Transform3D::IDENTITY.with_scale(Vec3::new(2.0, 3.0, 4.0));
        assert!(vec_approx(
            t.transform_point(Vec3::new(1.0, 1.0, 1.0)),
            Vec3::new(2.0, 3.0, 4.0)
        ));
    }

    #[test]
    fn trs_applies_scale_then_rotation_then_translation() {
        // Scale X by 2, rotate 90° around Z (X→Y), translate +1 on Y.
        let t = Transform3D::IDENTITY
            .with_scale(Vec3::new(2.0, 1.0, 1.0))
            .with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2))
            .with_position(Vec3::new(0.0, 1.0, 0.0));
        // x=1 → scaled (2,0,0) → rotated (0,2,0) → translated (0,3,0)
        assert!(vec_approx(t.transform_point(Vec3::X), Vec3::new(0.0, 3.0, 0.0)));
    }

    #[test]
    fn inverse_transform_undoes_forward_transform() {
        let t = Transform3D::IDENTITY
            .with_scale(Vec3::new(2.0, 3.0, 4.0))
            .with_rotation(Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalize(), 1.234))
            .with_position(Vec3::new(5.0, -2.0, 7.0));
        let p = Vec3::new(0.5, 0.6, 0.7);
        let world = t.transform_point(p);
        let restored = t.inverse_transform_point(world);
        assert!(vec_approx(restored, p), "got {restored:?} expected {p:?}");
    }

    #[test]
    fn transform_direction_ignores_position() {
        let t = Transform3D::IDENTITY
            .with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2))
            .with_position(Vec3::new(100.0, 100.0, 100.0));
        // Direction should rotate but not translate.
        assert!(vec_approx(t.transform_direction(Vec3::X), Vec3::Y));
    }

    #[test]
    fn combine_chains_transforms() {
        let parent = Transform3D::IDENTITY.with_position(Vec3::new(1.0, 0.0, 0.0));
        let child = Transform3D::IDENTITY.with_position(Vec3::new(0.0, 2.0, 0.0));
        let world = parent.combine(&child);
        // Child's origin in parent's frame is (0,2,0); plus parent's (1,0,0) = (1,2,0).
        assert!(vec_approx(world.position, Vec3::new(1.0, 2.0, 0.0)));
    }

    #[test]
    fn to_matrix_identity_is_standard_basis() {
        let m = Transform3D::IDENTITY.to_matrix();
        assert_eq!(m[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(m[2], [0.0, 0.0, 1.0, 0.0]);
        assert_eq!(m[3], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn to_matrix_translation_in_last_column() {
        let t = Transform3D::from_position(Vec3::new(7.0, 8.0, 9.0));
        let m = t.to_matrix();
        // Last column is (px, py, pz, 1)
        assert_eq!(m[3], [7.0, 8.0, 9.0, 1.0]);
    }

    #[test]
    fn to_matrix_rotation_matches_rotate_vec() {
        let q = Quat::from_axis_angle(Vec3::Y, FRAC_PI_2);
        let t = Transform3D::IDENTITY.with_rotation(q);
        let m = t.to_matrix();
        // Apply m to X = (1,0,0,1) (column-major), expect direction (0,0,-1).
        let x = m[0][0]; // column 0, row 0
        let y = m[0][1];
        let z = m[0][2];
        assert!(approx(x, 0.0));
        assert!(approx(y, 0.0));
        assert!(approx(z, -1.0));
    }
}
