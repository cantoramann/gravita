// physics-3d/src/integrator.rs

//! Semi-implicit Euler integrator with quaternion-based rotation update.

use gravita_math::{Quat, Vec3};

use crate::body::{BodyType, RigidBody};

/// Trait for pluggable integrators. Mirrors the 2D version but uses Vec3
/// linear and Vec3 angular velocity, with quaternion-based rotation update.
pub trait Integrator: Send + Sync {
    /// Integrate the body's velocities forward by `dt`.
    fn integrate_velocity(&mut self, body: &mut RigidBody, dt: f32);
    /// Integrate the body's position and rotation forward by `dt`.
    fn integrate_position(&mut self, body: &mut RigidBody, dt: f32);
}

/// Semi-implicit (symplectic) Euler.
pub struct SemiImplicitEuler;

impl Integrator for SemiImplicitEuler {
    fn integrate_velocity(&mut self, body: &mut RigidBody, dt: f32) {
        if body.body_type() != BodyType::Dynamic {
            return;
        }
        // Linear: a = F * m⁻¹; v += a * dt.
        let accel = body.force_accumulator * body.inv_mass;
        body.velocity += accel * dt;

        // Angular: α = I⁻¹ · τ (per-axis since I is diagonal); ω += α * dt.
        let t = body.torque_accumulator;
        let inv_i = body.inv_inertia;
        let alpha = Vec3::new(t.x * inv_i.x, t.y * inv_i.y, t.z * inv_i.z);
        body.angular_velocity += alpha * dt;
    }

    fn integrate_position(&mut self, body: &mut RigidBody, dt: f32) {
        if body.body_type() == BodyType::Static {
            return;
        }
        body.position += body.velocity * dt;
        body.rotation = integrate_quaternion(body.rotation, body.angular_velocity, dt);
    }
}

/// `q' = normalize(q + 0.5 · (ω, 0) · q · dt)`.
///
/// `ω` is the world-space angular velocity vector. Re-normalisation keeps
/// numerical drift bounded over long simulations.
fn integrate_quaternion(q: Quat, omega: Vec3, dt: f32) -> Quat {
    // Pure-vector quaternion (0, ω.x, ω.y, ω.z)
    let omega_q = Quat::from_xyzw(omega.x, omega.y, omega.z, 0.0);
    // dq/dt = 0.5 · ω_q · q
    let dq = omega_q * q;
    let half_dt = 0.5 * dt;
    let new_q = Quat::from_xyzw(
        dq.x.mul_add(half_dt, q.x),
        dq.y.mul_add(half_dt, q.y),
        dq.z.mul_add(half_dt, q.z),
        dq.w.mul_add(half_dt, q.w),
    );
    new_q.normalize()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use gravita_math::{Sphere, Vec3};

    use super::*;
    use crate::body::{CollisionShape, RigidBody};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
    }

    fn body() -> RigidBody {
        RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 1.0)))
    }

    #[test]
    fn euler_velocity_from_force() {
        let mut i = SemiImplicitEuler;
        let mut b = body();
        b.set_mass(2.0);
        b.force_accumulator = Vec3::new(10.0, 0.0, 0.0);
        i.integrate_velocity(&mut b, 1.0);
        // a = F/m = 5; v = a*t = 5
        assert!(approx(b.velocity.x, 5.0));
    }

    #[test]
    fn euler_position_from_velocity() {
        let mut i = SemiImplicitEuler;
        let mut b = body();
        b.velocity = Vec3::new(3.0, 0.0, 0.0);
        i.integrate_position(&mut b, 2.0);
        assert!(vec_approx(b.position, Vec3::new(6.0, 0.0, 0.0)));
    }

    #[test]
    fn quaternion_integration_y_axis_90_deg() {
        // ω = (0, ω, 0), dt small enough that t·ω ≈ π/2.
        let mut i = SemiImplicitEuler;
        let mut b = body();
        // 1 rad/sec around +Y for π/2 seconds.
        b.angular_velocity = Vec3::new(0.0, 1.0, 0.0);
        let steps = 1000;
        let dt = FRAC_PI_2 / steps as f32;
        for _ in 0..steps {
            i.integrate_position(&mut b, dt);
        }
        // After π/2 around Y, X should map to -Z.
        let rotated_x = b.rotation.rotate_vec(Vec3::X);
        assert!(
            vec_approx(rotated_x, Vec3::FORWARD),
            "got {rotated_x:?} expected {:?}",
            Vec3::FORWARD
        );
    }

    #[test]
    fn static_body_does_not_integrate_position() {
        let mut i = SemiImplicitEuler;
        let mut b = body().with_type(BodyType::Static);
        b.velocity = Vec3::new(100.0, 0.0, 0.0);
        i.integrate_position(&mut b, 1.0);
        assert_eq!(b.position, Vec3::ZERO);
    }

    #[test]
    fn dynamic_body_quaternion_stays_unit_after_long_run() {
        let mut i = SemiImplicitEuler;
        let mut b = body();
        b.angular_velocity = Vec3::new(0.5, 0.7, 0.3);
        for _ in 0..10_000 {
            i.integrate_position(&mut b, 1.0 / 60.0);
        }
        let len = b.rotation.length();
        assert!((len - 1.0).abs() < 1e-3, "rotation drifted: |q| = {len}");
    }
}
