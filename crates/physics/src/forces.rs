// physics/src/forces.rs

//! Force generators for applying external forces to rigid bodies.
//!
//! This module provides various force generators that can be registered
//! with a physics world to apply forces like gravity, springs, and fields.

use std::collections::HashMap;

use gravita_math::Vec2;

use crate::body::{BodyType, RigidBody};

/// Trait implemented by anything that can apply forces to bodies each step.
pub trait ForceGenerator: Send + Sync {
    /// Apply forces to the given bodies for the current time step.
    fn apply(&mut self, bodies: &mut [RigidBody], dt: f32);
}

/// Apply `f` to every dynamic body in the slice. Shared helper for force
/// generators that only affect dynamic bodies (the common case).
#[inline]
fn apply_to_dynamic<F: FnMut(&mut RigidBody)>(bodies: &mut [RigidBody], mut f: F) {
    for body in bodies {
        if body.body_type == BodyType::Dynamic {
            f(body);
        }
    }
}

/// Registry for managing global and per-body force generators.
#[derive(Default)]
pub struct ForceRegistry {
    generators: Vec<Box<dyn ForceGenerator>>,
    body_forces: HashMap<usize, Vec<Box<dyn ForceGenerator>>>,
}

impl ForceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a force that is applied to all bodies every step.
    pub fn add_global_force(&mut self, generator: Box<dyn ForceGenerator>) {
        self.generators.push(generator);
    }

    /// Register a force that only affects a single body id.
    pub fn add_body_force(&mut self, body_id: usize, generator: Box<dyn ForceGenerator>) {
        self.body_forces.entry(body_id).or_default().push(generator);
    }

    /// Apply all registered forces for the current time step.
    ///
    /// This is typically called once per frame by the world.
    pub fn apply_all(&mut self, bodies: &mut [RigidBody], dt: f32) {
        for generator in &mut self.generators {
            generator.apply(bodies, dt);
        }

        for (body_id, generators) in &mut self.body_forces {
            if *body_id < bodies.len() {
                let slice = std::slice::from_mut(&mut bodies[*body_id]);
                for generator in generators {
                    generator.apply(slice, dt);
                }
            }
        }
    }

    /// Remove all registered forces.
    pub fn clear(&mut self) {
        self.generators.clear();
        self.body_forces.clear();
    }
}

// ============================================================================
// BASIC FORCES
// ============================================================================

/// Simple constant gravity force applied to all dynamic bodies.
pub struct Gravity {
    /// The gravitational acceleration vector (e.g., (0, -9.81) for Earth).
    pub acceleration: Vec2,
}

impl Gravity {
    /// Create a gravity generator with custom acceleration.
    pub fn new(acceleration: Vec2) -> Self {
        Self { acceleration }
    }

    /// Earth-like gravity (9.81 m/s² downward).
    pub fn earth() -> Self {
        Self::new(Vec2::new(0.0, -9.81))
    }

    /// Moon-like gravity (1.62 m/s² downward).
    pub fn moon() -> Self {
        Self::new(Vec2::new(0.0, -1.62))
    }

    /// Mars-like gravity (3.71 m/s² downward).
    pub fn mars() -> Self {
        Self::new(Vec2::new(0.0, -3.71))
    }
}

impl ForceGenerator for Gravity {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        apply_to_dynamic(bodies, |body| {
            let force = self.acceleration * body.mass * body.gravity_scale;
            body.apply_force(force);
        });
    }
}

/// Linear drag force (simple air resistance proportional to velocity).
pub struct LinearDrag {
    /// Drag coefficient (higher = more resistance).
    pub coefficient: f32,
}

impl LinearDrag {
    /// Create a linear drag generator with the given coefficient.
    pub fn new(coefficient: f32) -> Self {
        Self { coefficient }
    }
}

impl ForceGenerator for LinearDrag {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        apply_to_dynamic(bodies, |body| {
            body.apply_force(-body.velocity * self.coefficient);
        });
    }
}

/// Quadratic drag force (realistic air resistance proportional to velocity²).
pub struct QuadraticDrag {
    /// Fluid density (kg/m³).
    pub density: f32,
    /// Shape-dependent drag coefficient.
    pub drag_coefficient: f32,
    /// Cross-sectional area (m²).
    pub area: f32,
}

impl QuadraticDrag {
    /// Create a quadratic drag generator with custom parameters.
    pub fn new(density: f32, drag_coefficient: f32, area: f32) -> Self {
        Self {
            density,
            drag_coefficient,
            area,
        }
    }

