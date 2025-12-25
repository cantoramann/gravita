use gravita_math::Vec2;

// physics/src/integrator.rs
use crate::body::RigidBody;

/// Trait for pluggable numerical integration schemes.
pub trait Integrator: Send + Sync {
    /// Integrate the velocity of a body.
    fn integrate_velocity(&mut self, body: &mut RigidBody, dt: f32);
    /// Integrate the position of a body.
    fn integrate_position(&mut self, body: &mut RigidBody, dt: f32);
}

/// Semi-implicit (symplectic) Euler integrator.
///
/// Simple and robust for most real-time games.
pub struct SemiImplicitEuler;

impl Integrator for SemiImplicitEuler {
    fn integrate_velocity(&mut self, body: &mut RigidBody, dt: f32) {
        // Linear velocity
        body.acceleration = body.force_accumulator * body.inv_mass;
        body.velocity += body.acceleration * dt;

        // Angular velocity
        body.angular_acceleration = body.torque_accumulator * body.inv_inertia;
        body.angular_velocity += body.angular_acceleration * dt;
    }

    fn integrate_position(&mut self, body: &mut RigidBody, dt: f32) {
        body.position += body.velocity * dt;
        body.rotation += body.angular_velocity * dt;
    }
}

/// Verlet integrator.
///
/// Uses previous positions to calculate velocity and position.
#[derive(Default)]
pub struct Verlet {
    /// Previous positions of the bodies.
    previous_positions: Vec<(usize, Vec2, f32)>, // (id, position, rotation)
}

impl Verlet {
    /// Create a new Verlet integrator with empty history.
    pub fn new() -> Self {
        Self::default()
    }

    fn get_or_init_previous(&mut self, body: &RigidBody) -> (Vec2, f32) {
        for (id, pos, rot) in &self.previous_positions {
            if *id == body.id {
                return (*pos, *rot);
            }
        }

        // Initialize with backward Euler
        let prev_pos = body.position - body.velocity * 0.016; // Assume 60 FPS
        let prev_rot = body.angular_velocity.mul_add(-0.016, body.rotation);
        self.previous_positions.push((body.id, prev_pos, prev_rot));
        (prev_pos, prev_rot)
    }

    fn update_previous(&mut self, body: &RigidBody) {
        for (id, pos, rot) in &mut self.previous_positions {
            if *id == body.id {
                *pos = body.position;
                *rot = body.rotation;
                return;
            }
        }
    }
}

impl Integrator for Verlet {
    fn integrate_velocity(&mut self, body: &mut RigidBody, _dt: f32) {
        // Verlet doesn't explicitly track velocity, but we update it for other systems
        body.acceleration = body.force_accumulator * body.inv_mass;
        body.angular_acceleration = body.torque_accumulator * body.inv_inertia;
    }

