// physics/src/collision/narrow_phase.rs
use gravita_math::{AABB, Circle, Vec2};

use super::contact::Contact;

/// Test collision between two circles.
///
/// Returns a contact if the circles overlap, with the normal pointing from A to B.
pub fn test_circle_circle(
    circle_a: &Circle,
    circle_b: &Circle,
    body_a_idx: usize,
    body_b_idx: usize,
) -> Option<Contact> {
    let distance = circle_a.center.distance(circle_b.center);
    let radius_sum = circle_a.radius + circle_b.radius;

    if distance >= radius_sum {
        return None;
    }

    let mut contact = Contact::new(body_a_idx, body_b_idx);

    if distance < 0.0001 {
        // Circles are at same position, push apart arbitrarily
        contact.normal = Vec2::UP;
        contact.penetration = radius_sum;
        contact.point = circle_a.center;
    } else {
        // Normal points from A to B
        contact.normal = (circle_b.center - circle_a.center) / distance;
        contact.penetration = radius_sum - distance;

        // Contact point is between the circles
        contact.point =
            circle_a.center + contact.normal * contact.penetration.mul_add(-0.5, circle_a.radius);
    }

    Some(contact)
}

/// Test collision between two axis-aligned bounding boxes.
///
/// Uses the axis of minimum penetration to determine the contact normal.
pub fn test_aabb_aabb(
    aabb_a: &AABB,
    aabb_b: &AABB,
    body_a_idx: usize,
    body_b_idx: usize,
) -> Option<Contact> {
    // Calculate overlap on each axis
    let overlap_x = (aabb_a.max.x.min(aabb_b.max.x) - aabb_a.min.x.max(aabb_b.min.x)).max(0.0);
    let overlap_y = (aabb_a.max.y.min(aabb_b.max.y) - aabb_a.min.y.max(aabb_b.min.y)).max(0.0);

    if overlap_x == 0.0 || overlap_y == 0.0 {
        return None;
    }

    let mut contact = Contact::new(body_a_idx, body_b_idx);

    // Find the axis of minimum penetration
    let center_a = aabb_a.center();
    let center_b = aabb_b.center();
    let delta = center_b - center_a;

    if overlap_x < overlap_y {
        // Separate along X axis
        contact.penetration = overlap_x;
        contact.normal = Vec2::new(delta.x.signum(), 0.0);

        // Contact point is on the edge of AABB A
        let x = if delta.x > 0.0 {
            aabb_a.max.x
        } else {
            aabb_a.min.x
        };
        let y = center_a.y.clamp(aabb_b.min.y, aabb_b.max.y);
        contact.point = Vec2::new(x, y);
    } else {
        // Separate along Y axis
        contact.penetration = overlap_y;
        contact.normal = Vec2::new(0.0, delta.y.signum());

        // Contact point is on the edge of AABB A
        let x = center_a.x.clamp(aabb_b.min.x, aabb_b.max.x);
        let y = if delta.y > 0.0 {
            aabb_a.max.y
        } else {
            aabb_a.min.y
        };
        contact.point = Vec2::new(x, y);
    }

    Some(contact)
}

/// Test collision between a circle and an AABB.
///
/// The circle is treated as body A and the AABB as body B.
pub fn test_circle_aabb(
    circle: &Circle,
    aabb: &AABB,
    circle_idx: usize,
    aabb_idx: usize,
) -> Option<Contact> {
    // Find closest point on AABB to circle center
    let closest = Vec2::new(
        circle.center.x.clamp(aabb.min.x, aabb.max.x),
        circle.center.y.clamp(aabb.min.y, aabb.max.y),
    );

    let distance = circle.center.distance(closest);

    if distance >= circle.radius {
        return None;
    }

    let mut contact = Contact::new(circle_idx, aabb_idx);

    if distance < 0.0001 {
        // Circle center is inside AABB, find closest edge
        let to_min = circle.center - aabb.min;
        let to_max = aabb.max - circle.center;

        let min_dist = to_min.x.min(to_min.y).min(to_max.x).min(to_max.y);

        if min_dist == to_min.x {
            contact.normal = Vec2::LEFT;
            contact.penetration = to_min.x + circle.radius;
            contact.point = Vec2::new(aabb.min.x, circle.center.y);
        } else if min_dist == to_max.x {
            contact.normal = Vec2::RIGHT;
            contact.penetration = to_max.x + circle.radius;
            contact.point = Vec2::new(aabb.max.x, circle.center.y);
        } else if min_dist == to_min.y {
            contact.normal = Vec2::DOWN;
            contact.penetration = to_min.y + circle.radius;
            contact.point = Vec2::new(circle.center.x, aabb.min.y);
        } else {
            contact.normal = Vec2::UP;
            contact.penetration = to_max.y + circle.radius;
            contact.point = Vec2::new(circle.center.x, aabb.max.y);
        }
    } else {
        // Circle center is outside AABB.
        // Normal should point from circle (body A) towards the AABB (body B),
        // to stay consistent with the convention "normal points from A to B".
        contact.normal = (closest - circle.center).normalize();
        contact.penetration = circle.radius - distance;
        contact.point = closest;
    }

    Some(contact)
}

