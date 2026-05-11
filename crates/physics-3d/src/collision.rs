// physics-3d/src/collision.rs

//! Narrow-phase 3D collision tests and the simple detector loop.

use gravita_math::{Aabb3, Obb, Sphere, Vec3};

use crate::body::{BodyType, CollisionShape, RigidBody};

/// Contact between two bodies.
#[derive(Debug, Clone, Copy)]
pub struct Contact {
    /// Index of first body in the world's body list.
    pub body_a: usize,
    /// Index of second body.
    pub body_b: usize,
    /// World-space contact point.
    pub point: Vec3,
    /// Unit-length contact normal pointing from `body_a` toward `body_b`.
    pub normal: Vec3,
    /// Positive overlap depth (zero when just touching).
    pub penetration: f32,
    /// Combined restitution (min).
    pub restitution: f32,
    /// Combined friction (geometric mean).
    pub friction: f32,
}

impl Contact {
    /// Build a new contact with zeroed material properties.
    pub fn new(body_a: usize, body_b: usize) -> Self {
        Self {
            body_a,
            body_b,
            point: Vec3::ZERO,
            normal: Vec3::Y,
            penetration: 0.0,
            restitution: 0.0,
            friction: 0.0,
        }
    }

    /// Swap bodies and flip the normal.
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.body_a, &mut self.body_b);
        self.normal = -self.normal;
    }
}

/// Sphere–Sphere narrow phase. Bodies passed as translated world-space spheres.
pub fn test_sphere_sphere(a: &Sphere, b: &Sphere, a_idx: usize, b_idx: usize) -> Option<Contact> {
    let delta = b.center - a.center;
    let r_sum = a.radius + b.radius;
    let dist_sq = delta.length_squared();
    if dist_sq >= r_sum * r_sum {
        return None;
    }
    let mut c = Contact::new(a_idx, b_idx);
    if dist_sq < 1e-10 {
        // Coincident centers — pick an arbitrary normal so the solver can resolve.
        c.normal = Vec3::Y;
        c.penetration = r_sum;
        c.point = a.center;
    } else {
        let dist = dist_sq.sqrt();
        c.normal = delta / dist;
        c.penetration = r_sum - dist;
        c.point = a.center + c.normal * c.penetration.mul_add(-0.5, a.radius);
    }
    Some(c)
}

/// Sphere–AABB narrow phase. AABB is body B; normal points from sphere toward box.
pub fn test_sphere_aabb(
    sphere: &Sphere,
    aabb: &Aabb3,
    sphere_idx: usize,
    aabb_idx: usize,
) -> Option<Contact> {
    let closest = aabb.closest_point(sphere.center);
    let delta = closest - sphere.center;
    let dist_sq = delta.length_squared();
    if dist_sq >= sphere.radius * sphere.radius {
        return None;
    }
    let mut c = Contact::new(sphere_idx, aabb_idx);
    if dist_sq < 1e-10 {
        // Center inside the box — push out along the shortest axis to a face.
        let to_min = sphere.center - aabb.min;
        let to_max = aabb.max - sphere.center;
        let mut best = to_min.x;
        let mut axis = Vec3::new(-1.0, 0.0, 0.0);
        if to_max.x < best {
            best = to_max.x;
            axis = Vec3::new(1.0, 0.0, 0.0);
        }
        if to_min.y < best {
            best = to_min.y;
            axis = Vec3::new(0.0, -1.0, 0.0);
        }
        if to_max.y < best {
            best = to_max.y;
            axis = Vec3::new(0.0, 1.0, 0.0);
        }
        if to_min.z < best {
            best = to_min.z;
            axis = Vec3::new(0.0, 0.0, -1.0);
        }
        if to_max.z < best {
            best = to_max.z;
            axis = Vec3::new(0.0, 0.0, 1.0);
        }
        c.normal = axis;
        c.penetration = sphere.radius + best;
        c.point = sphere.center + axis * best;
    } else {
        let dist = dist_sq.sqrt();
        c.normal = delta / dist;
        c.penetration = sphere.radius - dist;
        c.point = closest;
    }
    Some(c)
}

