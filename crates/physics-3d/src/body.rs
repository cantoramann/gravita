// physics-3d/src/body.rs

use gravita_math::{Aabb3, Obb, Quat, Sphere, Vec3};

/// How a body participates in the simulation.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BodyType {
    /// Never moved by the solver.
    Static,
    /// Moved by user code, not by forces.
    Kinematic,
    /// Fully simulated by forces, collisions, and constraints.
    Dynamic,
}

/// Geometric shape used for 3D collision detection.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CollisionShape {
    /// Sphere.
    Sphere(Sphere),
    /// World-axis-aligned box.
    Aabb(Aabb3),
    /// Oriented bounding box in body-local space. `center` and `rotation` are
    /// relative to the body — they're combined with the body's position/
    /// rotation at collision time.
    Obb(Obb),
}

impl CollisionShape {
    /// Tight world-space AABB after translating by `position`.
    pub fn world_aabb(&self, position: Vec3) -> Aabb3 {
        match self {
            Self::Sphere(s) => Sphere::new(position + s.center, s.radius).to_aabb(),
            Self::Aabb(b) => b.translate(position),
            Self::Obb(o) => Obb::new(o.center + position, o.half_extents, o.rotation).to_aabb(),
        }
    }

    /// Validity check. Used by `debug_assert!` in `RigidBody::new`.
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Sphere(s) => s.radius > 0.0 && s.radius.is_finite(),
            Self::Aabb(b) => {
                b.min.x < b.max.x
                    && b.min.y < b.max.y
                    && b.min.z < b.max.z
                    && b.min.x.is_finite()
                    && b.max.z.is_finite()
            },
            Self::Obb(o) => {
                o.half_extents.x > 0.0
                    && o.half_extents.y > 0.0
                    && o.half_extents.z > 0.0
                    && o.half_extents.x.is_finite()
                    && o.half_extents.y.is_finite()
                    && o.half_extents.z.is_finite()
            },
        }
    }

    /// Mass for the given uniform `density`.
    pub fn mass(&self, density: f32) -> f32 {
        match self {
            Self::Sphere(s) => {
                // m = ρ · (4/3) · π · r³
                density * (4.0 / 3.0) * std::f32::consts::PI * s.radius * s.radius * s.radius
            },
            Self::Aabb(b) => {
                let size = b.size();
                density * size.x * size.y * size.z
            },
            Self::Obb(o) => density * 8.0 * o.half_extents.x * o.half_extents.y * o.half_extents.z,
        }
    }

    /// Diagonal inertia tensor (Vec3) for the given `mass`. Body-local
    /// principal axes are aligned to the world axes.
    pub fn inertia(&self, mass: f32) -> Vec3 {
        match self {
            Self::Sphere(s) => {
                // I = (2/5) · m · r²
                let i = 0.4 * mass * s.radius * s.radius;
                Vec3::splat(i)
            },
            Self::Aabb(b) => {
                let size = b.size();
                let (w, h, d) = (size.x, size.y, size.z);
                let k = mass / 12.0;
                Vec3::new(
                    k * h.mul_add(h, d * d),
                    k * w.mul_add(w, d * d),
                    k * w.mul_add(w, h * h),
                )
            },
            Self::Obb(o) => {
                let (w, h, d) = (
                    o.half_extents.x * 2.0,
                    o.half_extents.y * 2.0,
                    o.half_extents.z * 2.0,
                );
                let k = mass / 12.0;
                Vec3::new(
                    k * h.mul_add(h, d * d),
                    k * w.mul_add(w, d * d),
                    k * w.mul_add(w, h * h),
                )
            },
        }
    }
}

/// 3D rigid body with quaternion rotation and diagonal inertia tensor.
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// Unique identifier within the physics world.
    pub id: usize,
    pub(crate) body_type: BodyType,

    /// World-space position of the body's center of mass.
    pub position: Vec3,
    /// Orientation as a unit quaternion.
    pub rotation: Quat,

    /// Linear velocity (units/sec).
    pub velocity: Vec3,
    /// Angular velocity (radians/sec, in world space).
    pub angular_velocity: Vec3,

    pub(crate) force_accumulator: Vec3,
    pub(crate) torque_accumulator: Vec3,

    pub(crate) mass: f32,
    pub(crate) inv_mass: f32,
    pub(crate) inertia: Vec3,
    pub(crate) inv_inertia: Vec3,

    /// Restitution coefficient `[0, 1]`.
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Linear damping per step.
    pub linear_damping: f32,
    /// Angular damping per step.
    pub angular_damping: f32,
    /// Multiplier on world gravity (1.0 by default).
    pub gravity_scale: f32,

    /// Shape used for collision.
    pub shape: CollisionShape,

    /// Sensor mode (detect overlap but skip resolution).
    pub is_sensor: bool,
}

