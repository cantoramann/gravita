// physics-3d/src/collision.rs

//! Narrow-phase 3D collision tests and the simple detector loop.

use gravita_math::{Aabb3, Sphere, Vec3};

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
pub fn test_sphere_sphere(
    a: &Sphere,
    b: &Sphere,
    a_idx: usize,
    b_idx: usize,
) -> Option<Contact> {
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

/// AABB–AABB narrow phase using the axis of minimum penetration.
pub fn test_aabb_aabb(
    a: &Aabb3,
    b: &Aabb3,
    a_idx: usize,
    b_idx: usize,
) -> Option<Contact> {
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
/// resulting contact (if any) into `out`.
pub fn test_pair(bodies: &[RigidBody], i: usize, j: usize, out: &mut Vec<Contact>) {
    let body_a = &bodies[i];
    let body_b = &bodies[j];
    if body_a.body_type() == BodyType::Static && body_b.body_type() == BodyType::Static {
        return;
    }

    let contact = match (&body_a.shape, &body_b.shape) {
        (CollisionShape::Sphere(sa), CollisionShape::Sphere(sb)) => test_sphere_sphere(
            &Sphere::new(body_a.position + sa.center, sa.radius),
            &Sphere::new(body_b.position + sb.center, sb.radius),
            i,
            j,
        ),
        (CollisionShape::Aabb(aa), CollisionShape::Aabb(ab)) => test_aabb_aabb(
            &aa.translate(body_a.position),
            &ab.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Sphere(s), CollisionShape::Aabb(a)) => test_sphere_aabb(
            &Sphere::new(body_a.position + s.center, s.radius),
            &a.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Aabb(a), CollisionShape::Sphere(s)) => test_sphere_aabb(
            &Sphere::new(body_b.position + s.center, s.radius),
            &a.translate(body_a.position),
            j,
            i,
        )
        .map(|mut c| {
            c.flip();
            c
        }),
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
            RigidBody::new(0, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0)))
                .with_type(BT::Static),
            RigidBody::new(1, CS::Sphere(Sphere::new(Vec3::ZERO, 1.0)))
                .with_type(BT::Static)
                .with_position(Vec3::new(0.5, 0.0, 0.0)),
        ];
        let mut contacts = Vec::new();
        SimpleCollisionDetector::detect(&bodies, &mut contacts);
        assert!(contacts.is_empty(), "static-static pair should be skipped");
    }
}