/// SAT (Separating Axis Theorem) test for polygons - useful for future extension
pub struct SATTest;

impl SATTest {
    /// Get support point - furthest point in given direction
    pub fn get_support(vertices: &[Vec2], direction: Vec2) -> Vec2 {
        let mut best_point = vertices[0];
        let mut best_projection = vertices[0].dot(direction);

        for vertex in vertices.iter().skip(1) {
            let projection = vertex.dot(direction);
            if projection > best_projection {
                best_projection = projection;
                best_point = *vertex;
            }
        }

        best_point
    }

    /// Project shape onto axis
    pub fn project(vertices: &[Vec2], axis: Vec2) -> (f32, f32) {
        let mut min = vertices[0].dot(axis);
        let mut max = min;

        for vertex in vertices.iter().skip(1) {
            let projection = vertex.dot(axis);
            min = min.min(projection);
            max = max.max(projection);
        }

        (min, max)
    }

    /// Check if projections overlap
    pub fn overlap(proj_a: (f32, f32), proj_b: (f32, f32)) -> Option<f32> {
        let overlap = proj_a.1.min(proj_b.1) - proj_a.0.max(proj_b.0);
        if overlap > 0.0 { Some(overlap) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // =========================================================================
    // Circle-Circle Collision
    // =========================================================================

    #[test]
    fn circle_circle_no_collision_when_separated() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(25.0, 0.0), 10.0);
        assert!(test_circle_circle(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn circle_circle_collision_when_overlapping() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(15.0, 0.0), 10.0);
        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        assert_eq!(contact.body_a, 0);
        assert_eq!(contact.body_b, 1);
    }

    #[test]
    fn circle_circle_collision_normal_points_from_a_to_b() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(15.0, 0.0), 10.0);
        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        // Normal should point from A to B (positive X direction)
        assert!(vec_approx_eq(contact.normal, Vec2::RIGHT));
    }

    #[test]
    fn circle_circle_penetration_depth() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(15.0, 0.0), 10.0);
        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        // Penetration = (10 + 10) - 15 = 5
        assert!(approx_eq(contact.penetration, 5.0));
    }

    #[test]
    fn circle_circle_touching_exactly() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::new(20.0, 0.0), 10.0);
        // Just touching (distance = sum of radii)
        assert!(test_circle_circle(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn circle_circle_concentric_collision() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        let b = Circle::new(Vec2::ZERO, 5.0);
        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        // When concentric, normal defaults to UP
        assert_eq!(contact.normal, Vec2::UP);
        assert!(approx_eq(contact.penetration, 15.0));
    }

    // =========================================================================
    // AABB-AABB Collision
    // =========================================================================

    #[test]
    fn aabb_aabb_no_collision_when_separated() {
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(20.0, 0.0), Vec2::new(30.0, 10.0));
        assert!(test_aabb_aabb(&a, &b, 0, 1).is_none());
    }

    #[test]
    fn aabb_aabb_collision_when_overlapping() {
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(5.0, 0.0), Vec2::new(15.0, 10.0));
        let contact = test_aabb_aabb(&a, &b, 0, 1).unwrap();
        assert_eq!(contact.body_a, 0);
        assert_eq!(contact.body_b, 1);
    }

    #[test]
    fn aabb_aabb_minimum_penetration_axis() {
        // Box A: 0-10 on X, 0-10 on Y
        // Box B: 8-18 on X, 0-10 on Y (overlaps 2 on X, 10 on Y)
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(8.0, 0.0), Vec2::new(18.0, 10.0));
        let contact = test_aabb_aabb(&a, &b, 0, 1).unwrap();
        // Should separate along X (smaller overlap)
        assert!(approx_eq(contact.penetration, 2.0));
        assert!(vec_approx_eq(contact.normal, Vec2::RIGHT));
    }

    #[test]
    fn aabb_aabb_vertical_separation() {
        // Box A: 0-10 on X, 0-10 on Y
        // Box B: 0-10 on X, 8-18 on Y (overlaps 10 on X, 2 on Y)
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(0.0, 8.0), Vec2::new(10.0, 18.0));
        let contact = test_aabb_aabb(&a, &b, 0, 1).unwrap();
        // Should separate along Y (smaller overlap)
        assert!(approx_eq(contact.penetration, 2.0));
        assert!(vec_approx_eq(contact.normal, Vec2::UP));
    }

    // =========================================================================
    // Circle-AABB Collision
    // =========================================================================

    #[test]
    fn circle_aabb_no_collision_when_separated() {
        let circle = Circle::new(Vec2::new(-15.0, 5.0), 5.0);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(test_circle_aabb(&circle, &aabb, 0, 1).is_none());
    }

    #[test]
    fn circle_aabb_collision_from_left() {
        let circle = Circle::new(Vec2::new(-3.0, 5.0), 5.0);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let contact = test_circle_aabb(&circle, &aabb, 0, 1).unwrap();
        assert_eq!(contact.body_a, 0);
        assert_eq!(contact.body_b, 1);
    }

    #[test]
    fn circle_aabb_collision_from_corner() {
        // Circle near corner of AABB
        let circle = Circle::new(Vec2::new(-3.0, -3.0), 5.0);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let contact = test_circle_aabb(&circle, &aabb, 0, 1);
        // Distance from circle center to corner (0,0) = sqrt(18) ≈ 4.24 < 5
        assert!(contact.is_some());
    }

    #[test]
    fn circle_aabb_no_collision_at_corner() {
        // Circle too far from corner
        let circle = Circle::new(Vec2::new(-5.0, -5.0), 5.0);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        // Distance from circle center to corner (0,0) = sqrt(50) ≈ 7.07 > 5
        assert!(test_circle_aabb(&circle, &aabb, 0, 1).is_none());
    }

    #[test]
    fn circle_inside_aabb_collision() {
        let circle = Circle::new(Vec2::new(5.0, 5.0), 2.0);
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let contact = test_circle_aabb(&circle, &aabb, 0, 1).unwrap();
        // Circle is fully inside, should still detect collision
        assert!(contact.penetration > 0.0);
    }

    // =========================================================================
    // SAT Helper Functions
    // =========================================================================

    #[test]
    fn sat_get_support_finds_furthest_point() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let support = SATTest::get_support(&vertices, Vec2::RIGHT);
        // Should find (10, 0) or (10, 10) as they have max X
        assert_eq!(support.x, 10.0);
    }

    #[test]
    fn sat_project_finds_min_max() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let (min, max) = SATTest::project(&vertices, Vec2::RIGHT);
        assert_eq!(min, 0.0);
        assert_eq!(max, 10.0);
    }

    #[test]
    fn sat_overlap_when_projections_overlap() {
        let proj_a = (0.0, 10.0);
        let proj_b = (5.0, 15.0);
        let overlap = SATTest::overlap(proj_a, proj_b).unwrap();
        assert_eq!(overlap, 5.0);
    }

    #[test]
    fn sat_no_overlap_when_separated() {
        let proj_a = (0.0, 10.0);
        let proj_b = (15.0, 25.0);
        assert!(SATTest::overlap(proj_a, proj_b).is_none());
    }

    // =========================================================================
    // Floating-Point Precision Tests
    // =========================================================================

    #[test]
    fn precision_circle_circle_near_touching() {
        // Circles that are VERY close to touching but not quite
        let a = Circle::new(Vec2::ZERO, 10.0);
        // Distance = 20.0001, sum of radii = 20, so gap of 0.0001
        let b = Circle::new(Vec2::new(20.0001, 0.0), 10.0);
        assert!(
            test_circle_circle(&a, &b, 0, 1).is_none(),
            "Should not collide when gap is 0.0001"
        );
    }

    #[test]
    fn precision_circle_circle_barely_overlapping() {
        let a = Circle::new(Vec2::ZERO, 10.0);
        // Distance = 19.9999, sum of radii = 20, so overlap of 0.0001
        let b = Circle::new(Vec2::new(19.9999, 0.0), 10.0);
        let contact = test_circle_circle(&a, &b, 0, 1);
        assert!(contact.is_some(), "Should collide when overlap is 0.0001");
        assert!(
            contact.unwrap().penetration < 0.001,
            "Penetration should be tiny"
        );
    }

    #[test]
    fn precision_circle_circle_diagonal_positions() {
        // Circles positioned along diagonal
        let a = Circle::new(Vec2::ZERO, 10.0);
        let diagonal_dist = 15.0; // Will overlap
        let offset = diagonal_dist / 2.0_f32.sqrt();
        let b = Circle::new(Vec2::new(offset, offset), 10.0);

        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();

        // Verify normal is along diagonal
        let expected_normal = Vec2::new(1.0, 1.0).normalize();
        assert!(
            (contact.normal.x - expected_normal.x).abs() < EPSILON,
            "Diagonal normal X incorrect"
        );
        assert!(
            (contact.normal.y - expected_normal.y).abs() < EPSILON,
            "Diagonal normal Y incorrect"
        );
    }

    #[test]
    fn precision_circle_circle_very_small_radii() {
        let a = Circle::new(Vec2::ZERO, 0.001);
        let b = Circle::new(Vec2::new(0.001, 0.0), 0.001);

        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        assert!(
            contact.penetration > 0.0,
            "Should detect collision with tiny circles"
        );
        assert!(
            contact.penetration < 0.002,
            "Penetration should be proportionally tiny"
        );
    }

    #[test]
    fn precision_circle_circle_very_large_radii() {
        let a = Circle::new(Vec2::ZERO, 1e6);
        let b = Circle::new(Vec2::new(1.5e6, 0.0), 1e6);

        let contact = test_circle_circle(&a, &b, 0, 1).unwrap();
        let expected_penetration = 5e5; // 2e6 - 1.5e6 = 0.5e6
        assert!(
            (contact.penetration - expected_penetration).abs() < 1000.0,
            "Large circle penetration: got {}, expected {}",
            contact.penetration,
            expected_penetration
        );
    }

    #[test]
    fn precision_aabb_aabb_near_touching() {
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        // Gap of 0.0001 on X axis
        let b = AABB::new(Vec2::new(10.0001, 0.0), Vec2::new(20.0, 10.0));
        assert!(
            test_aabb_aabb(&a, &b, 0, 1).is_none(),
            "Should not collide with tiny gap"
        );
    }

    #[test]
    fn precision_aabb_aabb_barely_overlapping() {
        let a = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        // Overlap of 0.0001 on X axis
        let b = AABB::new(Vec2::new(9.9999, 0.0), Vec2::new(20.0, 10.0));
        let contact = test_aabb_aabb(&a, &b, 0, 1);
        assert!(contact.is_some(), "Should collide with tiny overlap");
    }

    #[test]
    fn precision_circle_aabb_corner_exact() {
        // Circle positioned exactly at corner distance
        let aabb = AABB::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let _corner = Vec2::ZERO;
        let radius = 5.0;
        // Position circle so its edge just touches corner
        let offset = radius * 2.0_f32.sqrt();
        let circle = Circle::new(
            Vec2::new(-offset / 2.0_f32.sqrt(), -offset / 2.0_f32.sqrt()),
            radius,
        );

        let contact = test_circle_aabb(&circle, &aabb, 0, 1);
        // Should be right at the boundary - may or may not detect depending on floating point
        // This tests that we handle the edge case without crashing
        if let Some(c) = contact {
            assert!(
                c.penetration.abs() < 0.01,
                "Corner touch should have near-zero penetration"
            );
        }
    }

    #[test]
    fn precision_accumulated_collision_positions() {
        // Simulate accumulated position error from integration
        let mut pos = Vec2::ZERO;
        let step = Vec2::new(0.1, 0.1);

        // Accumulate 1000 small position changes
        for _ in 0..1000 {
            pos += step;
        }

        // pos should be approximately (100, 100)
        let circle = Circle::new(pos, 10.0);
        let expected_pos = Vec2::new(100.0, 100.0);
        let target = Circle::new(expected_pos, 10.0);

        // Should detect collision (same circle, allowing for accumulated error)
        let contact = test_circle_circle(&circle, &target, 0, 1);
        assert!(
            contact.is_some(),
            "Accumulated position should still detect self-collision"
        );
    }

    #[test]
    fn precision_normal_is_normalized() {
        // Verify collision normals are always unit length
        let test_cases = [
            (
                Circle::new(Vec2::ZERO, 10.0),
                Circle::new(Vec2::new(15.0, 0.0), 10.0),
            ),
            (
                Circle::new(Vec2::ZERO, 10.0),
                Circle::new(Vec2::new(10.0, 10.0), 10.0),
            ),
            (
                Circle::new(Vec2::ZERO, 100.0),
                Circle::new(Vec2::new(50.0, 50.0), 100.0),
            ),
        ];

        for (a, b) in &test_cases {
            if let Some(contact) = test_circle_circle(a, b, 0, 1) {
                let normal_len = contact.normal.length();
                assert!(
                    (normal_len - 1.0).abs() < EPSILON,
                    "Normal should be unit length, got {normal_len}"
                );
            }
        }
    }

    #[test]
    fn precision_penetration_non_negative() {
        // Penetration should never be negative
        let test_cases = [
            (
                Circle::new(Vec2::ZERO, 10.0),
                Circle::new(Vec2::new(15.0, 0.0), 10.0),
            ),
            (Circle::new(Vec2::ZERO, 10.0), Circle::new(Vec2::ZERO, 5.0)), // Concentric
            (
                Circle::new(Vec2::new(1e-6, 0.0), 10.0),
                Circle::new(Vec2::ZERO, 10.0),
            ), // Nearly concentric
        ];

        for (a, b) in &test_cases {
            if let Some(contact) = test_circle_circle(a, b, 0, 1) {
                assert!(
                    contact.penetration >= 0.0,
                    "Penetration should be non-negative, got {}",
                    contact.penetration
                );
            }
        }
    }
}