impl RigidBody {
    /// Create a new dynamic body with unit mass.
    ///
    /// # Panics (debug builds only)
    ///
    /// Debug assertion fails if the shape is invalid (zero/negative radius,
    /// non-finite bounds, etc.).
    pub fn new(id: usize, shape: CollisionShape) -> Self {
        debug_assert!(shape.is_valid(), "invalid 3D collision shape: {shape:?}");
        let mass = 1.0;
        let inertia = shape.inertia(mass);
        Self {
            id,
            body_type: BodyType::Dynamic,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass,
            inv_mass: 1.0 / mass,
            inertia,
            inv_inertia: Vec3::new(1.0 / inertia.x, 1.0 / inertia.y, 1.0 / inertia.z),
            restitution: 0.5,
            friction: 0.3,
            linear_damping: 0.01,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            shape,
            is_sensor: false,
        }
    }

    /// Builder: override [`BodyType`].
    #[must_use]
    pub fn with_type(mut self, body_type: BodyType) -> Self {
        self.body_type = body_type;
        self.update_mass_properties();
        self
    }

    /// Builder: override initial world position.
    #[must_use]
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Builder: override initial rotation.
    #[must_use]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Builder: override initial linear velocity.
    #[must_use]
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Builder: compute mass + inertia from a uniform density.
    #[must_use]
    pub fn with_density(mut self, density: f32) -> Self {
        debug_assert!(density > 0.0 && density.is_finite(), "bad density: {density}");
        self.mass = self.shape.mass(density);
        self.inertia = self.shape.inertia(self.mass);
        self.update_mass_properties();
        self
    }

    /// Builder: set mass directly (re-derives inertia from the shape).
    #[must_use]
    pub fn with_mass(mut self, mass: f32) -> Self {
        debug_assert!(mass > 0.0 && mass.is_finite(), "bad mass: {mass}");
        self.mass = mass;
        self.inertia = self.shape.inertia(mass);
        self.update_mass_properties();
        self
    }

    /// Builder: set restitution (bounciness, `[0, 1]`).
    #[must_use]
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Builder: set friction coefficient.
    #[must_use]
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Body type accessor.
    #[inline]
    #[must_use]
    pub fn body_type(&self) -> BodyType {
        self.body_type
    }

    /// Mass accessor.
    #[inline]
    #[must_use]
    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// Inverse mass accessor (`0` for non-dynamic bodies).
    #[inline]
    #[must_use]
    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    /// Set body type; refreshes inverse mass/inertia.
    pub fn set_body_type(&mut self, body_type: BodyType) {
        self.body_type = body_type;
        self.update_mass_properties();
    }

    /// Set mass; refreshes inverse mass and shape-derived inertia.
    pub fn set_mass(&mut self, mass: f32) {
        debug_assert!(mass > 0.0 && mass.is_finite(), "bad mass: {mass}");
        self.mass = mass;
        self.inertia = self.shape.inertia(mass);
        self.update_mass_properties();
    }