/// Sphere–OBB narrow phase using the OBB's closest-point routine.
pub fn test_sphere_obb(
    sphere: &Sphere,
    obb: &Obb,
    sphere_idx: usize,
    obb_idx: usize,
) -> Option<Contact> {
    let closest = obb.closest_point(sphere.center);
    let delta = closest - sphere.center;
    let dist_sq = delta.length_squared();
    if dist_sq >= sphere.radius * sphere.radius {
        return None;
    }
    let mut c = Contact::new(sphere_idx, obb_idx);
    if dist_sq < 1e-10 {
        // Sphere center inside the box: push out through the nearest local face.
        let local = obb.world_to_local(sphere.center);
        let h = obb.half_extents;
        // Distance to each face along its outward normal in local space.
        let candidates = [
            (h.x - local.x.abs(), Vec3::new(local.x.signum(), 0.0, 0.0)),
            (h.y - local.y.abs(), Vec3::new(0.0, local.y.signum(), 0.0)),
            (h.z - local.z.abs(), Vec3::new(0.0, 0.0, local.z.signum())),
        ];
        let (best_d, local_axis) = candidates
            .into_iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        c.normal = obb.rotation.rotate_vec(local_axis);
        c.penetration = sphere.radius + best_d;
        c.point = sphere.center + c.normal * best_d;
    } else {
        let dist = dist_sq.sqrt();
        c.normal = delta / dist;
        c.penetration = sphere.radius - dist;
        c.point = closest;
    }
    Some(c)
}

/// OBB–OBB narrow phase via the Separating Axis Theorem with all 15 axes
/// (3 from each box's local frame plus 9 cross products of those axes).
/// Reference: Christer Ericson, *Real-Time Collision Detection*, §4.4.
#[allow(clippy::suboptimal_flops)] // SAT projections read most clearly as Σ a·b.
#[allow(clippy::needless_range_loop)] // i/j index 5 parallel arrays; iter-zip would obscure the math.
pub fn test_obb_obb(a: &Obb, b: &Obb, ai: usize, bi: usize) -> Option<Contact> {
    let ax_a = a.axes();
    let ax_b = b.axes();
    let h_a = [a.half_extents.x, a.half_extents.y, a.half_extents.z];
    let h_b = [b.half_extents.x, b.half_extents.y, b.half_extents.z];
    let t_world = b.center - a.center;

    // R[i][j] = ax_a[i] · ax_b[j]; AbsR adds a small bias to avoid degenerate
    // axes when two boxes are exactly parallel.
    let mut r = [[0.0f32; 3]; 3];
    let mut abs_r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = ax_a[i].dot(ax_b[j]);
            abs_r[i][j] = r[i][j].abs() + 1e-6;
        }
    }
    // Translation expressed in A's local frame.
    let t = [
        t_world.dot(ax_a[0]),
        t_world.dot(ax_a[1]),
        t_world.dot(ax_a[2]),
    ];

    let mut min_overlap = f32::INFINITY;
    let mut best_normal = Vec3::Y;

    // Test the three face axes of A.
    for i in 0..3 {
        let ra = h_a[i];
        let rb = h_b[0] * abs_r[i][0] + h_b[1] * abs_r[i][1] + h_b[2] * abs_r[i][2];
        let dist = t[i].abs();
        let overlap = ra + rb - dist;
        if overlap < 0.0 {
            return None;
        }
        if overlap < min_overlap {
            min_overlap = overlap;
            // Orient the normal from A toward B along this axis.
            best_normal = ax_a[i] * if t[i] < 0.0 { -1.0 } else { 1.0 };
        }
    }

    // Test the three face axes of B.
    for i in 0..3 {
        let ra = h_a[0] * abs_r[0][i] + h_a[1] * abs_r[1][i] + h_a[2] * abs_r[2][i];
        let rb = h_b[i];
        // Project t onto ax_b[i]: that's the i-th column of R applied to t.
        let proj = t[0] * r[0][i] + t[1] * r[1][i] + t[2] * r[2][i];
        let dist = proj.abs();
        let overlap = ra + rb - dist;
        if overlap < 0.0 {
            return None;
        }
        if overlap < min_overlap {
            min_overlap = overlap;
            best_normal = ax_b[i] * if proj < 0.0 { -1.0 } else { 1.0 };
        }
    }

    // Test the nine cross-product axes (A.i × B.j). Skip degenerate axes where
    // the two parent axes are (nearly) parallel.
    for i in 0..3 {
        for j in 0..3 {
            let cross_world = ax_a[i].cross(ax_b[j]);
            let cross_len_sq = cross_world.length_squared();
            if cross_len_sq < 1e-6 {
                continue;
            }

            let i1 = (i + 1) % 3;
            let i2 = (i + 2) % 3;
            let j1 = (j + 1) % 3;
            let j2 = (j + 2) % 3;

            let ra = h_a[i1] * abs_r[i2][j] + h_a[i2] * abs_r[i1][j];
            let rb = h_b[j1] * abs_r[i][j2] + h_b[j2] * abs_r[i][j1];
            let dist_signed = t[i2] * r[i1][j] - t[i1] * r[i2][j];
            let dist = dist_signed.abs();
            let overlap = ra + rb - dist;
            if overlap < 0.0 {
                return None;
            }
            // Scale overlap to the unit-length cross axis (formula above is in
            // squared units of the parent axes) so face-axis vs edge-axis
            // comparisons are dimensionally consistent.
            let overlap_unit = overlap / cross_len_sq.sqrt();
            if overlap_unit < min_overlap {
                min_overlap = overlap_unit;
                let n = cross_world * (1.0 / cross_len_sq.sqrt());
                // Orient toward B.
                best_normal = if dist_signed < 0.0 { -n } else { n };
            }
        }
    }

    let mut c = Contact::new(ai, bi);
    c.normal = best_normal;
    c.penetration = min_overlap;
    // Contact point: midpoint between each OBB's closest point to the other's
    // center. This is a reasonable single-point approximation; a full
    // manifold would clip overlap polygons (future work).
    let pa = a.closest_point(b.center);
    let pb = b.closest_point(a.center);
    c.point = (pa + pb) * 0.5;
    Some(c)
}

