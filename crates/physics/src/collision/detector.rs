// physics/src/collision/detector.rs
use super::{
    broad_phase::{BroadPhase, SpatialHashGrid},
    contact::Contact,
    narrow_phase::{test_aabb_aabb, test_circle_aabb, test_circle_circle},
};
use crate::body::{BodyType, CollisionShape, RigidBody};

/// Trait for collision detection strategies.
///
/// Implementations are responsible for filling a list of contact points
/// between bodies each step.
pub trait CollisionDetector: Send + Sync {
    /// Detect collisions between all bodies and populate the contacts list.
    fn detect(&mut self, bodies: &[RigidBody], contacts: &mut Vec<Contact>);
}

/// Run the narrow-phase dispatch for a pair of bodies and push the resulting
/// contact (if any). Skips static-static pairs and sensor collisions.
fn test_pair(bodies: &[RigidBody], i: usize, j: usize, contacts: &mut Vec<Contact>) {
    let body_a = &bodies[i];
    let body_b = &bodies[j];

    if body_a.body_type == BodyType::Static && body_b.body_type == BodyType::Static {
        return;
    }

    let contact = match (&body_a.shape, &body_b.shape) {
        (CollisionShape::Circle(ca), CollisionShape::Circle(cb)) => test_circle_circle(
            &ca.translate(body_a.position),
            &cb.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Aabb(aa), CollisionShape::Aabb(ab)) => test_aabb_aabb(
            &aa.translate(body_a.position),
            &ab.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Circle(c), CollisionShape::Aabb(a)) => test_circle_aabb(
            &c.translate(body_a.position),
            &a.translate(body_b.position),
            i,
            j,
        ),
        (CollisionShape::Aabb(a), CollisionShape::Circle(c)) => test_circle_aabb(
            &c.translate(body_b.position),
            &a.translate(body_a.position),
            j,
            i,
        )
        .map(|mut contact| {
            contact.flip();
            contact
        }),
    };

    if let Some(mut contact) = contact {
        contact.restitution = body_a.restitution.min(body_b.restitution);
        contact.friction = (body_a.friction * body_b.friction).sqrt();
        if !body_a.is_sensor && !body_b.is_sensor {
            contacts.push(contact);
        }
    }
}

/// Simple collision detector without broad phase optimization.
pub struct SimpleCollisionDetector;

impl CollisionDetector for SimpleCollisionDetector {
    fn detect(&mut self, bodies: &[RigidBody], contacts: &mut Vec<Contact>) {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let aabb_a = bodies[i].get_world_aabb();
                let aabb_b = bodies[j].get_world_aabb();
                if !aabb_a.intersects(&aabb_b) {
                    continue;
                }
                test_pair(bodies, i, j, contacts);
            }
        }
    }
}

/// Optimized collision detector that uses a spatial hash grid as a broad phase.
///
/// Owns a reusable pair buffer so per-step allocations don't grow with body count.
pub struct SpatialHashDetector {
    broad_phase: SpatialHashGrid,
    pairs: Vec<(usize, usize)>,
}

impl SpatialHashDetector {
    /// Create a new detector with the given cell size for spatial hashing.
    pub fn new(cell_size: f32) -> Self {
        Self {
            broad_phase: SpatialHashGrid::new(cell_size),
            pairs: Vec::new(),
        }
    }
}

impl CollisionDetector for SpatialHashDetector {
    fn detect(&mut self, bodies: &[RigidBody], contacts: &mut Vec<Contact>) {
        self.broad_phase.update(bodies);
        self.pairs.clear();
        self.broad_phase.get_potential_pairs(&mut self.pairs);
        for &(i, j) in &self.pairs {
            test_pair(bodies, i, j, contacts);
        }
    }
}
