// math/src/aabb3.rs

use crate::vector3::Vec3;

/// Axis-aligned bounding box in 3D.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb3 {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
}

impl Aabb3 {
    /// Build from explicit min/max corners.
    #[inline]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Build from a center point and full extents.
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Center point.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Full size (width, height, depth).
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Half size from center to corner.
    pub fn half_size(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Test whether `point` is inside (inclusive of faces/edges/corners).
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Closest point on the box to `p` (clamps each coordinate to the bounds).
    pub fn closest_point(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            p.x.clamp(self.min.x, self.max.x),
            p.y.clamp(self.min.y, self.max.y),
            p.z.clamp(self.min.z, self.max.z),
        )
    }

    /// Test for overlap with another AABB (inclusive on touching faces).
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Overlapping region, if any.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        Some(Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        })
    }

    /// Smallest box that contains both `self` and `other`.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Uniformly expand outward by `amount` on all faces.
    pub fn expand(&self, amount: f32) -> Self {
        let e = Vec3::splat(amount);
        Self {
            min: self.min - e,
            max: self.max + e,
        }
    }

    /// Translate by `offset`.
    pub fn translate(&self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_records_corners() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert_eq!(a.min, Vec3::ZERO);
        assert_eq!(a.max, Vec3::splat(10.0));
    }

    #[test]
    fn from_center_size_yields_correct_bounds() {
        let a = Aabb3::from_center_size(Vec3::new(5.0, 5.0, 5.0), Vec3::splat(10.0));
        assert_eq!(a.min, Vec3::ZERO);
        assert_eq!(a.max, Vec3::splat(10.0));
    }

    #[test]
    fn center_and_size_round_trip() {
        let a = Aabb3::from_center_size(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 6.0, 8.0));
        assert_eq!(a.center(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(a.size(), Vec3::new(4.0, 6.0, 8.0));
        assert_eq!(a.half_size(), Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn contains_inside_edge_outside() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert!(a.contains_point(Vec3::new(5.0, 5.0, 5.0)));
        assert!(a.contains_point(Vec3::ZERO));
        assert!(a.contains_point(Vec3::splat(10.0)));
        assert!(!a.contains_point(Vec3::new(-1.0, 5.0, 5.0)));
        assert!(!a.contains_point(Vec3::new(5.0, 5.0, 11.0)));
    }

    #[test]
    fn closest_point_inside_returns_self() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let p = Vec3::new(3.0, 4.0, 5.0);
        assert_eq!(a.closest_point(p), p);
    }

    #[test]
    fn closest_point_outside_clamps() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        assert_eq!(
            a.closest_point(Vec3::new(-1.0, 5.0, 12.0)),
            Vec3::new(0.0, 5.0, 10.0)
        );
    }

    #[test]
    fn intersects_overlapping() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let b = Aabb3::new(Vec3::splat(5.0), Vec3::splat(15.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_touching_face() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let b = Aabb3::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(20.0, 10.0, 10.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn does_not_intersect_separated() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let b = Aabb3::new(Vec3::splat(20.0), Vec3::splat(30.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn intersection_returns_overlap() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let b = Aabb3::new(Vec3::splat(5.0), Vec3::splat(15.0));
        let o = a.intersection(&b).unwrap();
        assert_eq!(o.min, Vec3::splat(5.0));
        assert_eq!(o.max, Vec3::splat(10.0));
    }

    #[test]
    fn intersection_none_when_separated() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let b = Aabb3::new(Vec3::splat(20.0), Vec3::splat(30.0));
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn merge_produces_bounding_box() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(5.0));
        let b = Aabb3::new(Vec3::splat(10.0), Vec3::splat(15.0));
        let m = a.merge(&b);
        assert_eq!(m.min, Vec3::ZERO);
        assert_eq!(m.max, Vec3::splat(15.0));
    }

    #[test]
    fn expand_grows_all_faces() {
        let a = Aabb3::new(Vec3::splat(5.0), Vec3::splat(10.0)).expand(2.0);
        assert_eq!(a.min, Vec3::splat(3.0));
        assert_eq!(a.max, Vec3::splat(12.0));
    }

    #[test]
    fn translate_preserves_size() {
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(10.0));
        let t = a.translate(Vec3::new(100.0, 200.0, 300.0));
        assert_eq!(t.size(), a.size());
        assert_eq!(t.min, Vec3::new(100.0, 200.0, 300.0));
    }
}