    /// Create drag for air at sea level (density 1.225 kg/m³).
    pub fn air(drag_coefficient: f32, area: f32) -> Self {
        Self::new(1.225, drag_coefficient, area)
    }

    /// Create drag for water (density 1000 kg/m³).
    pub fn water(drag_coefficient: f32, area: f32) -> Self {
        Self::new(1000.0, drag_coefficient, area)
    }
}

impl ForceGenerator for QuadraticDrag {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        apply_to_dynamic(bodies, |body| {
            let speed_sq = body.velocity.length_squared();
            if speed_sq > 1e-8 {
                let speed = speed_sq.sqrt();
                let drag_magnitude =
                    0.5 * self.density * speed_sq * self.drag_coefficient * self.area;
                body.apply_force(-(body.velocity / speed) * drag_magnitude);
            }
        });
    }
}

// ============================================================================
// SPRING FORCES
// ============================================================================

/// Spring force between two bodies (Hooke's law with damping).
pub struct Spring {
    /// Index of the first body.
    pub body_a: usize,
    /// Index of the second body.
    pub body_b: usize,
    /// Natural length of the spring at rest.
    pub rest_length: f32,
    /// Spring stiffness constant (k in F = -kx).
    pub spring_constant: f32,
    /// Damping coefficient to reduce oscillation.
    pub damping: f32,
    /// Local-space attachment point on body A.
    pub attachment_a: Vec2,
    /// Local-space attachment point on body B.
    pub attachment_b: Vec2,
}

impl Spring {
    /// Create a spring between two bodies with default attachment at centers.
    pub fn new(body_a: usize, body_b: usize, spring_constant: f32, rest_length: f32) -> Self {
        Self {
            body_a,
            body_b,
            rest_length,
            spring_constant,
            damping: 0.1,
            attachment_a: Vec2::ZERO,
            attachment_b: Vec2::ZERO,
        }
    }

    /// Set the damping coefficient (builder pattern).
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Set local-space attachment points (builder pattern).
    pub fn with_attachments(mut self, attachment_a: Vec2, attachment_b: Vec2) -> Self {
        self.attachment_a = attachment_a;
        self.attachment_b = attachment_b;
        self
    }
}

impl ForceGenerator for Spring {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        if self.body_a >= bodies.len() || self.body_b >= bodies.len() {
            return;
        }

        let body_a = &bodies[self.body_a];
        let body_b = &bodies[self.body_b];

        // Calculate world space attachment points
        let world_a = body_a.position + self.attachment_a.rotate(body_a.rotation);
        let world_b = body_b.position + self.attachment_b.rotate(body_b.rotation);

        // Calculate spring vector
        let delta = world_b - world_a;
        let distance = delta.length();

        if distance < 0.0001 {
            return; // Avoid division by zero
        }

        // Calculate spring force (Hooke's law)
        let displacement = distance - self.rest_length;
        let spring_direction = delta / distance;
        let spring_magnitude = self.spring_constant * displacement;

        // Calculate damping force
        let velocity_a = body_a.get_velocity_at_point(world_a);
        let velocity_b = body_b.get_velocity_at_point(world_b);
        let relative_velocity = velocity_b - velocity_a;
        let damping_magnitude = self.damping * relative_velocity.dot(spring_direction);

        // Total force
        let force = spring_direction * (spring_magnitude + damping_magnitude);

        // Apply forces (mutable borrow workaround)
        let (body_a_mut, body_b_mut) = if self.body_a < self.body_b {
            let (left, right) = bodies.split_at_mut(self.body_b);
            (&mut left[self.body_a], &mut right[0])
        } else {
            let (left, right) = bodies.split_at_mut(self.body_a);
            (&mut right[0], &mut left[self.body_b])
        };

        body_a_mut.apply_force_at_point(force, world_a);
        body_b_mut.apply_force_at_point(-force, world_b);
    }
}

/// Anchored spring - connects a body to a fixed point in world space.
pub struct AnchoredSpring {
    /// Index of the body to attach.
    pub body: usize,
    /// Fixed world-space anchor point.
    pub anchor: Vec2,
    /// Natural length of the spring at rest.
    pub rest_length: f32,
    /// Spring stiffness constant.
    pub spring_constant: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Local-space attachment point on the body.
    pub attachment: Vec2,
}

