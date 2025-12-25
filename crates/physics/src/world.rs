// physics/src/world.rs
use gravita_math::Vec2;

use crate::{
    body::{BodyType, RigidBody},
    collision::{CollisionDetector, CollisionEvent, Contact, SimpleCollisionDetector},
    integrator::{Integrator, SemiImplicitEuler},
};

/// High-level container responsible for stepping the physics simulation.
///
/// Owns all rigid bodies for a given scene plus solver and collision state.
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    gravity: Vec2,
    /// The integrator to use for the physics simulation.
    pub integrator: Box<dyn Integrator>,
    collision_detector: Box<dyn CollisionDetector>,
    contacts: Vec<Contact>,
    collision_events: Vec<CollisionEvent>,

    // Solver parameters
    /// Position correction factor in range [0, 1]. Higher values correct overlap
    /// more aggressively but may cause jitter. Default: 0.2.
    pub position_correction: f32,
    /// Number of velocity constraint iterations per step. More iterations give
    /// more accurate collision response. Default: 8.
    pub velocity_iterations: usize,
    /// Number of position constraint iterations per step. More iterations give
    /// better overlap resolution. Default: 3.
    pub position_iterations: usize,
    /// Velocity magnitude below which bodies are considered sleeping.
    /// Sleeping bodies skip integration for performance. Default: 0.01.
    pub sleep_threshold: f32,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: Vec2::new(0.0, -9.81),
            integrator: Box::new(SemiImplicitEuler),
            collision_detector: Box::new(SimpleCollisionDetector),
            contacts: Vec::new(),
            collision_events: Vec::new(),
            position_correction: 0.2,
            velocity_iterations: 8,
            position_iterations: 3,
            sleep_threshold: 0.01,
        }
    }
}