/// AABB–AABB narrow phase using the axis of minimum penetration.
pub fn test_aabb_aabb(a: &Aabb3, b: &Aabb3, a_idx: usize, b_idx: usize) -> Option<Contact> {
    let ox = a.max.x.min(b.max.x) - a.min.x.max(b.min.x);
    let oy = a.max.y.min(b.max.y) - a.min.y.max(b.min.y);
    let oz = a.max.z.min(b.max.z) - a.min.z.max(b.min.z);
    if ox <= 0.0 || oy <= 0.0 || oz <= 0.0 {
        return None;
    }

    let mut c = Contact::new(a_idx, b_idx);
    let delta = b.center() - a.center();

    // Pick the axis with the smallest overlap.
    if ox <= oy && ox <= oz {
        c.penetration = ox;
        c.normal = Vec3::new(delta.x.signum(), 0.0, 0.0);
    } else if oy <= oz {
        c.penetration = oy;
        c.normal = Vec3::new(0.0, delta.y.signum(), 0.0);
    } else {
        c.penetration = oz;
        c.normal = Vec3::new(0.0, 0.0, delta.z.signum());
    }
    c.point = (a.center() + b.center()) * 0.5;
    Some(c)
}

/// Dispatch the right narrow-phase test for a pair of shapes and push the
/// resulting contact (if any) into `out`. Aabb-Obb pairs route through
/// OBB-OBB via [`Aabb3::to_obb`] so a single SAT routine handles every box.
pub fn test_pair(bodies: &[RigidBody], i: usize, j: usize, out: &mut Vec<Contact>) {
    let body_a = &bodies[i];
    let body_b = &bodies[j];
    if !body_a.enabled || !body_b.enabled {
        return;
    }
    if body_a.body_type() == BodyType::Static && body_b.body_type() == BodyType::Static {
        return;
    }

    let world_obb = |body: &RigidBody, o: &Obb| -> Obb {
        Obb::new(o.center + body.position, o.half_extents, o.rotation)
    };
    let world_aabb_as_obb =
        |body: &RigidBody, a: &Aabb3| -> Obb { a.translate(body.position).to_obb() };
    let world_sphere = |body: &RigidBody, s: &Sphere| -> Sphere {
        Sphere::new(body.position + s.center, s.radius)
    };

    let contact = match (&body_a.shape, &body_b.shape) {
        (CollisionShape::Sphere(sa), CollisionShape::Sphere(sb)) => {
            test_sphere_sphere(&world_sphere(body_a, sa), &world_sphere(body_b, sb), i, j)
        },
        (CollisionShape::Aabb(aa), CollisionShape::Aabb(ab)) => test_aabb_aabb(
            &aa.translate(body_a.position),
            &ab.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Sphere(s), CollisionShape::Aabb(a)) => test_sphere_aabb(
            &world_sphere(body_a, s),
            &a.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Aabb(a), CollisionShape::Sphere(s)) => test_sphere_aabb(
            &world_sphere(body_b, s),
            &a.translate(body_a.position),
            j,
            i,
        )
        .map(|mut c| {
            c.flip();
            c
        }),
        (CollisionShape::Sphere(s), CollisionShape::Obb(o)) => {
            test_sphere_obb(&world_sphere(body_a, s), &world_obb(body_b, o), i, j)
        },
        (CollisionShape::Obb(o), CollisionShape::Sphere(s)) => {
            test_sphere_obb(&world_sphere(body_b, s), &world_obb(body_a, o), j, i).map(|mut c| {
                c.flip();
                c
            })
        },
        (CollisionShape::Obb(oa), CollisionShape::Obb(ob)) => {
            test_obb_obb(&world_obb(body_a, oa), &world_obb(body_b, ob), i, j)
        },
        (CollisionShape::Aabb(aa), CollisionShape::Obb(ob)) => {
            test_obb_obb(&world_aabb_as_obb(body_a, aa), &world_obb(body_b, ob), i, j)
        },
        (CollisionShape::Obb(oa), CollisionShape::Aabb(ab)) => {
            test_obb_obb(&world_obb(body_a, oa), &world_aabb_as_obb(body_b, ab), i, j)
        },
    };

    if let Some(mut c) = contact {
        c.restitution = body_a.restitution.min(body_b.restitution);
        c.friction = (body_a.friction * body_b.friction).sqrt();
        if !body_a.is_sensor && !body_b.is_sensor {
            out.push(c);
        }
    }
}

