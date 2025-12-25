// physics/src/body.rs
use gravita_math::{AABB, Circle, Vec2};

/// High-level classification of how a body participates in the simulation.
///
/// - `Static`: never moved by the physics solver (e.g. ground, walls).
/// - `Kinematic`: moved by user code, but not by forces (e.g. moving platforms).
/// - `Dynamic`: fully simulated by forces, collisions and constraints.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BodyType {
    /// Never moved by the physics solver (e.g., ground, walls).
    Static,
    /// Moved by user code, but not by forces (e.g., moving platforms).
    Kinematic,
    /// Fully simulated by forces, collisions, and constraints.
    Dynamic,
}

/// Geometric shape used for collision detection.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CollisionShape {
    /// Circular shape, usually cheap to test and simulate.
    Circle(Circle),
    /// Axis-aligned box shape.
    AABB(AABB),
}

impl CollisionShape {
    /// Check if this shape has valid dimensions.
    ///
    /// Returns `false` for:
    /// - Circles with zero or negative radius
    /// - AABBs where min >= max on either axis
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Circle(c) => c.radius > 0.0 && c.radius.is_finite(),
            Self::AABB(aabb) => {
                aabb.min.x < aabb.max.x
                    && aabb.min.y < aabb.max.y
                    && aabb.min.x.is_finite()
                    && aabb.min.y.is_finite()
                    && aabb.max.x.is_finite()
                    && aabb.max.y.is_finite()
            },
        }
    }

    /// Compute an axis-aligned bounding box that encloses this shape.
    pub fn get_aabb(&self) -> AABB {
        match self {
            Self::Circle(c) => c.to_aabb(),
            Self::AABB(aabb) => *aabb,
        }
    }

    /// Compute mass from a uniform `density` and the shape's area.
    pub fn get_mass(&self, density: f32) -> f32 {
        match self {
            Self::Circle(c) => {
                // Mass = density * area
                density * std::f32::consts::PI * c.radius * c.radius
            },
            Self::AABB(aabb) => {
                let size = aabb.size();
                density * size.x * size.y
            },
        }
    }

    /// Compute the scalar moment of inertia for the given `mass`.
    pub fn get_inertia(&self, mass: f32) -> f32 {
        match self {
            Self::Circle(c) => {
                // I = 0.5 * m * r²
                0.5 * mass * c.radius * c.radius
            },
            Self::AABB(aabb) => {
                // I = m * (w² + h²) / 12
                let size = aabb.size();
                mass * size.x.mul_add(size.x, size.y * size.y) / 12.0
            },
        }
    }
}

/// Rigid body with position, velocity and collision shape.
///
/// This is the main simulation object used by [`crate::world::PhysicsWorld`].
#[derive(Debug, Clone)]
pub struct RigidBody {
    // Identity
    /// Unique identifier within the physics world.
    pub id: usize,
    /// How this body participates in the simulation.
    pub body_type: BodyType,

    // Transform
    /// World-space position of the body's center of mass.
    pub position: Vec2,
    /// Rotation angle in radians.
    pub rotation: f32,

    // Linear dynamics
    /// Current linear velocity (units per second).
    pub velocity: Vec2,
    /// Current linear acceleration (computed from forces).
    pub acceleration: Vec2,
    /// Accumulated forces to be applied this step.
    pub force_accumulator: Vec2,

    // Angular dynamics
    /// Current angular velocity (radians per second).
    pub angular_velocity: f32,
    /// Current angular acceleration (computed from torque).
    pub angular_acceleration: f32,
    /// Accumulated torque to be applied this step.
    pub torque_accumulator: f32,

    // Mass properties
    /// Mass of the body in arbitrary units.
    pub mass: f32,
    /// Inverse mass (1/mass), zero for infinite mass.
    pub inv_mass: f32,
    /// Moment of inertia (resistance to rotation).
    pub inertia: f32,
    /// Inverse inertia (1/inertia), zero for fixed rotation.
    pub inv_inertia: f32,

    // Material properties
    /// Coefficient of restitution (bounciness), range [0, 1].
    pub restitution: f32,
    /// Friction coefficient, range [0, 1].
    pub friction: f32,
    /// Linear damping (air resistance), range [0, 1].
    pub linear_damping: f32,
    /// Angular damping (rotational resistance), range [0, 1].
    pub angular_damping: f32,
    /// Multiplier for gravity effect on this body.
    pub gravity_scale: f32,