    /// Apply a force at the center of mass.
    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == BodyType::Dynamic {
            self.force_accumulator += force;
        }
    }

    /// Apply a force at a world-space point (generates torque).
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        if self.body_type == BodyType::Dynamic {
            self.force_accumulator += force;
            let r = point - self.position;
            self.torque_accumulator += r.cross(force);
        }
    }

    /// Apply an instantaneous impulse at the center of mass.
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Apply an instantaneous impulse at a world-space point.
    pub fn apply_impulse_at_point(&mut self, impulse: Vec3, point: Vec3) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;
            let r = point - self.position;
            let dw = r.cross(impulse);
            self.angular_velocity += Vec3::new(
                dw.x * self.inv_inertia.x,
                dw.y * self.inv_inertia.y,
                dw.z * self.inv_inertia.z,
            );
        }
    }

    /// World-space AABB for broad-phase queries.
    pub fn world_aabb(&self) -> Aabb3 {
        self.shape.world_aabb(self.position)
    }

    /// Linear velocity at the world-space point `p`, including the angular
    /// contribution `ω × r` where `r = p − position`.
    #[inline]
    #[must_use]
    pub fn velocity_at_point(&self, p: Vec3) -> Vec3 {
        let r = p - self.position;
        self.velocity + self.angular_velocity.cross(r)
    }

    /// Inverse inertia tensor (diagonal). `0` per-axis for non-dynamic bodies
    /// or for axes locked by fixed rotation.
    #[inline]
    #[must_use]
    pub fn inv_inertia(&self) -> Vec3 {
        self.inv_inertia
    }

    /// Clear accumulated forces and torque. Called by the world each step.
    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec3::ZERO;
        self.torque_accumulator = Vec3::ZERO;
    }

    fn update_mass_properties(&mut self) {
        match self.body_type {
            BodyType::Static | BodyType::Kinematic => {
                self.inv_mass = 0.0;
                self.inv_inertia = Vec3::ZERO;
            },
            BodyType::Dynamic => {
                self.inv_mass = if self.mass > 0.0 { 1.0 / self.mass } else { 0.0 };
                self.inv_inertia = Vec3::new(
                    if self.inertia.x > 0.0 { 1.0 / self.inertia.x } else { 0.0 },
                    if self.inertia.y > 0.0 { 1.0 / self.inertia.y } else { 0.0 },
                    if self.inertia.z > 0.0 { 1.0 / self.inertia.z } else { 0.0 },
                );
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_shape(r: f32) -> CollisionShape {
        CollisionShape::Sphere(Sphere::new(Vec3::ZERO, r))
    }

    fn box_shape(w: f32, h: f32, d: f32) -> CollisionShape {
        CollisionShape::Aabb(Aabb3::from_center_size(Vec3::ZERO, Vec3::new(w, h, d)))
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn new_body_is_dynamic_unit_mass() {
        let b = RigidBody::new(0, sphere_shape(1.0));
        assert_eq!(b.body_type(), BodyType::Dynamic);
        assert_eq!(b.mass(), 1.0);
    }

    #[test]
    fn sphere_mass_matches_volume_times_density() {
        let s = sphere_shape(1.0);
        let m = s.mass(1.0);
        // 4/3 π r³ at r=1 = 4.189
        assert!(approx(m, (4.0 / 3.0) * std::f32::consts::PI));
    }

    #[test]
    fn box_mass_is_volume_times_density() {
        let b = box_shape(2.0, 3.0, 4.0);
        assert!(approx(b.mass(2.0), 2.0 * 24.0));
    }

    #[test]
    fn sphere_inertia_is_2_5_mass_r_squared() {
        let s = sphere_shape(2.0);
        // I = 2/5 m r² with m=10, r=2: 2/5 · 10 · 4 = 16
        let i = s.inertia(10.0);
        assert!(approx(i.x, 16.0));
        assert!(approx(i.y, 16.0));
        assert!(approx(i.z, 16.0));
    }

    #[test]
    fn box_inertia_uniaxial_formula() {
        let b = box_shape(2.0, 4.0, 6.0); // w=2, h=4, d=6
        // I_x = m/12 (h² + d²) = m/12 (16 + 36) = 52m/12
        let i = b.inertia(12.0);
        assert!(approx(i.x, 52.0));
        // I_y = m/12 (w² + d²) = m/12 (4 + 36) = 40
        assert!(approx(i.y, 40.0));
        // I_z = m/12 (w² + h²) = m/12 (4 + 16) = 20
        assert!(approx(i.z, 20.0));
    }

    #[test]
    fn static_body_zero_inverse_mass() {
        let b = RigidBody::new(0, sphere_shape(1.0)).with_type(BodyType::Static);
        assert_eq!(b.inv_mass(), 0.0);
    }

    #[test]
    fn apply_force_only_on_dynamic() {
        let mut b = RigidBody::new(0, sphere_shape(1.0));
        b.apply_force(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(b.force_accumulator, Vec3::new(10.0, 0.0, 0.0));

        let mut s = RigidBody::new(0, sphere_shape(1.0)).with_type(BodyType::Static);
        s.apply_force(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(s.force_accumulator, Vec3::ZERO);
    }

    #[test]
    fn apply_force_at_point_adds_torque() {
        let mut b = RigidBody::new(0, sphere_shape(1.0)).with_position(Vec3::ZERO);
        // Apply (0, 10, 0) at (1, 0, 0): torque = r × F = (1,0,0)×(0,10,0) = (0, 0, 10)
        b.apply_force_at_point(Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        assert!(approx(b.torque_accumulator.z, 10.0));
        assert!(approx(b.torque_accumulator.x, 0.0));
        assert!(approx(b.torque_accumulator.y, 0.0));
    }

    #[test]
    fn world_aabb_translates_with_position() {
        let b = RigidBody::new(0, sphere_shape(2.0)).with_position(Vec3::new(10.0, 0.0, 0.0));
        let a = b.world_aabb();
        assert_eq!(a.center(), Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(a.size(), Vec3::splat(4.0));
    }

    #[test]
    fn set_mass_keeps_inv_mass_in_sync() {
        let mut b = RigidBody::new(0, sphere_shape(1.0));
        b.set_mass(4.0);
        assert!(approx(b.mass(), 4.0));
        assert!(approx(b.inv_mass(), 0.25));
    }

    #[test]
    fn clear_forces_resets_accumulators() {
        let mut b = RigidBody::new(0, sphere_shape(1.0));
        b.apply_force(Vec3::new(10.0, 20.0, 30.0));
        b.torque_accumulator = Vec3::new(1.0, 2.0, 3.0);
        b.clear_forces();
        assert_eq!(b.force_accumulator, Vec3::ZERO);
        assert_eq!(b.torque_accumulator, Vec3::ZERO);
    }
}
