// physics/src/collision/contact.rs
use gravita_math::Vec2;

/// Event emitted when two bodies collide.
/// This is a higher-level abstraction than Contact, containing
/// the impulse magnitude for game logic (damage, sound, effects).
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// Index of first body in collision
    pub body_a: usize,
    /// Index of second body in collision
    pub body_b: usize,
    /// Contact point in world space
    pub point: Vec2,
    /// Contact normal (from A to B)
    pub normal: Vec2,
    /// Relative velocity at impact (before resolution)
    pub relative_velocity: f32,
    /// Impulse magnitude applied to resolve collision
    pub impulse_magnitude: f32,
}

impl CollisionEvent {
    /// Create a new collision event with default values.
    pub fn new(body_a: usize, body_b: usize) -> Self {
        Self {
            body_a,
            body_b,
            point: Vec2::ZERO,
            normal: Vec2::ZERO,
            relative_velocity: 0.0,
            impulse_magnitude: 0.0,
        }
    }
}

/// Collision contact point between two bodies.
///
/// Contains all information needed to resolve the collision.
#[derive(Debug, Clone)]
pub struct Contact {
    /// Index of the first body in the physics world.
    pub body_a: usize,
    /// Index of the second body in the physics world.
    pub body_b: usize,
    /// Contact point in world space.
    pub point: Vec2,
    /// Contact normal pointing from body A to body B.
    pub normal: Vec2,
    /// Penetration depth (positive when overlapping).
    pub penetration: f32,
    /// Combined coefficient of restitution.
    pub restitution: f32,
    /// Combined friction coefficient.
    pub friction: f32,
}

impl Contact {
    /// Create a new contact with default values.
    pub fn new(body_a: usize, body_b: usize) -> Self {
        Self {
            body_a,
            body_b,
            point: Vec2::ZERO,
            normal: Vec2::ZERO,
            penetration: 0.0,
            restitution: 0.0,
            friction: 0.0,
        }
    }

    /// Flip the contact (swap bodies and reverse normal)
    pub fn flip(&mut self) {
        std::mem::swap(&mut self.body_a, &mut self.body_b);
        self.normal = -self.normal;
    }
}

/// Manifold holds multiple contact points for stable collision.
///
/// Used for edge-edge and face-face collisions where multiple
/// contact points improve stability.
#[derive(Debug, Clone)]
pub struct ContactManifold {
    /// Index of the first body.
    pub body_a: usize,
    /// Index of the second body.
    pub body_b: usize,
    /// List of contact points.
    pub contacts: Vec<Contact>,
    /// Number of active contacts.
    pub contact_count: usize,
}

impl ContactManifold {
    /// Create a new empty manifold.
    pub fn new(body_a: usize, body_b: usize) -> Self {
        Self {
            body_a,
            body_b,
            contacts: Vec::with_capacity(2), // Most collisions have 1-2 points
            contact_count: 0,
        }
    }

    /// Add a contact point, merging with existing nearby contacts.
    pub fn add_contact(&mut self, contact: Contact) {
        // Merge similar contacts to avoid redundancy
        const MERGE_THRESHOLD: f32 = 0.01;

        for existing in &mut self.contacts {
            if existing.point.distance_squared(contact.point) < MERGE_THRESHOLD * MERGE_THRESHOLD {
                // Update existing contact with deeper penetration
                if contact.penetration > existing.penetration {
                    *existing = contact;
                }
                return;
            }
        }

        self.contacts.push(contact);
        self.contact_count = self.contacts.len();
    }

    /// Remove all contacts from the manifold.
    pub fn clear(&mut self) {
        self.contacts.clear();
        self.contact_count = 0;
    }
}