/// O(N²) brute-force collision detector. Adequate for small scenes and
/// matches the 2D `SimpleCollisionDetector` design.
pub struct SimpleCollisionDetector;

impl SimpleCollisionDetector {
    /// Detect all colliding pairs and append contacts to `out`.
    pub fn detect(bodies: &[RigidBody], out: &mut Vec<Contact>) {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let aabb_a = bodies[i].world_aabb();
                let aabb_b = bodies[j].world_aabb();
                if !aabb_a.intersects(&aabb_b) {
                    continue;
                }
                test_pair(bodies, i, j, out);
            }
        }
    }
}

/// Broad-phase-accelerated collision detector using a [`SpatialHashGrid`].
///
/// Suitable when there are many bodies and most pairs are far apart. Owns
/// a reusable pair buffer so the steady-state simulation step doesn't
/// allocate.
///
/// [`SpatialHashGrid`]: crate::broad_phase::SpatialHashGrid
pub struct SpatialHashDetector {
    broad_phase: crate::broad_phase::SpatialHashGrid,
    pairs: Vec<(usize, usize)>,
}

impl SpatialHashDetector {
    /// Build a new detector with the given cell size (world units).
    pub fn new(cell_size: f32) -> Self {
        Self {
            broad_phase: crate::broad_phase::SpatialHashGrid::new(cell_size),
            pairs: Vec::new(),
        }
    }

