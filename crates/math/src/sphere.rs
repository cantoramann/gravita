// math/src/sphere.rs

use crate::{aabb3::Aabb3, vector3::Vec3};

/// Sphere primitive used for 3D collision and broad-phase culling.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    /// Center in world or local space.
    pub center: Vec3,
    /// Radius.
    pub radius: f32,
}

impl Sphere {
    /// Build from a center and radius.
    #[inline]
    pub const fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Test whether a point lies inside or on the surface.
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }

    /// Test against another sphere.
    pub fn intersects_sphere(&self, other: &Self) -> bool {
        let d_sq = self.center.distance_squared(other.center);
        let r = self.radius + other.radius;
        d_sq <= r * r
    }

    /// Test against an axis-aligned box.
    pub fn intersects_aabb(&self, aabb: &Aabb3) -> bool {
        let closest = aabb.closest_point(self.center);
        self.contains_point(closest)
    }

    /// Tight bounding box that encloses the sphere.
    pub fn to_aabb(&self) -> Aabb3 {
        let r = Vec3::splat(self.radius);
        Aabb3 {
            min: self.center - r,
            max: self.center + r,
        }
    }

    /// Translate by `offset`.
    pub fn translate(&self, offset: Vec3) -> Self {
        Self {
            center: self.center + offset,
            radius: self.radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_point_inside_and_on_surface() {
        let s = Sphere::new(Vec3::ZERO, 10.0);
        assert!(s.contains_point(Vec3::ZERO));
        assert!(s.contains_point(Vec3::new(10.0, 0.0, 0.0)));
        assert!(s.contains_point(Vec3::new(5.0, 5.0, 5.0)));
        assert!(!s.contains_point(Vec3::new(11.0, 0.0, 0.0)));
    }

    #[test]
    fn intersects_sphere_overlap() {
        let a = Sphere::new(Vec3::ZERO, 10.0);
        let b = Sphere::new(Vec3::new(15.0, 0.0, 0.0), 10.0);
        assert!(a.intersects_sphere(&b));
    }

    #[test]
    fn intersects_sphere_touching() {
        let a = Sphere::new(Vec3::ZERO, 10.0);
        let b = Sphere::new(Vec3::new(20.0, 0.0, 0.0), 10.0);
        // Distance == radius sum → just touching is treated as intersecting.
        assert!(a.intersects_sphere(&b));
    }

    #[test]
    fn does_not_intersect_separated_spheres() {
        let a = Sphere::new(Vec3::ZERO, 10.0);
        let b = Sphere::new(Vec3::new(25.0, 0.0, 0.0), 10.0);
        assert!(!a.intersects_sphere(&b));
    }

    #[test]
    fn intersects_aabb_when_overlapping() {
        let s = Sphere::new(Vec3::new(5.0, 5.0, 5.0), 10.0);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert!(s.intersects_aabb(&a));
    }

    #[test]
    fn intersects_aabb_corner_brush() {
        // Sphere near corner of the box.
        let s = Sphere::new(Vec3::new(-3.0, -3.0, -3.0), 6.0);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert!(s.intersects_aabb(&a));
    }

    #[test]
    fn does_not_intersect_aabb_separated() {
        let s = Sphere::new(Vec3::splat(-20.0), 5.0);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert!(!s.intersects_aabb(&a));
    }

    #[test]
    fn to_aabb_creates_tight_bounds() {
        let s = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 5.0);
        let a = s.to_aabb();
        assert_eq!(a.min, Vec3::new(-4.0, -3.0, -2.0));
        assert_eq!(a.max, Vec3::new(6.0, 7.0, 8.0));
    }

    #[test]
    fn translate_moves_center_keeps_radius() {
        let s = Sphere::new(Vec3::ZERO, 5.0);
        let t = s.translate(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(t.center, Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(t.radius, 5.0);
    }
}
