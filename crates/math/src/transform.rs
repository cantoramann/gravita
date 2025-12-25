// math/src/transform.rs
use crate::vector2::Vec2;

/// 2D transform combining translation, rotation and non-uniform scale.
///
/// This is primarily used for game-object style transforms and is not
/// meant to be a full matrix library.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform2D {
    /// World-space position.
    pub position: Vec2,
    /// Rotation in radians (counter-clockwise, right-handed).
    pub rotation: f32,
    /// Non-uniform scale applied before rotation.
    pub scale: Vec2,
}

impl Transform2D {
    /// Construct a new transform from position, rotation and scale.
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Identity transform (no translation, no rotation, unit scale).
    pub fn identity() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Transform a point from local space to world space.
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        // Scale -> Rotate -> Translate
        let scaled = Vec2::new(point.x * self.scale.x, point.y * self.scale.y);
        let rotated = scaled.rotate(self.rotation);
        rotated + self.position
    }

    /// Transform a vector (ignores translation).
    pub fn transform_vector(&self, vector: Vec2) -> Vec2 {
        let scaled = Vec2::new(vector.x * self.scale.x, vector.y * self.scale.y);
        scaled.rotate(self.rotation)
    }

    /// Get the inverse transform.
    pub fn inverse(&self) -> Self {
        let inv_scale = Vec2::new(1.0 / self.scale.x, 1.0 / self.scale.y);
        let inv_rotation = -self.rotation;
        let inv_position = -self.position.rotate(-self.rotation);

        Self {
            position: Vec2::new(inv_position.x * inv_scale.x, inv_position.y * inv_scale.y),
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// Combine two transforms (equivalent to `parent * child`).
    pub fn combine(&self, child: &Self) -> Self {
        Self {
            position: self.transform_point(child.position),
            rotation: self.rotation + child.rotation,
            scale: Vec2::new(self.scale.x * child.scale.x, self.scale.y * child.scale.y),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn construction_with_all_parameters() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0), PI, Vec2::new(2.0, 3.0));
        assert_eq!(t.position, Vec2::new(10.0, 20.0));
        assert_eq!(t.rotation, PI);
        assert_eq!(t.scale, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn identity_transform() {
        let t = Transform2D::identity();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn default_is_identity() {
        let t: Transform2D = Default::default();
        assert_eq!(t, Transform2D::identity());
    }

    // =========================================================================
    // Point Transformation
    // =========================================================================

    #[test]
    fn transform_point_with_translation_only() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0), 0.0, Vec2::ONE);
        let p = t.transform_point(Vec2::new(5.0, 5.0));
        assert_eq!(p, Vec2::new(15.0, 25.0));
    }

    #[test]
    fn transform_point_with_scale_only() {
        let t = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(2.0, 3.0));
        let p = t.transform_point(Vec2::new(5.0, 5.0));
        assert_eq!(p, Vec2::new(10.0, 15.0));
    }

    #[test]
    fn transform_point_with_rotation_only() {
        let t = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::ONE);
        let p = t.transform_point(Vec2::new(1.0, 0.0));
        assert!(vec_approx_eq(p, Vec2::new(0.0, 1.0)));
    }

    #[test]
    fn transform_point_with_all_components() {
        // Scale (2,2) -> Rotate 90° -> Translate (10, 10)
        let t = Transform2D::new(Vec2::new(10.0, 10.0), FRAC_PI_2, Vec2::new(2.0, 2.0));
        let p = t.transform_point(Vec2::new(1.0, 0.0));
        // (1,0) * 2 = (2,0), rotate 90° = (0,2), translate = (10, 12)
        assert!(vec_approx_eq(p, Vec2::new(10.0, 12.0)));
    }

    #[test]
    fn identity_transform_point_unchanged() {
        let t = Transform2D::identity();
        let p = Vec2::new(5.0, 7.0);
        assert_eq!(t.transform_point(p), p);
    }

    // =========================================================================
    // Vector Transformation
    // =========================================================================

    #[test]
    fn transform_vector_ignores_translation() {
        let t = Transform2D::new(Vec2::new(100.0, 100.0), 0.0, Vec2::ONE);
        let v = t.transform_vector(Vec2::new(1.0, 0.0));
        assert_eq!(v, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn transform_vector_applies_scale() {
        let t = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(2.0, 3.0));
        let v = t.transform_vector(Vec2::new(1.0, 1.0));
        assert_eq!(v, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn transform_vector_applies_rotation() {
        let t = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::ONE);
        let v = t.transform_vector(Vec2::new(1.0, 0.0));
        assert!(vec_approx_eq(v, Vec2::new(0.0, 1.0)));
    }

    // =========================================================================
    // Inverse Transform
    // =========================================================================

    #[test]
    fn inverse_of_identity_is_identity() {
        let t = Transform2D::identity();
        let inv = t.inverse();
        assert_eq!(inv.position, Vec2::ZERO);
        assert_eq!(inv.rotation, 0.0);
        assert_eq!(inv.scale, Vec2::ONE);
    }

    #[test]
    fn inverse_undoes_translation() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0), 0.0, Vec2::ONE);
        let inv = t.inverse();
        let p = Vec2::new(5.0, 5.0);
        let transformed = t.transform_point(p);
        let restored = inv.transform_point(transformed);
        assert!(vec_approx_eq(restored, p));
    }

    #[test]
    fn inverse_undoes_scale() {
        let t = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(2.0, 4.0));
        let inv = t.inverse();
        assert_eq!(inv.scale, Vec2::new(0.5, 0.25));
    }

    #[test]
    fn inverse_undoes_rotation() {
        let t = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::ONE);
        let inv = t.inverse();
        assert!(approx_eq(inv.rotation, -FRAC_PI_2));
    }

    // =========================================================================
    // Combine Transforms
    // =========================================================================

    #[test]
    fn combine_translations() {
        let parent = Transform2D::new(Vec2::new(10.0, 0.0), 0.0, Vec2::ONE);
        let child = Transform2D::new(Vec2::new(5.0, 0.0), 0.0, Vec2::ONE);
        let combined = parent.combine(&child);
        assert_eq!(combined.position, Vec2::new(15.0, 0.0));
    }

    #[test]
    fn combine_scales() {
        let parent = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(2.0, 2.0));
        let child = Transform2D::new(Vec2::ZERO, 0.0, Vec2::new(3.0, 3.0));
        let combined = parent.combine(&child);
        assert_eq!(combined.scale, Vec2::new(6.0, 6.0));
    }

    #[test]
    fn combine_rotations() {
        let parent = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::ONE);
        let child = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::ONE);
        let combined = parent.combine(&child);
        assert!(approx_eq(combined.rotation, PI));
    }

    #[test]
    fn combine_with_identity_unchanged() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0), 0.5, Vec2::new(2.0, 3.0));
        let identity = Transform2D::identity();
        let combined = t.combine(&identity);
        assert_eq!(combined.position, t.position);
        assert_eq!(combined.rotation, t.rotation);
        assert_eq!(combined.scale, t.scale);
    }
}