impl PhysicsWorld {
    /// Create a new empty world with default parameters and gravity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get collision events from the last physics step.
    /// These are cleared at the start of each step.
    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }

    /// Take ownership of collision events, clearing the internal buffer.
    pub fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        std::mem::take(&mut self.collision_events)
    }

    /// Set the global gravity acceleration applied to dynamic bodies.
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    /// Add a body to the world and return its assigned id.
    pub fn add_body(&mut self, mut body: RigidBody) -> usize {
        body.id = self.bodies.len();
        let id = body.id;
        self.bodies.push(body);
        id
    }

    /// Read-only slice of all bodies currently in the world.
    pub fn get_bodies(&self) -> &[RigidBody] {
        &self.bodies
    }

    /// Get an immutable reference to a body by id.
    pub fn get_body(&self, id: usize) -> Option<&RigidBody> {
        self.bodies.get(id)
    }

    /// Get a mutable reference to a body by id.
    pub fn get_body_mut(&mut self, id: usize) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id)
    }

    /// Mark a body for removal. The body will be disabled immediately
    /// (moved off-screen and made static) but not actually removed from
    /// the array to preserve indices.
    ///
    /// For games that need to spawn/despawn many bodies, consider using
    /// an object pool pattern instead.
    pub fn disable_body(&mut self, id: usize) {
        if let Some(body) = self.bodies.get_mut(id) {
            body.body_type = BodyType::Static;
            body.position = Vec2::new(-10000.0, -10000.0);
            body.velocity = Vec2::ZERO;
            body.angular_velocity = 0.0;
            body.is_sensor = true; // Won't generate collisions
        }
    }

    /// Check if a body is currently active (not disabled).
    pub fn is_body_active(&self, id: usize) -> bool {
        self.bodies
            .get(id)
            .is_some_and(|body| body.position.x > -5000.0) // Simple heuristic
    }

    /// Advance the simulation by `dt` seconds using a fixed pipeline:
    /// clear forces, apply gravity/forces, integrate, collide and solve.
    /// Advance the simulation by `dt` seconds.
    ///
    /// # Panics (debug builds only)
    ///
    /// Debug assertion fails if `dt` is not positive and finite.
    pub fn step(&mut self, dt: f32) {
        debug_assert!(
            dt > 0.0 && dt.is_finite(),
            "Time step must be positive and finite, got {dt}"
        );

        // Clear forces from last frame
        for body in &mut self.bodies {
            body.clear_forces();
        }

        // Apply gravity
        self.apply_gravity();

        // Apply other forces (springs, etc.)
        self.apply_forces();

        // Integrate velocities
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                self.integrator.integrate_velocity(body, dt);
            }
        }

        // Detect collisions
        self.contacts.clear();
        self.collision_events.clear();
        self.collision_detector
            .detect(&self.bodies, &mut self.contacts);

        // Take contacts out so we can mutably borrow bodies and contacts at the same time
        let mut contacts = std::mem::take(&mut self.contacts);

        // Solve constraints (velocity)
        // On first iteration, record collision events with impulse data
        for iteration in 0..self.velocity_iterations {
            for contact in &mut contacts {
                let impulse = Self::solve_velocity_constraint(&mut self.bodies, contact);

                // Record collision event on first iteration (when impulse is calculated)
                if iteration == 0 && impulse > 0.0 {
                    let event = CollisionEvent {
                        body_a: contact.body_a,
                        body_b: contact.body_b,
                        point: contact.point,
                        normal: contact.normal,
                        relative_velocity: 0.0, // Already resolved by this point
                        impulse_magnitude: impulse,
                    };
                    self.collision_events.push(event);
                }
            }
        }

        // Integrate positions
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                self.integrator.integrate_position(body, dt);
            }
        }

        // Solve position constraints (fix overlap)
        for _ in 0..self.position_iterations {
            for contact in &contacts {
                Self::solve_position_constraint(
                    &mut self.bodies,
                    contact,
                    self.position_correction,
                );
            }
        }

        // Store contacts back on the world for external querying/inspection
        self.contacts = contacts;

        // Snap very slow bodies to rest so they don't jitter or micro-bounce.
        self.apply_sleeping();
    }

    fn apply_gravity(&mut self) {
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                let gravity_force = self.gravity * body.mass * body.gravity_scale;
                body.apply_force(gravity_force);
            }
        }
    }

    fn apply_forces(&mut self) {
        // Apply damping
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                // Linear damping
                let drag_force = -body.velocity * body.linear_damping;
                body.apply_force(drag_force);

                // Angular damping
                body.torque_accumulator -= body.angular_velocity * body.angular_damping;
            }
        }
    }

    /// Zero out tiny velocities so objects come to rest visually.
    fn apply_sleeping(&mut self) {
        let linear_slop_sq = self.sleep_threshold * self.sleep_threshold;

        for body in &mut self.bodies {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            if body.velocity.length_squared() < linear_slop_sq
                && body.angular_velocity.abs() < self.sleep_threshold
            {
                body.velocity = Vec2::ZERO;
                body.angular_velocity = 0.0;
            }
        }
    }

    /// Solve velocity constraint for a contact pair.
    /// Returns the impulse magnitude applied (0.0 if no impulse was needed).
    fn solve_velocity_constraint(bodies: &mut [RigidBody], contact: &mut Contact) -> f32 {
        // Below this relative speed along the normal we treat the collision as inelastic
        // (no bounce). This helps objects settle visually instead of micro-bouncing forever.
        const RESTITUTION_VELOCITY_SLOP: f32 = 1.0;
        let (body_a_index, body_b_index) = (contact.body_a, contact.body_b);
        debug_assert!(body_a_index != body_b_index);

        // Safely get two distinct mutable references into the bodies slice
        let (body_a, body_b) = if body_a_index < body_b_index {
            let (first, second) = bodies.split_at_mut(body_b_index);
            (&mut first[body_a_index], &mut second[0])
        } else {
            let (first, second) = bodies.split_at_mut(body_a_index);
            (&mut second[0], &mut first[body_b_index])
        };

        // Calculate relative velocity
        let va = body_a.get_velocity_at_point(contact.point);
        let vb = body_b.get_velocity_at_point(contact.point);
        let relative_velocity = vb - va;

        // Velocity along normal
        let velocity_along_normal = relative_velocity.dot(contact.normal);

        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.0 {
            return 0.0;
        }

        // Calculate restitution (bounciness), but disable it for very slow impacts
        let mut e = body_a.restitution.min(body_b.restitution);
        if velocity_along_normal.abs() < RESTITUTION_VELOCITY_SLOP {
            e = 0.0;
        }

        // Calculate impulse scalar
        let ra = contact.point - body_a.position;
        let rb = contact.point - body_b.position;

        let ra_cross_n = ra.cross(contact.normal);
        let rb_cross_n = rb.cross(contact.normal);

        let inv_mass_sum = (rb_cross_n * rb_cross_n).mul_add(
            body_b.inv_inertia,
            (ra_cross_n * ra_cross_n)
                .mul_add(body_a.inv_inertia, body_a.inv_mass + body_b.inv_mass),
        );

        if inv_mass_sum == 0.0 {
            return 0.0;
        }

        let j = -(1.0 + e) * velocity_along_normal / inv_mass_sum;
        let impulse = contact.normal * j;
        let impulse_magnitude = j.abs();

        // Apply impulse to bodies
        let impulse_a = -impulse;
        let impulse_b = impulse;

        body_a.apply_impulse_at_point(impulse_a, contact.point);
        body_b.apply_impulse_at_point(impulse_b, contact.point);

        // Apply friction
        Self::apply_friction(bodies, contact, &impulse);

        impulse_magnitude
    }

    fn apply_friction(bodies: &mut [RigidBody], contact: &Contact, normal_impulse: &Vec2) {
        // Below this tangential speed we don't apply friction impulses.
        // This avoids tiny numerical noise from causing visible spinning.
        const FRICTION_VELOCITY_SLOP: f32 = 0.5;
        let (body_a_index, body_b_index) = (contact.body_a, contact.body_b);
        debug_assert!(body_a_index != body_b_index);

        let (body_a, body_b) = if body_a_index < body_b_index {
            let (first, second) = bodies.split_at_mut(body_b_index);
            (&mut first[body_a_index], &mut second[0])
        } else {
            let (first, second) = bodies.split_at_mut(body_a_index);
            (&mut second[0], &mut first[body_b_index])
        };

        // Calculate relative velocity
        let va = body_a.get_velocity_at_point(contact.point);
        let vb = body_b.get_velocity_at_point(contact.point);
        let relative_velocity = vb - va;

        // Tangent direction (perpendicular to normal)
        let tangent = relative_velocity - contact.normal * relative_velocity.dot(contact.normal);

        if tangent.length_squared() < 0.0001 {
            return;
        }

        let tangent = tangent.normalize();

        // Magnitude of friction
        let friction_coefficient = (body_a.friction * body_b.friction).sqrt();
        let max_friction = friction_coefficient * normal_impulse.length();

        // Calculate friction impulse
        let velocity_along_tangent = relative_velocity.dot(tangent);

        // Ignore very small tangential velocities to prevent micro-spinning.
        if velocity_along_tangent.abs() < FRICTION_VELOCITY_SLOP {
            return;
        }

        let ra = contact.point - body_a.position;
        let rb = contact.point - body_b.position;

        let ra_cross_t = ra.cross(tangent);
        let rb_cross_t = rb.cross(tangent);

        let inv_mass_sum = (rb_cross_t * rb_cross_t).mul_add(
            body_b.inv_inertia,
            (ra_cross_t * ra_cross_t)
                .mul_add(body_a.inv_inertia, body_a.inv_mass + body_b.inv_mass),
        );

        if inv_mass_sum == 0.0 {
            return;
        }

        let jt = -velocity_along_tangent / inv_mass_sum;
        let friction_impulse = tangent * jt.clamp(-max_friction, max_friction);

        body_a.apply_impulse_at_point(-friction_impulse, contact.point);
        body_b.apply_impulse_at_point(friction_impulse, contact.point);
    }

    fn solve_position_constraint(
        bodies: &mut [RigidBody],
        contact: &Contact,
        position_correction: f32,
    ) {
        // Allow a small amount of penetration so objects don't jitter or hover.
        // Only correct when we are deeper than this slop value.
        const PENETRATION_SLOP: f32 = 0.5;

        if contact.penetration <= PENETRATION_SLOP {
            return;
        }

        let (body_a_index, body_b_index) = (contact.body_a, contact.body_b);
        debug_assert!(body_a_index != body_b_index);

        let (body_a, body_b) = if body_a_index < body_b_index {
            let (first, second) = bodies.split_at_mut(body_b_index);
            (&mut first[body_a_index], &mut second[0])
        } else {
            let (first, second) = bodies.split_at_mut(body_a_index);
            (&mut second[0], &mut first[body_b_index])
        };

        let inv_mass_sum = body_a.inv_mass + body_b.inv_mass;
        if inv_mass_sum == 0.0 {
            return;
        }

        let corrected_penetration = contact.penetration - PENETRATION_SLOP;
        let correction =
            contact.normal * (corrected_penetration * position_correction / inv_mass_sum);

        body_a.position -= correction * body_a.inv_mass;
        body_b.position += correction * body_b.inv_mass;
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::{AABB, Circle};

    use super::*;
    use crate::body::CollisionShape;

    fn circle_shape(radius: f32) -> CollisionShape {
        CollisionShape::Circle(Circle::new(Vec2::ZERO, radius))
    }

    fn box_shape(width: f32, height: f32) -> CollisionShape {
        CollisionShape::AABB(AABB::from_center_size(Vec2::ZERO, Vec2::new(width, height)))
    }

    // =========================================================================
    // World Construction
    // =========================================================================

    #[test]
    fn new_world_is_empty() {
        let world = PhysicsWorld::new();
        assert!(world.get_bodies().is_empty());
    }

    #[test]
    fn new_world_has_default_gravity() {
        let world = PhysicsWorld::new();
        // Default gravity is (0, -9.81)
        // We can verify by checking body behavior after step
        let mut world = world;
        let id = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        world.step(1.0);
        let body = world.get_body(id).unwrap();
        // Body should have moved downward
        assert!(body.position.y < 0.0);
    }

    // =========================================================================
    // Body Management
    // =========================================================================

    #[test]
    fn add_body_returns_sequential_ids() {
        let mut world = PhysicsWorld::new();
        let id0 = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        let id1 = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        let id2 = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn get_body_returns_correct_body() {
        let mut world = PhysicsWorld::new();
        let body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::new(100.0, 200.0));
        let id = world.add_body(body);
        let retrieved = world.get_body(id).unwrap();
        assert_eq!(retrieved.position, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn get_body_mut_allows_modification() {
        let mut world = PhysicsWorld::new();
        let id = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        world.get_body_mut(id).unwrap().restitution = 0.8;
        assert_eq!(world.get_body(id).unwrap().restitution, 0.8);
    }

    #[test]
    fn get_nonexistent_body_returns_none() {
        let world = PhysicsWorld::new();
        assert!(world.get_body(999).is_none());
    }

    #[test]
    fn disable_body_moves_offscreen() {
        let mut world = PhysicsWorld::new();
        let id = world
            .add_body(RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::new(100.0, 100.0)));
        world.disable_body(id);
        let body = world.get_body(id).unwrap();
        assert!(body.position.x < -5000.0);
    }

    #[test]
    fn disabled_body_is_not_active() {
        let mut world = PhysicsWorld::new();
        let id = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        assert!(world.is_body_active(id));
        world.disable_body(id);
        assert!(!world.is_body_active(id));
    }

    // =========================================================================
    // Gravity
    // =========================================================================

    #[test]
    fn set_gravity_changes_simulation() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -100.0));
        let id = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        world.step(1.0);
        let body = world.get_body(id).unwrap();
        // With stronger gravity, body should fall faster
        assert!(body.velocity.y < -50.0);
    }

    #[test]
    fn zero_gravity_no_vertical_movement() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);
        let id = world.add_body(RigidBody::new(0, circle_shape(10.0)));
        world.step(1.0);
        let body = world.get_body(id).unwrap();
        // With zero gravity and no initial velocity, body stays put (almost)
        assert!(body.position.y.abs() < 1.0);
    }

    #[test]
    fn static_body_ignores_gravity() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -100.0));
        let id = world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_type(BodyType::Static)
                .with_position(Vec2::new(0.0, 100.0)),
        );
        world.step(1.0);
        let body = world.get_body(id).unwrap();
        assert_eq!(body.position, Vec2::new(0.0, 100.0));
    }

    // =========================================================================
    // Collision Detection
    // =========================================================================

    #[test]
    fn collision_events_generated_on_impact() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);

        // Two circles approaching each other
        world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::new(0.0, 0.0))
                .with_velocity(Vec2::new(100.0, 0.0)),
        );
        world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::new(15.0, 0.0))
                .with_velocity(Vec2::new(-100.0, 0.0)),
        );

        world.step(0.016);
        let events = world.get_collision_events();
        assert!(!events.is_empty());
    }

    #[test]
    fn drain_collision_events_clears_buffer() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);

        world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::ZERO)
                .with_velocity(Vec2::new(100.0, 0.0)),
        );
        world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::new(15.0, 0.0))
                .with_velocity(Vec2::new(-100.0, 0.0)),
        );

        world.step(0.016);
        let events = world.drain_collision_events();
        assert!(!events.is_empty());
        assert!(world.get_collision_events().is_empty());
    }

    // =========================================================================
    // Integration
    // =========================================================================

    #[test]
    fn body_with_velocity_moves() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);
        let id = world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::ZERO)
                .with_velocity(Vec2::new(100.0, 0.0)),
        );
        world.step(0.1);
        let body = world.get_body(id).unwrap();
        // Should have moved approximately 10 units (100 * 0.1, minus damping)
        assert!(body.position.x > 5.0);
    }

    #[test]
    fn kinematic_body_not_affected_by_forces() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -100.0));
        let id = world.add_body(
            RigidBody::new(0, circle_shape(10.0))
                .with_type(BodyType::Kinematic)
                .with_position(Vec2::new(0.0, 100.0)),
        );
        world.step(1.0);
        let body = world.get_body(id).unwrap();
        assert_eq!(body.position.y, 100.0);
    }

    // =========================================================================
    // Collision Response
    // =========================================================================

    #[test]
    fn dynamic_bodies_bounce_on_collision() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);

        // Two circles with high restitution colliding head-on
        let id1 = world.add_body({
            let mut b = RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::new(0.0, 0.0))
                .with_velocity(Vec2::new(50.0, 0.0));
            b.restitution = 1.0;
            b
        });
        let id2 = world.add_body({
            let mut b = RigidBody::new(0, circle_shape(10.0))
                .with_position(Vec2::new(15.0, 0.0))
                .with_velocity(Vec2::new(-50.0, 0.0));
            b.restitution = 1.0;
            b
        });

        // Multiple steps to ensure collision and response
        for _ in 0..10 {
            world.step(0.016);
        }

        let body1 = world.get_body(id1).unwrap();
        let body2 = world.get_body(id2).unwrap();

        // Bodies should have reversed direction (or close to it)
        assert!(body1.velocity.x < 0.0);
        assert!(body2.velocity.x > 0.0);
    }

    #[test]
    fn dynamic_body_stopped_by_static() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -100.0));

        // Static floor
        world.add_body(
            RigidBody::new(0, box_shape(200.0, 20.0))
                .with_type(BodyType::Static)
                .with_position(Vec2::new(0.0, 0.0)),
        );

        // Falling ball
        let ball_id = world.add_body({
            let mut b = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::new(0.0, 50.0));
            b.restitution = 0.0; // No bounce
            b
        });

        // Simulate for a while
        for _ in 0..100 {
            world.step(0.016);
        }

        let ball = world.get_body(ball_id).unwrap();
        // Ball should have come to rest on the floor
        assert!(ball.velocity.y.abs() < 1.0);
        // Ball should be near the floor (y ~ 10 + 10 = 20, accounting for floor height)
        assert!(ball.position.y > 5.0 && ball.position.y < 30.0);
    }

    // =========================================================================
    // Sleeping (Rest Detection)
    // =========================================================================

    #[test]
    fn slow_bodies_come_to_rest() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO);

        let id = world.add_body({
            let mut b =
                RigidBody::new(0, circle_shape(10.0)).with_velocity(Vec2::new(0.001, 0.001));
            b.linear_damping = 0.0; // Disable damping
            b
        });

        world.sleep_threshold = 0.01;
        world.step(0.016);

        let body = world.get_body(id).unwrap();
        // Very slow body should be zeroed out
        assert_eq!(body.velocity, Vec2::ZERO);
    }
}
