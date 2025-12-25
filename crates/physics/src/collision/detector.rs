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

/// Simple collision detector without broad phase optimization.
pub struct SimpleCollisionDetector;

impl CollisionDetector for SimpleCollisionDetector {
    fn detect(&mut self, bodies: &[RigidBody], contacts: &mut Vec<Contact>) {
        // O(n²) check all pairs
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let body_a = &bodies[i];
                let body_b = &bodies[j];

                // Skip if both are static or sensors
                if body_a.body_type == BodyType::Static && body_b.body_type == BodyType::Static {
                    continue;
                }

                // Quick AABB check first
                let aabb_a = body_a.get_world_aabb();
                let aabb_b = body_b.get_world_aabb();

                if !aabb_a.intersects(&aabb_b) {
                    continue;
                }

                // Detailed collision test based on shapes
                let contact = match (&body_a.shape, &body_b.shape) {
                    (CollisionShape::Circle(ca), CollisionShape::Circle(cb)) => test_circle_circle(
                        &ca.translate(body_a.position),
                        &cb.translate(body_b.position),
                        i,
                        j,
                    ),
                    (CollisionShape::AABB(aa), CollisionShape::AABB(ab)) => test_aabb_aabb(
                        &aa.translate(body_a.position),
                        &ab.translate(body_b.position),
                        i,
                        j,
                    ),
                    (CollisionShape::Circle(c), CollisionShape::AABB(a)) => test_circle_aabb(
                        &c.translate(body_a.position),
                        &a.translate(body_b.position),
                        i,
                        j,
                    ),
                    (CollisionShape::AABB(a), CollisionShape::Circle(c)) => {
                        // Test with swapped order (circle first), then flip the result
                        test_circle_aabb(
                            &c.translate(body_b.position),
                            &a.translate(body_a.position),
                            j,
                            i,
                        )
                        .map(|mut contact| {
                            contact.flip(); // Flip because we swapped the order
                            contact
                        })
                    },
                };

                if let Some(mut contact) = contact {
                    // Combine material properties
                    contact.restitution = body_a.restitution.min(body_b.restitution);
                    contact.friction = (body_a.friction * body_b.friction).sqrt();

                    // Add to contacts if not a sensor collision
                    if !body_a.is_sensor && !body_b.is_sensor {
                        contacts.push(contact);
                    }
                }
            }
        }
    }
}

/// Optimized collision detector that uses a spatial hash grid as a broad phase.
pub struct SpatialHashDetector {
    /// The spatial hash grid used for broad phase culling.
    broad_phase: SpatialHashGrid,
}

impl SpatialHashDetector {
    /// Create a new detector with the given cell size for spatial hashing.
    pub fn new(cell_size: f32) -> Self {
        Self {
            broad_phase: SpatialHashGrid::new(cell_size),
        }
    }
}

impl CollisionDetector for SpatialHashDetector {
    fn detect(&mut self, bodies: &[RigidBody], contacts: &mut Vec<Contact>) {
        // Update broad phase
        self.broad_phase.update(bodies);

        // Get potential collision pairs
        let pairs = self.broad_phase.get_potential_pairs();

        // Test each potential pair
        for (i, j) in pairs {
            let body_a = &bodies[i];
            let body_b = &bodies[j];

            // Skip if both are static
            if body_a.body_type == BodyType::Static && body_b.body_type == BodyType::Static {
                continue;
            }

            // Detailed collision test
            let contact = match (&body_a.shape, &body_b.shape) {
                (CollisionShape::Circle(ca), CollisionShape::Circle(cb)) => test_circle_circle(
                    &ca.translate(body_a.position),
                    &cb.translate(body_b.position),
                    i,
                    j,
                ),
                (CollisionShape::AABB(aa), CollisionShape::AABB(ab)) => test_aabb_aabb(
                    &aa.translate(body_a.position),
                    &ab.translate(body_b.position),
                    i,
                    j,
                ),
                (CollisionShape::Circle(c), CollisionShape::AABB(a)) => test_circle_aabb(
                    &c.translate(body_a.position),
                    &a.translate(body_b.position),
                    i,
                    j,
                ),
                (CollisionShape::AABB(a), CollisionShape::Circle(c)) => {
                    // Test with swapped order (circle first), then flip the result
                    test_circle_aabb(
                        &c.translate(body_b.position),
                        &a.translate(body_a.position),
                        j,
                        i,
                    )
                    .map(|mut contact| {
                        contact.flip();
                        contact
                    })
                },
            };

            if let Some(mut contact) = contact {
                // Combine material properties
                contact.restitution = body_a.restitution.min(body_b.restitution);
                contact.friction = (body_a.friction * body_b.friction).sqrt();

                // Add to contacts if not a sensor collision
                if !body_a.is_sensor && !body_b.is_sensor {
                    contacts.push(contact);
                }
            }
        }
    }
}
