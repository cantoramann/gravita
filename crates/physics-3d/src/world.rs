// physics-3d/src/world.rs

use gravita_math::Vec3;

use crate::{
    body::{BodyType, RigidBody},
    collision::{Contact, SimpleCollisionDetector},
    integrator::{Integrator, SemiImplicitEuler},
};

/// 3D physics world: owns bodies, gravity, integrator, and the per-step
/// collision pipeline.
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    gravity: Vec3,
    /// Integrator (default: [`SemiImplicitEuler`]).
    pub integrator: Box<dyn Integrator>,
    contacts: Vec<Contact>,
    /// Number of velocity-constraint iterations per step. Default `8`.
    pub velocity_iterations: usize,
    /// Position-correction fraction in `[0, 1]`. Default `0.2`.
    pub position_correction: f32,
    /// Velocity magnitude below which a body's velocities are snapped to zero.
    pub sleep_threshold: f32,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integrator: Box::new(SemiImplicitEuler),
            contacts: Vec::new(),
            velocity_iterations: 8,
            position_correction: 0.2,
            sleep_threshold: 0.01,
        }
    }
}

impl PhysicsWorld {
    /// Empty world with default integrator + Earth gravity along `-Y`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a body. The world overrides its `id` to its index in the body list.
    pub fn add_body(&mut self, mut body: RigidBody) -> usize {
        body.id = self.bodies.len();
        let id = body.id;
        self.bodies.push(body);
        id
    }

    /// Immutable body list.
    #[must_use]
    pub fn bodies(&self) -> &[RigidBody] {
        &self.bodies
    }