    /// Detect collisions and push contacts into `out`.
    pub fn detect(&mut self, bodies: &[RigidBody], out: &mut Vec<Contact>) {
        use crate::broad_phase::BroadPhase;
        self.broad_phase.update(bodies);
        self.pairs.clear();
        self.broad_phase.get_potential_pairs(&mut self.pairs);
        for &(i, j) in &self.pairs {
            test_pair(bodies, i, j, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    #[test]
    fn sphere_sphere_separate_no_contact() {
        let a = Sphere::new(Vec3::ZERO, 1.0);
        let b = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);
        assert!(test_sphere_sphere(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn sphere_sphere_overlapping_contact_normal_points_a_to_b() {
        let a = Sphere::new(Vec3::ZERO, 1.0);
        let b = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let c = test_sphere_sphere(&a, &b, 0, 1).unwrap();
        assert!(approx(c.penetration, 0.5));
        assert!(vec_approx(c.normal, Vec3::X));
    }

    #[test]
    fn sphere_aabb_overlap_returns_normal_toward_box() {
        let s = Sphere::new(Vec3::new(-0.5, 0.0, 0.0), 1.0);
        let a = Aabb3::new(Vec3::ZERO, Vec3::splat(2.0));
        let c = test_sphere_aabb(&s, &a, 0, 1).unwrap();
        // Closest point on box to (-0.5, 0, 0) is (0, 0, 0); delta is (+0.5, 0, 0).
        assert!(vec_approx(c.normal, Vec3::X));
        assert!(approx(c.penetration, 0.5));
    }

    #[test]
    fn obb_obb_separate_no_contact() {
        let a = Obb::new(Vec3::ZERO, Vec3::splat(1.0), gravita_math::Quat::IDENTITY);
        let b = Obb::new(
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::splat(1.0),
            gravita_math::Quat::IDENTITY,
        );
        assert!(test_obb_obb(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn obb_obb_axis_aligned_matches_aabb_intuition() {
        // Two axis-aligned OBBs overlapping by 0.5 along X should produce a
        // contact with normal ±X and penetration 0.5.
        let a = Obb::new(Vec3::ZERO, Vec3::splat(1.0), gravita_math::Quat::IDENTITY);
        let b = Obb::new(
            Vec3::new(1.5, 0.0, 0.0),
            Vec3::splat(1.0),
            gravita_math::Quat::IDENTITY,
        );
        let c = test_obb_obb(&a, &b, 0, 1).unwrap();
        assert!(
            (c.penetration - 0.5).abs() < 1e-4,
            "penetration={}",
            c.penetration
        );
        assert!((c.normal.x.abs() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn obb_obb_rotated_45_overlaps_along_diagonal() {
        // Box B rotated 45° around Y intersects A along a non-axis-aligned
        // direction. SAT should still detect the overlap.
        use std::f32::consts::FRAC_PI_4;
        let a = Obb::new(Vec3::ZERO, Vec3::splat(1.0), gravita_math::Quat::IDENTITY);
        let b = Obb::new(
            Vec3::new(1.5, 0.0, 0.0),
            Vec3::splat(1.0),
            gravita_math::Quat::from_axis_angle(Vec3::Y, FRAC_PI_4),
        );
        assert!(test_obb_obb(&a, &b, 0, 1).is_some());
    }

    #[test]
    fn obb_obb_clearly_separated_rotated_returns_none() {
        use std::f32::consts::FRAC_PI_4;
        let a = Obb::new(Vec3::ZERO, Vec3::splat(1.0), gravita_math::Quat::IDENTITY);
        let b = Obb::new(
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::splat(1.0),
            gravita_math::Quat::from_axis_angle(Vec3::Y, FRAC_PI_4),
        );
        assert!(test_obb_obb(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn sphere_obb_overlap_uses_obb_closest_point() {
        // Sphere just outside the +X face of a unit OBB centered at origin.
        let s = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let o = Obb::new(Vec3::ZERO, Vec3::splat(1.0), gravita_math::Quat::IDENTITY);
        let c = test_sphere_obb(&s, &o, 0, 1).unwrap();
        assert!((c.penetration - 0.5).abs() < 1e-4);
        // Normal points from sphere toward box, i.e., -X.
        assert!((c.normal.x + 1.0).abs() < 1e-4);
    }

    #[test]
    fn sphere_obb_uses_oriented_closest_point() {
        // Rotate the OBB 45° around Y. At 45°, the OBB's +X corner is at
        // (√2, 0, 0). A sphere at (2.0, 0, 0) with radius 1 just clips the
        // corner; the same sphere position against an unrotated OBB would
        // miss (corner at (1, 0, 0), gap = 0). Test verifies orientation
        // actually changes the result.
        use std::f32::consts::FRAC_PI_4;
        let s = Sphere::new(Vec3::new(2.0, 0.0, 0.0), 1.0);
        let o = Obb::new(
            Vec3::ZERO,
            Vec3::splat(1.0),
            gravita_math::Quat::from_axis_angle(Vec3::Y, FRAC_PI_4),
        );
        let c = test_sphere_obb(&s, &o, 0, 1).unwrap();
        assert!(c.penetration > 0.0);
    }

    #[test]
    fn aabb_aabb_overlap_picks_minimum_axis() {
        // Two boxes overlapping mostly on X but with a thin Y overlap.
        let a = Aabb3::from_center_size(Vec3::ZERO, Vec3::new(4.0, 1.0, 4.0));
        let b = Aabb3::from_center_size(Vec3::new(0.0, 0.5, 0.0), Vec3::new(4.0, 1.0, 4.0));
        let c = test_aabb_aabb(&a, &b, 0, 1).unwrap();
        // Min overlap is along Y (0.5).
        assert!(approx(c.penetration, 0.5));
        assert!(vec_approx(c.normal, Vec3::Y));
    }

    #[test]
    fn detector_finds_overlap() {
        use crate::body::{CollisionShape as CS, RigidBody};
        let mut bodies = vec![
            RigidBody::new(0, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0))),
            RigidBody::new(1, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0)))
                .with_position(Vec3::new(1.0, 0.0, 0.0)),
        ];
        bodies[0].id = 0;
        bodies[1].id = 1;
        let mut contacts = Vec::new();
        SimpleCollisionDetector::detect(&bodies, &mut contacts);
        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].penetration > 0.0);
    }

    #[test]
    fn detector_skips_static_static() {
        use crate::body::{BodyType as BT, CollisionShape as CS, RigidBody};
        let bodies = vec![
            RigidBody::new(0, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0))).with_type(BT::Static),
            RigidBody::new(1, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0)))
                .with_type(BT::Static)
                .with_position(Vec3::new(0.5, 0.0, 0.0)),
        ];
        let mut contacts = Vec::new();
        SimpleCollisionDetector::detect(&bodies, &mut contacts);
        assert!(contacts.is_empty(), "static-static pair should be skipped");
    }
}