    // Collision
    /// Geometric shape used for collision detection.
    pub shape: CollisionShape,

    // Constraints
    /// If true, detects collisions but doesn't respond physically.
    pub is_sensor: bool,
    /// If true, prevents rotation from physics forces.
    pub fixed_rotation: bool,
}

impl RigidBody {
    /// Create a new dynamic body with default material properties.
    ///
    /// The `id` is typically set by [`crate::world::PhysicsWorld::add_body`]
    /// and is only meaningful inside a given world.
    ///
    /// # Panics (debug builds only)
    ///
    /// Debug assertions will panic if:
    /// - Circle radius is zero or negative
    /// - AABB has min >= max on either axis
    pub fn new(id: usize, shape: CollisionShape) -> Self {
        debug_assert!(shape.is_valid(), "Invalid collision shape: {shape:?}");

        let mass = 1.0;
        let inertia = shape.get_inertia(mass);

        Self {
            id,
            body_type: BodyType::Dynamic,
            position: Vec2::ZERO,
            rotation: 0.0,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            force_accumulator: Vec2::ZERO,
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            torque_accumulator: 0.0,
            mass,
            inv_mass: 1.0 / mass,
            inertia,
            inv_inertia: 1.0 / inertia,
            restitution: 0.5,
            friction: 0.3,
            linear_damping: 0.01,
            angular_damping: 0.01,
            gravity_scale: 1.0,
            shape,
            is_sensor: false,
            fixed_rotation: false,
        }
    }

    /// Builder-style setter for [`BodyType`].
    pub fn with_type(mut self, body_type: BodyType) -> Self {
        self.body_type = body_type;
        self.update_mass_properties();
        self
    }

    /// Builder-style setter for mass based on `density`.
    ///
    /// This automatically recomputes inertia for the current shape.
    ///
    /// # Panics (debug builds only)
    ///
    /// Debug assertion fails if density is not positive and finite.
    pub fn with_density(mut self, density: f32) -> Self {
        debug_assert!(
            density > 0.0 && density.is_finite(),
            "Density must be positive and finite, got {density}"
        );
        self.mass = self.shape.get_mass(density);
        self.inertia = self.shape.get_inertia(self.mass);
        self.update_mass_properties();
        self
    }

    /// Builder-style setter for initial world position.
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Builder-style setter for initial linear velocity.
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    /// Recalculate inverse mass and inertia based on current values.
    fn update_mass_properties(&mut self) {
        match self.body_type {
            BodyType::Static | BodyType::Kinematic => {
                self.inv_mass = 0.0;
                self.inv_inertia = 0.0;
            },
            BodyType::Dynamic => {
                self.inv_mass = if self.mass > 0.0 {
                    1.0 / self.mass
                } else {
                    0.0
                };
                self.inv_inertia = if self.inertia > 0.0 && !self.fixed_rotation {
                    1.0 / self.inertia
                } else {
                    0.0
                };
            },
        }
    }