impl AnchoredSpring {
    /// Create an anchored spring from a body to a fixed point.
    pub fn new(body: usize, anchor: Vec2, spring_constant: f32, rest_length: f32) -> Self {
        Self {
            body,
            anchor,
            rest_length,
            spring_constant,
            damping: 0.1,
            attachment: Vec2::ZERO,
        }
    }
}

impl ForceGenerator for AnchoredSpring {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        if let Some(body) = bodies.get_mut(self.body) {
            if body.body_type != BodyType::Dynamic {
                return;
            }

            // Calculate world space attachment point
            let world_attachment = body.position + self.attachment.rotate(body.rotation);

            // Calculate spring vector
            let delta = self.anchor - world_attachment;
            let distance = delta.length();

            if distance < 0.0001 {
                return;
            }

            // Spring force
            let displacement = distance - self.rest_length;
            let spring_direction = delta / distance;
            let spring_magnitude = self.spring_constant * displacement;

            // Damping force
            let velocity = body.get_velocity_at_point(world_attachment);
            let damping_magnitude = self.damping * velocity.dot(spring_direction);

            // Total force
            let force = spring_direction * (spring_magnitude - damping_magnitude);
            body.apply_force_at_point(force, world_attachment);
        }
    }
}

/// Bungee cord - only applies force when stretched beyond rest length.
pub struct BungeeSpring {
    /// The underlying spring (only applies when stretched).
    pub spring: Spring,
}

impl BungeeSpring {
    /// Create a bungee spring between two bodies.
    pub fn new(body_a: usize, body_b: usize, spring_constant: f32, rest_length: f32) -> Self {
        Self {
            spring: Spring::new(body_a, body_b, spring_constant, rest_length),
        }
    }
}

impl ForceGenerator for BungeeSpring {
    fn apply(&mut self, bodies: &mut [RigidBody], dt: f32) {
        // Check current length
        if self.spring.body_a >= bodies.len() || self.spring.body_b >= bodies.len() {
            return;
        }

        let body_a = &bodies[self.spring.body_a];
        let body_b = &bodies[self.spring.body_b];

        let world_a = body_a.position + self.spring.attachment_a.rotate(body_a.rotation);
        let world_b = body_b.position + self.spring.attachment_b.rotate(body_b.rotation);

        let distance = world_a.distance(world_b);

        // Only apply force if stretched
        if distance > self.spring.rest_length {
            self.spring.apply(bodies, dt);
        }
    }
}

// ============================================================================
// FIELD FORCES
// ============================================================================

/// Radial force field (explosion or implosion).
pub struct RadialForce {
    /// Center of the force field in world space.
    pub center: Vec2,
    /// Maximum radius of effect.
    pub radius: f32,
    /// Base strength (positive = outward, negative = inward).
    pub strength: f32,
    /// How force diminishes with distance.
    pub falloff: FalloffType,
    /// If true, applies as impulse instead of continuous force.
    pub is_impulse: bool,
}

/// How a force diminishes with distance.
#[derive(Debug, Clone, Copy)]
pub enum FalloffType {
    /// Force is constant regardless of distance.
    Constant,
    /// Force decreases linearly with distance.
    Linear,
    /// Force decreases with the square of distance.
    Quadratic,
}

impl RadialForce {
    /// Create an explosion force (outward impulse with quadratic falloff).
    pub fn explosion(center: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            center,
            radius,
            strength,
            falloff: FalloffType::Quadratic,
            is_impulse: true,
        }
    }

    /// Create a black hole force (inward continuous force with quadratic falloff).
    pub fn black_hole(center: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            center,
            radius,
            strength: -strength,
            falloff: FalloffType::Quadratic,
            is_impulse: false,
        }
    }
}

impl ForceGenerator for RadialForce {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        let radius_sq = self.radius * self.radius;
        apply_to_dynamic(bodies, |body| {
            let delta = body.position - self.center;
            let dist_sq = delta.length_squared();
            // Squared-space early out lets ~95% of out-of-range bodies skip the sqrt.
            if dist_sq > radius_sq || dist_sq < 1e-8 {
                return;
            }

            let distance = dist_sq.sqrt();
            let direction = delta / distance;

            let magnitude = match self.falloff {
                FalloffType::Constant => self.strength,
                FalloffType::Linear => self.strength * (1.0 - distance / self.radius),
                FalloffType::Quadratic => {
                    let ratio = 1.0 - distance / self.radius;
                    self.strength * ratio * ratio
                },
            };

            let force = direction * magnitude;

            if self.is_impulse {
                body.apply_impulse(force);
            } else {
                body.apply_force(force);
            }
        });
    }
}

