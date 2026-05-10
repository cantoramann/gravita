// math/src/circle.rs
use crate::{aabb::Aabb, vector2::Vec2};

/// Circle primitive used for simple collision and broad-phase acceleration.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Circle {
    /// Center of the circle in world or local space.
    pub center: Vec2,
    /// Radius of the circle.
    pub radius: f32,
}

impl Circle {
    /// Create a new circle from a center and radius.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Test whether a point lies inside or on the circle.
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }

    /// Test for intersection with another circle.
    pub fn intersects_circle(&self, other: &Self) -> bool {
        let distance_squared = self.center.distance_squared(other.center);
        let radius_sum = self.radius + other.radius;
        distance_squared <= radius_sum * radius_sum
    }

    /// Test for intersection with an axis-aligned bounding box.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        self.contains_point(aabb.closest_point(self.center))
    }

    /// Compute the tight Aabb that encloses this circle.
    pub fn to_aabb(&self) -> Aabb {
        let radius_vec = Vec2::new(self.radius, self.radius);
        Aabb {
            min: self.center - radius_vec,
            max: self.center + radius_vec,
        }
    }

    /// Return a copy of this circle translated by `offset`.
    pub fn translate(&self, offset: Vec2) -> Self {
        Self {
            center: self.center + offset,
            radius: self.radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn construction_with_center_and_radius() {
        let c = Circle::new(Vec2::new(5.0, 5.0), 10.0);
        assert_eq!(c.center, Vec2::new(5.0, 5.0));
        assert_eq!(c.radius, 10.0);
    }

    // =========================================================================
    // Point Containment
    // =========================================================================

    #[test]
    fn contains_point_at_center() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        assert!(c.contains_point(Vec2::ZERO));
    }

    #[test]
    fn contains_point_inside() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        assert!(c.contains_point(Vec2::new(5.0, 0.0)));
        assert!(c.contains_point(Vec2::new(0.0, 5.0)));
    }

    #[test]
    fn contains_point_on_boundary() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        assert!(c.contains_point(Vec2::new(10.0, 0.0)));
        assert!(c.contains_point(Vec2::new(0.0, 10.0)));
    }

    #[test]
    fn does_not_contain_point_outside() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        assert!(!c.contains_point(Vec2::new(11.0, 0.0)));
        assert!(!c.contains_point(Vec2::new(8.0, 8.0))); // sqrt(128) > 10
    }

    // =========================================================================
    // Circle-Circle Intersection
    // =========================================================================

    #[test]
    fn intersects_overlapping_circles() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(15.0, 0.0), 10.0);
        assert!(a.intersects_circle(&b));
    }

    #[test]
    fn intersects_touching_circles() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(20.0, 0.0), 10.0);
        assert!(a.intersects_circle(&b));
    }

    #[test]
    fn intersects_concentric_circles() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::ZERO, 5.0);
        assert!(a.intersects_circle(&b));
    }

    #[test]
    fn does_not_intersect_separated_circles() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(25.0, 0.0), 10.0);
        assert!(!a.intersects_circle(&b));
    }

    // =========================================================================
    // Circle-Aabb Intersection
    // =========================================================================

    #[test]
    fn intersects_aabb_when_overlapping() {
        let c = Circle::new(Vec2::new(5.0, 5.0), 10.0);
        let aabb = Aabb::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(c.intersects_aabb(&aabb));
    }

    #[test]
    fn intersects_aabb_circle_inside() {
        let c = Circle::new(Vec2::new(5.0, 5.0), 2.0);
        let aabb = Aabb::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(c.intersects_aabb(&aabb));
    }

    #[test]
    fn intersects_aabb_touching_corner() {
        // Circle at corner, touching exactly
        let c = Circle::new(Vec2::new(-5.0, -5.0), 5.0 * 2.0_f32.sqrt());
        let aabb = Aabb::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(c.intersects_aabb(&aabb));
    }

    #[test]
    fn does_not_intersect_aabb_when_separated() {
        let c = Circle::new(Vec2::new(-20.0, 5.0), 5.0);
        let aabb = Aabb::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(!c.intersects_aabb(&aabb));
    }

    // =========================================================================
    // Aabb Conversion
    // =========================================================================

    #[test]
    fn to_aabb_creates_tight_bounds() {
        let c = Circle::new(Vec2::new(10.0, 20.0), 5.0);
        let aabb = c.to_aabb();
        assert_eq!(aabb.min, Vec2::new(5.0, 15.0));
        assert_eq!(aabb.max, Vec2::new(15.0, 25.0));
    }

    #[test]
    fn to_aabb_at_origin() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        let aabb = c.to_aabb();
        assert_eq!(aabb.min, Vec2::new(-10.0, -10.0));
        assert_eq!(aabb.max, Vec2::new(10.0, 10.0));
    }

    // =========================================================================
    // Translation
    // =========================================================================

    #[test]
    fn translate_moves_center() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        let translated = c.translate(Vec2::new(5.0, 3.0));
        assert_eq!(translated.center, Vec2::new(5.0, 3.0));
    }

    #[test]
    fn translate_preserves_radius() {
        let c = Circle::new(Vec2::ZERO, 10.0);
        let translated = c.translate(Vec2::new(100.0, 100.0));
        assert_eq!(c.radius, translated.radius);
    }
}