    /// Mutable body by id.
    pub fn body_mut(&mut self, id: usize) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id)
    }

    /// Set the world's linear gravity acceleration.
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// Step the simulation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        debug_assert!(dt > 0.0 && dt.is_finite(), "non-finite or non-positive dt: {dt}");

        for body in &mut self.bodies {
            body.clear_forces();
        }
        self.apply_gravity_and_damping();

        for body in &mut self.bodies {
            self.integrator.integrate_velocity(body, dt);
        }

        self.contacts.clear();
        SimpleCollisionDetector::detect(&self.bodies, &mut self.contacts);

        let mut contacts = std::mem::take(&mut self.contacts);
        for _ in 0..self.velocity_iterations {
            for contact in &mut contacts {
                Self::solve_velocity(&mut self.bodies, contact);
            }
        }

        for body in &mut self.bodies {
            self.integrator.integrate_position(body, dt);
        }

        for contact in &contacts {
            Self::solve_position(&mut self.bodies, contact, self.position_correction);
        }

        self.contacts = contacts;
        self.apply_sleeping();
    }

    fn apply_gravity_and_damping(&mut self) {
        for body in &mut self.bodies {
            if body.body_type() != BodyType::Dynamic {
                continue;
            }
            body.force_accumulator += self.gravity * body.mass() * body.gravity_scale;
            body.force_accumulator += -body.velocity * body.linear_damping;
            body.torque_accumulator += -body.angular_velocity * body.angular_damping;
        }
    }

    fn apply_sleeping(&mut self) {
        let lin_sq = self.sleep_threshold * self.sleep_threshold;
        let ang_sq = self.sleep_threshold * self.sleep_threshold;
        for body in &mut self.bodies {
            if body.body_type() != BodyType::Dynamic {
                continue;
            }
            if body.velocity.length_squared() < lin_sq
                && body.angular_velocity.length_squared() < ang_sq
            {
                body.velocity = Vec3::ZERO;
                body.angular_velocity = Vec3::ZERO;
            }
        }
    }

    fn solve_velocity(bodies: &mut [RigidBody], contact: &mut Contact) {
        // Below this normal-aligned closing speed, treat the impact as
        // inelastic to avoid micro-bouncing.
        const RESTITUTION_SLOP: f32 = 0.5;
        let (ia, ib) = (contact.body_a, contact.body_b);
        debug_assert!(ia != ib);

        let (a, b) = if ia < ib {
            let (left, right) = bodies.split_at_mut(ib);
            (&mut left[ia], &mut right[0])
        } else {
            let (left, right) = bodies.split_at_mut(ia);
            (&mut right[0], &mut left[ib])
        };

        let rv = b.velocity - a.velocity;
        let vn = rv.dot(contact.normal);
        if vn > 0.0 {
            return; // separating
        }

        let inv_mass_sum = a.inv_mass() + b.inv_mass();
        if inv_mass_sum == 0.0 {
            return;
        }

        let mut e = contact.restitution;
        if vn.abs() < RESTITUTION_SLOP {
            e = 0.0;
        }
        let j = -(1.0 + e) * vn / inv_mass_sum;
        let impulse = contact.normal * j;
        a.apply_impulse(-impulse);
        b.apply_impulse(impulse);
    }

    fn solve_position(
        bodies: &mut [RigidBody],
        contact: &Contact,
        position_correction: f32,
    ) {
        // Allow a small amount of overlap to keep stacks from jittering.
        const SLOP: f32 = 0.01;
        if contact.penetration <= SLOP {
            return;
        }

        let (ia, ib) = (contact.body_a, contact.body_b);
        debug_assert!(ia != ib);
        let (a, b) = if ia < ib {
            let (left, right) = bodies.split_at_mut(ib);
            (&mut left[ia], &mut right[0])
        } else {
            let (left, right) = bodies.split_at_mut(ia);
            (&mut right[0], &mut left[ib])
        };

        let inv_mass_sum = a.inv_mass() + b.inv_mass();
        if inv_mass_sum == 0.0 {
            return;
        }
        let correction =
            contact.normal * ((contact.penetration - SLOP).max(0.0) * position_correction / inv_mass_sum);
        a.position -= correction * a.inv_mass();
        b.position += correction * b.inv_mass();
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::Sphere;

    use super::*;
    use crate::body::{BodyType, CollisionShape, RigidBody};

    fn sphere_shape(r: f32) -> CollisionShape {
        CollisionShape::Sphere(Sphere::new(Vec3::ZERO, r))
    }

    #[test]
    fn new_world_is_empty() {
        let w = PhysicsWorld::new();
        assert!(w.bodies().is_empty());
    }

    #[test]
    fn gravity_pulls_bodies_down() {
        let mut w = PhysicsWorld::new();
        let id = w.add_body(RigidBody::new(0, sphere_shape(0.5)));
        for _ in 0..60 {
            w.step(1.0 / 60.0);
        }
        let b = w.body_mut(id).unwrap();
        // After 1 second of gravity ≈ 9.81 m/s², y should be near -4.9 m
        // (factoring in damping). Loose check: must have moved DOWN substantially.
        assert!(b.position.y < -3.0, "expected significant fall, got y={}", b.position.y);
    }

    #[test]
    fn sphere_rests_on_static_floor() {
        let mut w = PhysicsWorld::new();
        // Static floor: large AABB at y=-1 ± 0.5
        w.add_body(
            RigidBody::new(
                0,
                CollisionShape::Aabb(gravita_math::Aabb3::from_center_size(
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(100.0, 1.0, 100.0),
                )),
            )
            .with_type(BodyType::Static),
        );
        // Falling sphere above the floor.
        let mut ball = RigidBody::new(0, sphere_shape(0.5));
        ball.restitution = 0.0;
        ball.position = Vec3::new(0.0, 5.0, 0.0);
        let ball_id = w.add_body(ball);

        // Simulate 3 seconds.
        for _ in 0..(60 * 3) {
            w.step(1.0 / 60.0);
        }
        let b = w.body_mut(ball_id).unwrap();
        // Should have come to rest near the floor (y ≈ floor_top + radius)
        assert!(b.velocity.length() < 0.5, "sphere should be roughly at rest, v={:?}", b.velocity);
        // The floor top is at y = -1 + 0.5 = -0.5. The sphere's center at rest
        // would be approximately at -0.5 + 0.5 + slop ≈ ~0. Loose bound.
        assert!(
            b.position.y > -1.0 && b.position.y < 1.0,
            "sphere should be near the floor, y={}",
            b.position.y
        );
    }

    #[test]
    fn elastic_head_on_collision_swaps_velocities_qualitatively() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(Vec3::ZERO);

        let mut a = RigidBody::new(0, sphere_shape(0.5));
        a.position = Vec3::new(-2.0, 0.0, 0.0);
        a.velocity = Vec3::new(5.0, 0.0, 0.0);
        a.restitution = 1.0;
        let ai = w.add_body(a);

        let mut b = RigidBody::new(0, sphere_shape(0.5));
        b.position = Vec3::new(2.0, 0.0, 0.0);
        b.velocity = Vec3::new(-5.0, 0.0, 0.0);
        b.restitution = 1.0;
        let bi = w.add_body(b);

        for _ in 0..600 {
            w.step(1.0 / 60.0);
        }
        let a = &w.bodies()[ai];
        let b = &w.bodies()[bi];
        // After elastic head-on collision both should be moving outward.
        assert!(a.velocity.x < 0.0, "A should be moving left after bounce, v={:?}", a.velocity);
        assert!(b.velocity.x > 0.0, "B should be moving right after bounce, v={:?}", b.velocity);
    }
}
