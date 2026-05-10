// math/src/aabb.rs
use crate::vector2::Vec2;

/// Axis-aligned bounding box in 2D.
///
/// Used both for broad-phase collision queries and simple geometry tests.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb {
    /// Minimum corner (lower-left) of the box.
    pub min: Vec2,
    /// Maximum corner (upper-right) of the box.
    pub max: Vec2,
}

impl Aabb {
    /// Create a new Aabb from its minimum and maximum corners.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create an Aabb from a center point and full size.
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Center point of the box.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Full width/height of the box.
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Half extents of the box (size / 2).
    pub fn half_size(&self) -> Vec2 {
        self.size() * 0.5
    }

    /// Test whether the box contains a point (inclusive of edges).
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Closest point on the box to `p` (clamps each coordinate to the box bounds).
    ///
    /// If `p` is inside the box, this returns `p` unchanged.
    pub fn closest_point(&self, p: Vec2) -> Vec2 {
        Vec2::new(
            p.x.clamp(self.min.x, self.max.x),
            p.y.clamp(self.min.y, self.max.y),
        )
    }

    /// Test whether two Aabbs overlap.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Compute the overlapping region between two Aabbs, if any.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        Some(Self {
            min: Vec2::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            max: Vec2::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        })
    }

    /// Return the smallest Aabb that contains both `self` and `other`.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: Vec2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Vec2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// Expand the box uniformly in all directions by `amount`.
    pub fn expand(&self, amount: f32) -> Self {
        let expansion = Vec2::new(amount, amount);
        Self {
            min: self.min - expansion,
            max: self.max + expansion,
        }
    }

    /// Return a copy of this box translated by `offset`.
    pub fn translate(&self, offset: Vec2) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
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
    fn construction_from_min_max() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(aabb.min, Vec2::new(0.0, 0.0));
        assert_eq!(aabb.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn construction_from_center_and_size() {
        let aabb = Aabb::from_center_size(Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0));
        assert_eq!(aabb.min, Vec2::new(0.0, 0.0));
        assert_eq!(aabb.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn construction_with_non_uniform_size() {
        let aabb = Aabb::from_center_size(Vec2::new(0.0, 0.0), Vec2::new(4.0, 6.0));
        assert_eq!(aabb.min, Vec2::new(-2.0, -3.0));
        assert_eq!(aabb.max, Vec2::new(2.0, 3.0));
    }

    // =========================================================================
    // Geometric Properties
    // =========================================================================

    #[test]
    fn center_of_aabb() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(aabb.center(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn size_of_aabb() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0));
        assert_eq!(aabb.size(), Vec2::new(10.0, 20.0));
    }

    #[test]
    fn half_size_of_aabb() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0));
        assert_eq!(aabb.half_size(), Vec2::new(5.0, 10.0));
    }

    // =========================================================================
    // Point Containment
    // =========================================================================

    #[test]
    fn contains_point_inside() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::new(5.0, 5.0)));
    }

    #[test]
    fn contains_point_on_edge() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::new(0.0, 5.0)));
        assert!(aabb.contains_point(Vec2::new(10.0, 5.0)));
        assert!(aabb.contains_point(Vec2::new(5.0, 0.0)));
        assert!(aabb.contains_point(Vec2::new(5.0, 10.0)));
    }

    #[test]
    fn contains_point_on_corner() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::new(0.0, 0.0)));
        assert!(aabb.contains_point(Vec2::new(10.0, 10.0)));
    }

    #[test]
    fn does_not_contain_point_outside() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(!aabb.contains_point(Vec2::new(-1.0, 5.0)));
        assert!(!aabb.contains_point(Vec2::new(11.0, 5.0)));
        assert!(!aabb.contains_point(Vec2::new(5.0, -1.0)));
        assert!(!aabb.contains_point(Vec2::new(5.0, 11.0)));
    }

    // =========================================================================
    // Closest Point
    // =========================================================================

    #[test]
    fn closest_point_inside_returns_point() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(aabb.closest_point(Vec2::new(3.0, 7.0)), Vec2::new(3.0, 7.0));
    }

    #[test]
    fn closest_point_outside_clamps_to_edge() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(
            aabb.closest_point(Vec2::new(-5.0, 5.0)),
            Vec2::new(0.0, 5.0)
        );
        assert_eq!(
            aabb.closest_point(Vec2::new(15.0, 5.0)),
            Vec2::new(10.0, 5.0)
        );
        assert_eq!(
            aabb.closest_point(Vec2::new(5.0, -3.0)),
            Vec2::new(5.0, 0.0)
        );
        assert_eq!(
            aabb.closest_point(Vec2::new(5.0, 13.0)),
            Vec2::new(5.0, 10.0)
        );
    }

    #[test]
    fn closest_point_diagonal_outside_clamps_to_corner() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(
            aabb.closest_point(Vec2::new(-5.0, -5.0)),
            Vec2::new(0.0, 0.0)
        );
        assert_eq!(
            aabb.closest_point(Vec2::new(15.0, 15.0)),
            Vec2::new(10.0, 10.0)
        );
    }

    // =========================================================================
    // Aabb-Aabb Intersection
    // =========================================================================

    #[test]
    fn intersects_overlapping_aabbs() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_touching_edges() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Aabb::new(Vec2::new(10.0, 0.0), Vec2::new(20.0, 10.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn does_not_intersect_separated_aabbs() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Aabb::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn intersection_region_of_overlapping_aabbs() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.min, Vec2::new(5.0, 5.0));
        assert_eq!(intersection.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn intersection_of_non_overlapping_returns_none() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = Aabb::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));
        assert!(a.intersection(&b).is_none());
    }

    // =========================================================================
    // Merge and Expand
    // =========================================================================

    #[test]
    fn merge_creates_bounding_aabb() {
        let a = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 5.0));
        let b = Aabb::new(Vec2::new(10.0, 10.0), Vec2::new(15.0, 15.0));
        let merged = a.merge(&b);
        assert_eq!(merged.min, Vec2::new(0.0, 0.0));
        assert_eq!(merged.max, Vec2::new(15.0, 15.0));
    }

    #[test]
    fn merge_with_contained_aabb_unchanged() {
        let outer = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
        let inner = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let merged = outer.merge(&inner);
        assert_eq!(merged.min, outer.min);
        assert_eq!(merged.max, outer.max);
    }

    #[test]
    fn expand_uniformly() {
        let aabb = Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0));
        let expanded = aabb.expand(2.0);
        assert_eq!(expanded.min, Vec2::new(3.0, 3.0));
        assert_eq!(expanded.max, Vec2::new(12.0, 12.0));
    }

    // =========================================================================
    // Translation
    // =========================================================================

    #[test]
    fn translate_moves_aabb() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let translated = aabb.translate(Vec2::new(5.0, 3.0));
        assert_eq!(translated.min, Vec2::new(5.0, 3.0));
        assert_eq!(translated.max, Vec2::new(15.0, 13.0));
    }

    #[test]
    fn translate_preserves_size() {
        let aabb = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let translated = aabb.translate(Vec2::new(100.0, 100.0));
        assert_eq!(aabb.size(), translated.size());
    }
}