/// Directional force field (wind, current, conveyor belt).
pub struct DirectionalForce {
    /// Direction of the force (normalized).
    pub direction: Vec2,
    /// Magnitude of the force.
    pub strength: f32,
    /// Optional bounds - force only applies within this area.
    pub bounds: Option<gravita_math::Aabb>,
    /// Random variation in force direction (0 = none, 1 = ±100%).
    pub turbulence: f32,
}

impl DirectionalForce {
    /// Create a wind force with optional turbulence.
    pub fn wind(direction: Vec2, strength: f32) -> Self {
        Self {
            direction: direction.normalize(),
            strength,
            bounds: None,
            turbulence: 0.1,
        }
    }

    /// Set the bounds of the force field.
    pub fn with_bounds(mut self, bounds: gravita_math::Aabb) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

impl ForceGenerator for DirectionalForce {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        apply_to_dynamic(bodies, |body| {
            if let Some(bounds) = self.bounds
                && !bounds.contains_point(body.position)
            {
                return;
            }

            let mut force = self.direction * self.strength;

            if self.turbulence > 0.0 {
                let noise_x = (body.position.x * 0.1).sin() * self.turbulence;
                let noise_y = (body.position.y * 0.1).cos() * self.turbulence;
                force += Vec2::new(noise_x, noise_y) * self.strength;
            }

            body.apply_force(force);
        });
    }
}

/// Buoyancy force for fluid simulation
pub struct Buoyancy {
    /// The level of the water in world space.
    pub water_level: f32,
    /// The density of the liquid.
    pub liquid_density: f32,
    /// The velocity of the flow of the liquid.
    pub flow_velocity: Vec2,
    /// The drag coefficient of the body in the liquid.
    pub drag_coefficient: f32,
}

impl Buoyancy {
    /// Create a buoyancy generator for water.
    pub fn water(water_level: f32) -> Self {
        Self {
            water_level,
            liquid_density: 1000.0, // Water density
            flow_velocity: Vec2::ZERO,
            drag_coefficient: 0.47, // Sphere drag coefficient
        }
    }

    /// Set the velocity of the flow of the liquid.
    pub fn with_flow(mut self, velocity: Vec2) -> Self {
        self.flow_velocity = velocity;
        self
    }
}

impl ForceGenerator for Buoyancy {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        apply_to_dynamic(bodies, |body| {
            let aabb = body.get_world_aabb();
            if aabb.min.y > self.water_level {
                return;
            }

            let submerged_height = (self.water_level - aabb.min.y).min(aabb.size().y);
            let submerged_ratio = submerged_height / aabb.size().y;
            let submerged_area = aabb.size().x * submerged_height;

            let buoyancy = Vec2::new(0.0, self.liquid_density * submerged_area * 9.81);
            body.apply_force(buoyancy);

            let relative_velocity = body.velocity - self.flow_velocity;
            let speed_sq = relative_velocity.length_squared();
            if speed_sq > 1e-8 {
                let speed = speed_sq.sqrt();
                let drag_magnitude = 0.5
                    * self.liquid_density
                    * speed_sq
                    * self.drag_coefficient
                    * submerged_area
                    * submerged_ratio;
                let drag = -(relative_velocity / speed) * drag_magnitude;
                body.apply_force(drag);
            }
        });
    }
}

/// Magnetic/Electric field force
pub struct FieldForce {
    /// The sources of the force field.
    pub sources: Vec<(Vec2, f32)>, // (position, strength)
    /// The bodies that are affected by the force field.
    pub affected_bodies: Vec<usize>,
    /// The type of force field.
    pub force_type: FieldType,
}

/// The type of force field.
#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    /// Like gravity or magnetic attraction.
    Attractive,
    /// Like same-charge repulsion.
    Repulsive,
    /// Alternating based on distance.
    Dipole,
}

impl FieldForce {
    /// Create a new field force generator.
    pub fn new(force_type: FieldType) -> Self {
        Self {
            sources: Vec::new(),
            affected_bodies: Vec::new(),
            force_type,
        }
    }

    /// Add a source to the force field.
    pub fn add_source(&mut self, position: Vec2, strength: f32) {
        self.sources.push((position, strength));
    }

    /// Add a body to the force field.
    pub fn affects_body(&mut self, body_id: usize) {
        self.affected_bodies.push(body_id);
    }