    /// Apply a continuous force at the center of mass (affects acceleration).
    pub fn apply_force(&mut self, force: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.force_accumulator += force;
        }
    }

    /// Apply a continuous force at an arbitrary world-space point.
    ///
    /// This affects both linear and angular acceleration.
    pub fn apply_force_at_point(&mut self, force: Vec2, point: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.force_accumulator += force;

            // Torque = r × F (in 2D, this becomes a scalar)
            let r = point - self.position;
            self.torque_accumulator += r.cross(force);
        }
    }

    /// Apply an instantaneous impulse at the center of mass.
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Apply an instantaneous impulse at an arbitrary world-space point.
    pub fn apply_impulse_at_point(&mut self, impulse: Vec2, point: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;

            let r = point - self.position;
            self.angular_velocity += r.cross(impulse) * self.inv_inertia;
        }
    }

    /// Get this body's shape as a world-space axis-aligned bounding box.
    ///
    /// This is used by the broad-phase and for simple overlap queries.
    pub fn get_world_aabb(&self) -> AABB {
        match &self.shape {
            CollisionShape::Circle(c) => Circle::new(self.position + c.center, c.radius).to_aabb(),
            CollisionShape::AABB(aabb) => aabb.translate(self.position),
        }
    }

    /// Compute the linear velocity at an arbitrary world-space point.
    ///
    /// This includes both linear and angular contributions.
    pub fn get_velocity_at_point(&self, point: Vec2) -> Vec2 {
        let r = point - self.position;
        let tangent = r.perpendicular();
        self.velocity + tangent * self.angular_velocity
    }

    /// Clear all accumulated forces and torque from the previous step.
    ///
    /// This is typically called once per simulation step by the world.
    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec2::ZERO;
        self.torque_accumulator = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    fn circle_shape(radius: f32) -> CollisionShape {
        CollisionShape::Circle(Circle::new(Vec2::ZERO, radius))
    }

    fn box_shape(width: f32, height: f32) -> CollisionShape {
        CollisionShape::AABB(AABB::from_center_size(Vec2::ZERO, Vec2::new(width, height)))
    }

    // =========================================================================
    // CollisionShape
    // =========================================================================

    #[test]
    fn circle_shape_aabb_conversion() {
        let shape = circle_shape(10.0);
        let aabb = shape.get_aabb();
        assert_eq!(aabb.min, Vec2::new(-10.0, -10.0));
        assert_eq!(aabb.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn box_shape_aabb_is_self() {
        let shape = box_shape(20.0, 30.0);
        let aabb = shape.get_aabb();
        assert_eq!(aabb.size(), Vec2::new(20.0, 30.0));
    }

    #[test]
    fn circle_mass_from_density() {
        let shape = circle_shape(10.0);
        let mass = shape.get_mass(1.0);
        // Area = π * r² = π * 100 ≈ 314.159
        assert!(approx_eq(mass, PI * 100.0));
    }

    #[test]
    fn box_mass_from_density() {
        let shape = box_shape(10.0, 20.0);
        let mass = shape.get_mass(1.0);
        // Area = 10 * 20 = 200
        assert_eq!(mass, 200.0);
    }

    #[test]
    fn circle_inertia_calculation() {
        let shape = circle_shape(10.0);
        let mass = 2.0;
        let inertia = shape.get_inertia(mass);
        // I = 0.5 * m * r² = 0.5 * 2 * 100 = 100
        assert_eq!(inertia, 100.0);
    }

    #[test]
    fn box_inertia_calculation() {
        let shape = box_shape(6.0, 8.0);
        let mass = 12.0;
        let inertia = shape.get_inertia(mass);
        // I = m * (w² + h²) / 12 = 12 * (36 + 64) / 12 = 100
        assert_eq!(inertia, 100.0);
    }

    // =========================================================================
    // RigidBody Construction
    // =========================================================================

    #[test]
    fn new_body_is_dynamic() {
        let body = RigidBody::new(0, circle_shape(10.0));
        assert_eq!(body.body_type, BodyType::Dynamic);
    }

    #[test]
    fn new_body_at_origin() {
        let body = RigidBody::new(0, circle_shape(10.0));
        assert_eq!(body.position, Vec2::ZERO);
    }

    #[test]
    fn new_body_has_unit_mass() {
        let body = RigidBody::new(0, circle_shape(10.0));
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.inv_mass, 1.0);
    }

    #[test]
    fn new_body_has_zero_velocity() {
        let body = RigidBody::new(0, circle_shape(10.0));
        assert_eq!(body.velocity, Vec2::ZERO);
        assert_eq!(body.angular_velocity, 0.0);
    }

    // =========================================================================
    // Builder Pattern
    // =========================================================================

    #[test]
    fn with_position_sets_position() {
        let body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::new(100.0, 200.0));
        assert_eq!(body.position, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn with_velocity_sets_velocity() {
        let body = RigidBody::new(0, circle_shape(10.0)).with_velocity(Vec2::new(50.0, 0.0));
        assert_eq!(body.velocity, Vec2::new(50.0, 0.0));
    }

    #[test]
    fn with_type_static_zeroes_inverse_mass() {
        let body = RigidBody::new(0, circle_shape(10.0)).with_type(BodyType::Static);
        assert_eq!(body.inv_mass, 0.0);
        assert_eq!(body.inv_inertia, 0.0);
    }

    #[test]
    fn with_type_kinematic_zeroes_inverse_mass() {
        let body = RigidBody::new(0, circle_shape(10.0)).with_type(BodyType::Kinematic);
        assert_eq!(body.inv_mass, 0.0);
        assert_eq!(body.inv_inertia, 0.0);
    }

    #[test]
    fn with_density_updates_mass_and_inertia() {
        let body = RigidBody::new(0, box_shape(10.0, 10.0)).with_density(2.0);
        // Area = 100, density = 2, mass = 200
        assert_eq!(body.mass, 200.0);
        assert!(body.inv_mass > 0.0);
    }

    // =========================================================================
    // Force Application
    // =========================================================================

    #[test]
    fn apply_force_accumulates() {
        let mut body = RigidBody::new(0, circle_shape(10.0));
        body.apply_force(Vec2::new(100.0, 0.0));
        body.apply_force(Vec2::new(0.0, 50.0));
        assert_eq!(body.force_accumulator, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn apply_force_ignored_for_static() {
        let mut body = RigidBody::new(0, circle_shape(10.0)).with_type(BodyType::Static);
        body.apply_force(Vec2::new(100.0, 0.0));
        assert_eq!(body.force_accumulator, Vec2::ZERO);
    }

    #[test]
    fn apply_force_at_point_adds_torque() {
        let mut body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::ZERO);
        // Force applied at (10, 0), pointing up
        body.apply_force_at_point(Vec2::new(0.0, 100.0), Vec2::new(10.0, 0.0));
        // Torque = r × F = (10, 0) × (0, 100) = 1000
        assert_eq!(body.torque_accumulator, 1000.0);
    }

    #[test]
    fn clear_forces_resets_accumulators() {
        let mut body = RigidBody::new(0, circle_shape(10.0));
        body.apply_force(Vec2::new(100.0, 50.0));
        body.torque_accumulator = 500.0;
        body.clear_forces();
        assert_eq!(body.force_accumulator, Vec2::ZERO);
        assert_eq!(body.torque_accumulator, 0.0);
    }

    // =========================================================================
    // Impulse Application
    // =========================================================================

    #[test]
    fn apply_impulse_changes_velocity() {
        let mut body = RigidBody::new(0, circle_shape(10.0));
        body.apply_impulse(Vec2::new(10.0, 0.0));
        // velocity += impulse * inv_mass = (10, 0) * 1 = (10, 0)
        assert_eq!(body.velocity, Vec2::new(10.0, 0.0));
    }

    #[test]
    fn apply_impulse_at_point_adds_angular_velocity() {
        let mut body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::ZERO);
        // Impulse applied at (10, 0), pointing up
        body.apply_impulse_at_point(Vec2::new(0.0, 1.0), Vec2::new(10.0, 0.0));
        // Angular velocity += r × impulse * inv_inertia
        assert!(body.angular_velocity > 0.0);
    }

    #[test]
    fn apply_impulse_ignored_for_static() {
        let mut body = RigidBody::new(0, circle_shape(10.0)).with_type(BodyType::Static);
        body.apply_impulse(Vec2::new(100.0, 0.0));
        assert_eq!(body.velocity, Vec2::ZERO);
    }

    // =========================================================================
    // World AABB
    // =========================================================================

    #[test]
    fn world_aabb_translated_by_position() {
        let body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::new(100.0, 100.0));
        let aabb = body.get_world_aabb();
        assert_eq!(aabb.center(), Vec2::new(100.0, 100.0));
    }

    #[test]
    fn world_aabb_for_box_shape() {
        let body = RigidBody::new(0, box_shape(20.0, 10.0)).with_position(Vec2::new(50.0, 50.0));
        let aabb = body.get_world_aabb();
        assert_eq!(aabb.min, Vec2::new(40.0, 45.0));
        assert_eq!(aabb.max, Vec2::new(60.0, 55.0));
    }

    // =========================================================================
    // Velocity at Point
    // =========================================================================

    #[test]
    fn velocity_at_center_equals_linear_velocity() {
        let body = RigidBody::new(0, circle_shape(10.0))
            .with_position(Vec2::ZERO)
            .with_velocity(Vec2::new(10.0, 5.0));
        let v = body.get_velocity_at_point(Vec2::ZERO);
        assert_eq!(v, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn velocity_at_point_includes_angular_component() {
        let mut body = RigidBody::new(0, circle_shape(10.0)).with_position(Vec2::ZERO);
        body.angular_velocity = 1.0; // 1 rad/s
        // At point (10, 0), tangent is (0, 10), angular contribution = (0, 10) * 1 = (0, 10)
        let v = body.get_velocity_at_point(Vec2::new(10.0, 0.0));
        assert!(vec_approx_eq(v, Vec2::new(0.0, 10.0)));
    }
}
