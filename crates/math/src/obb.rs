// math/src/obb.rs

//! Oriented bounding box: a box that may be rotated relative to the world axes.
//!
//! `Obb` is a strict superset of [`Aabb3`](crate::Aabb3): an OBB with the
//! identity rotation is equivalent to an axis-aligned box. The conversion is
//! free in both directions via [`Aabb3::to_obb`] and [`Obb::to_aabb`].

use crate::{aabb3::Aabb3, quat::Quat, vector3::Vec3};

/// Box defined by its center, three half-extents along its local axes, and
/// a rotation that maps local axes to world axes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Obb {
    /// World-space center.
    pub center: Vec3,
    /// Half-extents along the OBB's local `X`, `Y`, `Z` axes.
    pub half_extents: Vec3,
    /// Rotation from local space to world space.
    pub rotation: Quat,
}

impl Obb {
    /// Build a new OBB.
    #[inline]
    pub const fn new(center: Vec3, half_extents: Vec3, rotation: Quat) -> Self {
        Self {
            center,
            half_extents,
            rotation,
        }
    }

    /// World-space orientation axes in the order `[X, Y, Z]`.
    pub fn axes(&self) -> [Vec3; 3] {
        [
            self.rotation.rotate_vec(Vec3::X),
            self.rotation.rotate_vec(Vec3::Y),
            self.rotation.rotate_vec(Vec3::Z),
        ]
    }

    /// Map a point in OBB-local space to world space.
    pub fn local_to_world(&self, p: Vec3) -> Vec3 {
        self.center + self.rotation.rotate_vec(p)
    }

    /// Map a world-space point into OBB-local space.
    pub fn world_to_local(&self, p: Vec3) -> Vec3 {
        self.rotation.inverse().rotate_vec(p - self.center)
    }

    /// The eight world-space corners of the box.
    pub fn corners(&self) -> [Vec3; 8] {
        let h = self.half_extents;
        let signs = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ];
        let mut out = [Vec3::ZERO; 8];
        for (i, s) in signs.iter().enumerate() {
            let local = Vec3::new(s.x * h.x, s.y * h.y, s.z * h.z);
            out[i] = self.local_to_world(local);
        }
        out
    }

    /// Closest point on (or inside) the OBB to `p`.
    pub fn closest_point(&self, p: Vec3) -> Vec3 {
        let local = self.world_to_local(p);
        let h = self.half_extents;
        let clamped = Vec3::new(
            local.x.clamp(-h.x, h.x),
            local.y.clamp(-h.y, h.y),
            local.z.clamp(-h.z, h.z),
        );
        self.local_to_world(clamped)
    }

    /// World-axis-aligned box that contains every corner of the OBB.
    pub fn to_aabb(&self) -> Aabb3 {
        let corners = self.corners();
        let mut min = corners[0];
        let mut max = corners[0];
        for c in corners.iter().skip(1) {
            min = min.min(*c);
            max = max.max(*c);
        }
        Aabb3::new(min, max)
    }

    /// Quick reject AABB used by broad-phase queries.
    #[inline]
    pub fn enclosing_aabb(&self) -> Aabb3 {
        self.to_aabb()
    }
}

impl Aabb3 {
    /// Promote an AABB to an OBB with identity rotation.
    pub fn to_obb(&self) -> Obb {
        Obb::new(self.center(), self.half_size(), Quat::IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }
    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    #[test]
    fn identity_obb_is_axis_aligned() {
        let o = Obb::new(Vec3::ZERO, Vec3::splat(1.0), Quat::IDENTITY);
        let [x, y, z] = o.axes();
        assert!(vec_approx(x, Vec3::X));
        assert!(vec_approx(y, Vec3::Y));
        assert!(vec_approx(z, Vec3::Z));
    }

    #[test]
    fn rotation_changes_axes() {
        let o = Obb::new(
            Vec3::ZERO,
            Vec3::splat(1.0),
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
        );
        let [x, _, z] = o.axes();
        // 90° around Y takes X to -Z, Z to X.
        assert!(vec_approx(x, -Vec3::Z));
        assert!(vec_approx(z, Vec3::X));
    }

    #[test]
    fn world_local_round_trip() {
        let o = Obb::new(
            Vec3::new(2.0, 3.0, 4.0),
            Vec3::splat(1.0),
            Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalize(), 1.234),
        );
        let p = Vec3::new(5.0, -2.0, 7.0);
        assert!(vec_approx(o.local_to_world(o.world_to_local(p)), p));
    }

    #[test]
    fn corners_are_distinct_for_non_degenerate_obb() {
        let o = Obb::new(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY);
        let corners = o.corners();
        // Eight unique corners.
        use std::collections::HashSet;
        let bits: HashSet<_> = corners
            .iter()
            .map(|c| (c.x.to_bits(), c.y.to_bits(), c.z.to_bits()))
            .collect();
        assert_eq!(bits.len(), 8);
    }

    #[test]
    fn closest_point_inside_returns_p() {
        let o = Obb::new(Vec3::ZERO, Vec3::splat(2.0), Quat::IDENTITY);
        let p = Vec3::new(0.5, 0.5, 0.5);
        assert!(vec_approx(o.closest_point(p), p));
    }

    #[test]
    fn closest_point_outside_along_axis() {
        let o = Obb::new(Vec3::ZERO, Vec3::splat(1.0), Quat::IDENTITY);
        let p = Vec3::new(5.0, 0.0, 0.0);
        assert!(vec_approx(o.closest_point(p), Vec3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn closest_point_handles_rotated_obb() {
        // Rotate 45° around Y. A point on the +X world axis should clamp to
        // the OBB's local-X face (now sqrt(2)/2 along world X and Z).
        let o = Obb::new(
            Vec3::ZERO,
            Vec3::splat(1.0),
            Quat::from_axis_angle(Vec3::Y, FRAC_PI_4),
        );
        let p = Vec3::new(5.0, 0.0, 0.0);
        let c = o.closest_point(p);
        // The closest point is a corner of the OBB on the +X side.
        assert!(approx(c.length(), 2.0_f32.sqrt()));
    }

    #[test]
    fn to_aabb_bounds_all_corners() {
        let o = Obb::new(
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::splat(1.0),
            Quat::from_axis_angle(Vec3::Y, FRAC_PI_4),
        );
        let a = o.to_aabb();
        for c in o.corners() {
            assert!(a.contains_point(c));
        }
    }

    #[test]
    fn aabb_to_obb_round_trip() {
        let a = Aabb3::from_center_size(Vec3::new(1.0, 2.0, 3.0), Vec3::new(2.0, 4.0, 6.0));
        let o = a.to_obb();
        assert!(vec_approx(o.center, a.center()));
        assert!(vec_approx(o.half_extents, a.half_size()));
        assert_eq!(o.rotation, Quat::IDENTITY);
    }
}