    /// Affect all bodies in the force field.
    pub fn affects_all(&mut self) {
        self.affected_bodies.clear(); // Empty means all
    }
}

impl ForceGenerator for FieldForce {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        let check_all = self.affected_bodies.is_empty();

        for (idx, body) in bodies.iter_mut().enumerate() {
            if body.body_type != BodyType::Dynamic {
                continue;
            }
            if !check_all && !self.affected_bodies.contains(&idx) {
                continue;
            }

            let mut total_force = Vec2::ZERO;
            for (source_pos, strength) in &self.sources {
                let delta = *source_pos - body.position;
                let dist_sq = delta.length_squared();
                if dist_sq < 1e-6 {
                    continue;
                }

                let distance = dist_sq.sqrt();
                let direction = delta / distance;

                let magnitude = match self.force_type {
                    FieldType::Attractive => strength / dist_sq,
                    FieldType::Repulsive => -strength / dist_sq,
                    FieldType::Dipole => strength * (distance * 0.5).cos() / dist_sq,
                };

                total_force += direction * magnitude * body.mass;
            }

            body.apply_force(total_force);
        }
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::Circle;

    use super::*;
    use crate::body::CollisionShape;

    fn dynamic_body(id: usize, position: Vec2, mass: f32) -> RigidBody {
        let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 1.0));
        let mut b = RigidBody::new(id, shape).with_position(position);
        b.mass = mass;
        b.inv_mass = 1.0 / mass;
        b
    }

    fn static_body(id: usize, position: Vec2) -> RigidBody {
        let shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 1.0));
        RigidBody::new(id, shape)
            .with_position(position)
            .with_type(BodyType::Static)
    }

    // =========================================================================
    // apply_to_dynamic helper
    // =========================================================================

    #[test]
    fn apply_to_dynamic_skips_static_and_kinematic() {
        let mut bodies = vec![
            dynamic_body(0, Vec2::ZERO, 1.0),
            static_body(1, Vec2::ZERO),
            {
                let mut k = dynamic_body(2, Vec2::ZERO, 1.0);
                k.body_type = BodyType::Kinematic;
                k
            },
            dynamic_body(3, Vec2::ZERO, 1.0),
        ];
        let mut called = 0;
        apply_to_dynamic(&mut bodies, |_b| called += 1);
        assert_eq!(called, 2);
    }

    // =========================================================================
    // Gravity
    // =========================================================================

    #[test]
    fn gravity_applies_force_proportional_to_mass() {
        let mut g = Gravity::new(Vec2::new(0.0, -10.0));
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 5.0)];
        g.apply(&mut bodies, 0.016);
        // F = m*g = 5 * (0, -10) = (0, -50)
        assert_eq!(bodies[0].force_accumulator, Vec2::new(0.0, -50.0));
    }

    #[test]
    fn gravity_respects_gravity_scale() {
        let mut g = Gravity::new(Vec2::new(0.0, -10.0));
        let mut body = dynamic_body(0, Vec2::ZERO, 1.0);
        body.gravity_scale = 2.0;
        let mut bodies = vec![body];
        g.apply(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::new(0.0, -20.0));
    }

    #[test]
    fn gravity_skips_static_bodies() {
        let mut g = Gravity::new(Vec2::new(0.0, -10.0));
        let mut bodies = vec![static_body(0, Vec2::ZERO)];
        g.apply(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
    }

    #[test]
    fn gravity_presets_have_expected_acceleration() {
        assert_eq!(Gravity::earth().acceleration.y, -9.81);
        assert_eq!(Gravity::moon().acceleration.y, -1.62);
        assert_eq!(Gravity::mars().acceleration.y, -3.71);
    }

    // =========================================================================
    // LinearDrag
    // =========================================================================

    #[test]
    fn linear_drag_opposes_velocity() {
        let mut drag = LinearDrag::new(0.5);
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        bodies[0].velocity = Vec2::new(10.0, -4.0);
        drag.apply(&mut bodies, 0.016);
        // F = -coefficient * v = -0.5 * (10, -4) = (-5, 2)
        assert_eq!(bodies[0].force_accumulator, Vec2::new(-5.0, 2.0));
    }

    #[test]
    fn linear_drag_zero_velocity_no_force() {
        let mut drag = LinearDrag::new(1.0);
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        drag.apply(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
    }

    // =========================================================================
    // QuadraticDrag (uses length_squared early-out)
    // =========================================================================

    #[test]
    fn quadratic_drag_opposes_velocity() {
        let mut drag = QuadraticDrag::new(1.0, 1.0, 1.0);
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        bodies[0].velocity = Vec2::new(10.0, 0.0);
        drag.apply(&mut bodies, 0.016);
        // F_magnitude = 0.5 * density * speed² * Cd * area = 0.5 * 1 * 100 * 1 * 1 = 50
        // F direction is -velocity / speed = (-1, 0)
        assert_eq!(bodies[0].force_accumulator, Vec2::new(-50.0, 0.0));
    }

    #[test]
    fn quadratic_drag_skips_resting_body_squared_threshold() {
        // Verify the length_squared early-out: speed below 1e-4 (=> speed_sq < 1e-8) skips.
        let mut drag = QuadraticDrag::new(1.0, 1.0, 1.0);
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        bodies[0].velocity = Vec2::new(1e-5, 0.0);
        drag.apply(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
    }

    // =========================================================================
    // RadialForce (squared-space early-out)
    // =========================================================================

    #[test]
    fn radial_force_outside_radius_skipped() {
        let mut explosion = RadialForce::explosion(Vec2::ZERO, 5.0, 100.0);
        let mut bodies = vec![dynamic_body(0, Vec2::new(10.0, 0.0), 1.0)];
        explosion.apply(&mut bodies, 0.016);
        // Body is 10 away, radius 5 -> outside, skipped
        assert_eq!(bodies[0].velocity, Vec2::ZERO);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
    }

    #[test]
    fn radial_force_inside_radius_pushes_outward() {
        let mut explosion = RadialForce::explosion(Vec2::ZERO, 10.0, 100.0);
        let mut bodies = vec![dynamic_body(0, Vec2::new(5.0, 0.0), 1.0)];
        explosion.apply(&mut bodies, 0.016);
        // Body inside, impulse should push outward (positive x direction)
        assert!(bodies[0].velocity.x > 0.0);
    }

    #[test]
    fn black_hole_pulls_inward() {
        let mut bh = RadialForce::black_hole(Vec2::ZERO, 100.0, 50.0);
        let mut bodies = vec![dynamic_body(0, Vec2::new(10.0, 0.0), 1.0)];
        bh.apply(&mut bodies, 0.016);
        // Continuous force pulling toward origin: negative x force.
        assert!(bodies[0].force_accumulator.x < 0.0);
    }

    // =========================================================================
    // ForceRegistry (verifies clone-free body slicing)
    // =========================================================================

    #[test]
    fn force_registry_applies_global_force() {
        let mut reg = ForceRegistry::new();
        reg.add_global_force(Box::new(Gravity::new(Vec2::new(0.0, -10.0))));
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        reg.apply_all(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::new(0.0, -10.0));
    }

    #[test]
    fn force_registry_body_specific_force_mutates_target_in_place() {
        // Confirms the slice::from_mut refactor: changes survive without a clone+copy roundtrip.
        let mut reg = ForceRegistry::new();
        reg.add_body_force(0, Box::new(LinearDrag::new(0.5)));
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        bodies[0].velocity = Vec2::new(10.0, 0.0);
        reg.apply_all(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::new(-5.0, 0.0));
    }

    #[test]
    fn force_registry_body_specific_does_not_touch_other_bodies() {
        let mut reg = ForceRegistry::new();
        reg.add_body_force(1, Box::new(Gravity::new(Vec2::new(0.0, -10.0))));
        let mut bodies = vec![
            dynamic_body(0, Vec2::ZERO, 1.0),
            dynamic_body(1, Vec2::ZERO, 1.0),
        ];
        reg.apply_all(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
        assert_eq!(bodies[1].force_accumulator, Vec2::new(0.0, -10.0));
    }

    #[test]
    fn force_registry_clear_drops_all_generators() {
        let mut reg = ForceRegistry::new();
        reg.add_global_force(Box::new(Gravity::new(Vec2::new(0.0, -10.0))));
        reg.add_body_force(0, Box::new(LinearDrag::new(0.5)));
        reg.clear();
        let mut bodies = vec![dynamic_body(0, Vec2::ZERO, 1.0)];
        bodies[0].velocity = Vec2::new(10.0, 0.0);
        reg.apply_all(&mut bodies, 0.016);
        assert_eq!(bodies[0].force_accumulator, Vec2::ZERO);
    }
}
