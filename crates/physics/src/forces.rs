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
        // Apply global forces
        for generator in &mut self.generators {
            generator.apply(bodies, dt);
        }

        // Apply body-specific forces
        for (body_id, generators) in &mut self.body_forces {
            if let Some(body) = bodies.get_mut(*body_id) {
                // Create a temporary single-body slice
                let mut temp_bodies = vec![body.clone()];
                for generator in generators {
                    generator.apply(&mut temp_bodies, dt);
                }
                // Copy back the changes
                if let Some(original_body) = bodies.get_mut(*body_id) {
                    *original_body = temp_bodies[0].clone();
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
        for body in bodies {
            if body.body_type == BodyType::Dynamic {
                let force = self.acceleration * body.mass * body.gravity_scale;
                body.apply_force(force);
            }
        }
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
        for body in bodies {
            if body.body_type == BodyType::Dynamic {
                let drag_force = -body.velocity * self.coefficient;
                body.apply_force(drag_force);
            }
        }
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
        for body in bodies {
            if body.body_type == BodyType::Dynamic {
                let speed = body.velocity.length();
                if speed > 0.0001 {
                    let drag_magnitude =
                        0.5 * self.density * speed * speed * self.drag_coefficient * self.area;
                    let drag_force = -body.velocity.normalize() * drag_magnitude;
                    body.apply_force(drag_force);
                }
            }
        }
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
        for body in bodies {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            let delta = body.position - self.center;
            let distance = delta.length();

            if distance > self.radius || distance < 0.0001 {
                continue;
            }

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
        }
    }
}

/// Directional force field (wind, current, conveyor belt).
pub struct DirectionalForce {
    /// Direction of the force (normalized).
    pub direction: Vec2,
    /// Magnitude of the force.
    pub strength: f32,
    /// Optional bounds - force only applies within this area.
    pub bounds: Option<gravita_math::AABB>,
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
    pub fn with_bounds(mut self, bounds: gravita_math::AABB) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

impl ForceGenerator for DirectionalForce {
    fn apply(&mut self, bodies: &mut [RigidBody], _dt: f32) {
        for body in bodies {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            // Check if body is within bounds
            if let Some(bounds) = self.bounds
                && !bounds.contains_point(body.position)
            {
                continue;
            }

            // Apply base force
            let mut force = self.direction * self.strength;

            // Add turbulence (simple random variation)
            if self.turbulence > 0.0 {
                let noise_x = (body.position.x * 0.1).sin() * self.turbulence;
                let noise_y = (body.position.y * 0.1).cos() * self.turbulence;
                force += Vec2::new(noise_x, noise_y) * self.strength;
            }

            body.apply_force(force);
        }
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
        for body in bodies {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            let aabb = body.get_world_aabb();

            // Check if body is at least partially submerged
            if aabb.min.y > self.water_level {
                continue; // Above water
            }

            // Calculate submerged volume (simplified for 2D)
            let submerged_height = (self.water_level - aabb.min.y).min(aabb.size().y);
            let submerged_ratio = submerged_height / aabb.size().y;
            let submerged_area = aabb.size().x * submerged_height;

            // Buoyancy force (Archimedes' principle)
            let buoyancy = Vec2::new(0.0, self.liquid_density * submerged_area * 9.81);
            body.apply_force(buoyancy);

            // Drag in fluid
            let relative_velocity = body.velocity - self.flow_velocity;
            let speed = relative_velocity.length();

            if speed > 0.0001 {
                let drag_magnitude = 0.5
                    * self.liquid_density
                    * speed
                    * speed
                    * self.drag_coefficient
                    * submerged_area
                    * submerged_ratio;
                let drag = -relative_velocity.normalize() * drag_magnitude;
                body.apply_force(drag);
            }
        }
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
                let distance = delta.length();

                if distance < 0.001 {
                    continue;
                }

                let direction = delta / distance;

                let magnitude = match self.force_type {
                    FieldType::Attractive => strength / (distance * distance),
                    FieldType::Repulsive => -strength / (distance * distance),
                    FieldType::Dipole => {
                        // Alternates between attractive and repulsive
                        strength * (distance * 0.5).cos() / (distance * distance)
                    },
                };

                total_force += direction * magnitude * body.mass;
            }

            body.apply_force(total_force);
        }
    }
}