    fn integrate_position(&mut self, body: &mut RigidBody, dt: f32) {
        let (prev_pos, prev_rot) = self.get_or_init_previous(body);

        // Verlet integration: x(t+dt) = 2*x(t) - x(t-dt) + a*dt²
        let new_position = body.position * 2.0 - prev_pos + body.acceleration * dt * dt;
        let new_rotation =
            (body.angular_acceleration * dt).mul_add(dt, body.rotation.mul_add(2.0, -prev_rot));

        // Update velocity (for other systems that need it)
        body.velocity = (new_position - body.position) / dt;
        body.angular_velocity = (new_rotation - body.rotation) / dt;

        // Store current as previous for next frame
        self.update_previous(body);

        // Update position
        body.position = new_position;
        body.rotation = new_rotation;
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::Circle;

    use super::*;
    use crate::body::CollisionShape;

    const EPSILON: f32 = 1e-3;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn create_test_body() -> RigidBody {
        RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 10.0)))
    }

    // =========================================================================
    // Semi-Implicit Euler
    // =========================================================================

    #[test]
    fn euler_velocity_integration_from_force() {
        let mut integrator = SemiImplicitEuler;
        let mut body = create_test_body();
        body.force_accumulator = Vec2::new(100.0, 0.0); // 100N force
        body.mass = 10.0;
        body.inv_mass = 0.1;

        integrator.integrate_velocity(&mut body, 1.0);

        // a = F/m = 100/10 = 10 m/s²
        // v = a*t = 10 * 1 = 10 m/s
        assert!(approx_eq(body.velocity.x, 10.0));
    }

    #[test]
    fn euler_position_integration_from_velocity() {
        let mut integrator = SemiImplicitEuler;
        let mut body = create_test_body();
        body.velocity = Vec2::new(10.0, 0.0);

        integrator.integrate_position(&mut body, 1.0);

        // x = v*t = 10 * 1 = 10
        assert!(approx_eq(body.position.x, 10.0));
    }

    #[test]
    fn euler_angular_velocity_integration() {
        let mut integrator = SemiImplicitEuler;
        let mut body = create_test_body();
        body.torque_accumulator = 50.0;
        body.inertia = 10.0;
        body.inv_inertia = 0.1;

        integrator.integrate_velocity(&mut body, 1.0);

        // α = τ/I = 50/10 = 5 rad/s²
        // ω = α*t = 5 * 1 = 5 rad/s
        assert!(approx_eq(body.angular_velocity, 5.0));
    }

    #[test]
    fn euler_rotation_integration() {
        let mut integrator = SemiImplicitEuler;
        let mut body = create_test_body();
        body.angular_velocity = 2.0; // 2 rad/s

        integrator.integrate_position(&mut body, 0.5);

        // θ = ω*t = 2 * 0.5 = 1 rad
        assert!(approx_eq(body.rotation, 1.0));
    }

    #[test]
    fn euler_combined_linear_and_angular() {
        let mut integrator = SemiImplicitEuler;
        let mut body = create_test_body();

        body.force_accumulator = Vec2::new(10.0, 10.0);
        body.mass = 1.0;
        body.inv_mass = 1.0;
        body.torque_accumulator = 5.0;
        body.inertia = 1.0;
        body.inv_inertia = 1.0;

        integrator.integrate_velocity(&mut body, 1.0);
        integrator.integrate_position(&mut body, 1.0);

        assert!(approx_eq(body.velocity.x, 10.0));
        assert!(approx_eq(body.velocity.y, 10.0));
        assert!(approx_eq(body.angular_velocity, 5.0));
        assert!(approx_eq(body.position.x, 10.0));
        assert!(approx_eq(body.position.y, 10.0));
        assert!(approx_eq(body.rotation, 5.0));
    }

    // =========================================================================
    // Verlet Integrator
    // =========================================================================

    #[test]
    fn verlet_position_integration() {
        let mut integrator = Verlet::new();
        let mut body = create_test_body();
        body.velocity = Vec2::new(10.0, 0.0);
        body.position = Vec2::new(0.0, 0.0);

        // First step initializes previous position
        integrator.integrate_velocity(&mut body, 0.016);
        integrator.integrate_position(&mut body, 0.016);

        // Body should have moved
        assert!(body.position.x > 0.0);
    }

    #[test]
    fn verlet_maintains_velocity_estimate() {
        let mut integrator = Verlet::new();
        let mut body = create_test_body();
        body.velocity = Vec2::new(100.0, 0.0);
        body.position = Vec2::ZERO;

        integrator.integrate_velocity(&mut body, 0.016);
        integrator.integrate_position(&mut body, 0.016);

        // Velocity should still be approximately the same (accounting for numerical differences)
        assert!(body.velocity.x > 50.0);
    }

    #[test]
    fn verlet_handles_multiple_bodies() {
        let mut integrator = Verlet::new();

        let mut body1 = create_test_body();
        body1.id = 0;
        body1.velocity = Vec2::new(10.0, 0.0);

        let mut body2 = create_test_body();
        body2.id = 1;
        body2.velocity = Vec2::new(-10.0, 0.0);

        integrator.integrate_velocity(&mut body1, 0.016);
        integrator.integrate_position(&mut body1, 0.016);

        integrator.integrate_velocity(&mut body2, 0.016);
        integrator.integrate_position(&mut body2, 0.016);

        // Bodies should move in opposite directions
        assert!(body1.position.x > 0.0);
        assert!(body2.position.x < 0.0);
    }
}
