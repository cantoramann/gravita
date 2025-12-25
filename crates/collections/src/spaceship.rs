use gravita_math::Vec2;
use gravita_renderer::{draw_circle, draw_line};

/// Simple thrust-and-rotate 2D spaceship.
///
/// Designed for top-down or side-view arcade games.
/// This type handles its own kinematic motion and rendering,
/// but leaves input handling and world boundaries to the caller.
pub struct Spaceship {
    /// World-space position of the ship's center.
    pub position: Vec2,
    /// Linear velocity in world units per second.
    pub velocity: Vec2,
    /// Facing angle in radians (0 = +X, counter-clockwise positive).
    pub angle: f32,
    /// Angular velocity in radians per second.
    pub angular_velocity: f32,

    /// Maximum thrust force magnitude when fully engaged.
    pub thrust_power: f32,
    /// Angular acceleration in radians per second² when turn input is ±1.
    pub turn_accel: f32,
    /// Linear damping applied every frame (simple drag).
    pub linear_damping: f32,
    /// Angular damping applied every frame (rotational drag).
    pub angular_damping: f32,
    /// Maximum linear speed (soft clamp).
    pub max_speed: f32,

    /// Visual radius used to scale the triangle.
    pub size: f32,

    // Cached input state for the next update step.
    thrust_input: f32,
    turn_input: f32,
}

impl Spaceship {
    /// Create a new spaceship at the given position, facing right.
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
            thrust_power: 600.0,
            turn_accel: 6.0,
            linear_damping: 0.4,
            angular_damping: 2.0,
            max_speed: 500.0,
            size: 18.0,
            thrust_input: 0.0,
            turn_input: 0.0,
        }
    }

    /// Set control inputs for this frame.
    ///
    /// - `thrust` in \[0, 1\] controls forward thrust.
    /// - `turn` in \[-1, 1\] controls rotation (negative = left, positive = right).
    pub fn set_input(&mut self, thrust: f32, turn: f32) {
        self.thrust_input = thrust.clamp(0.0, 1.0);
        self.turn_input = turn.clamp(-1.0, 1.0);
    }

    /// Advance the spaceship motion by `dt` seconds.
    ///
    /// This is purely kinematic and does not perform collision detection.
    pub fn update(&mut self, dt: f32) {
        // Integrate rotation from input (inertial turning, not instant).
        let angular_accel = self.turn_input * self.turn_accel;
        self.angular_velocity += angular_accel * dt;

        // Rotational drag to keep spin from growing unbounded.
        self.angular_velocity *= 1.0 / self.angular_damping.mul_add(dt, 1.0);

        self.angle += self.angular_velocity * dt;

        // Integrate linear motion from thrust + damping.
        if self.thrust_input > 0.0 {
            let forward = Vec2::new(self.angle.cos(), self.angle.sin());
            let accel = forward * (self.thrust_power * self.thrust_input);
            self.velocity += accel * dt;
        }

        // Simple linear drag.
        self.velocity *= 1.0 / self.linear_damping.mul_add(dt, 1.0);

        // Soft clamp on speed for better control.
        let speed_sq = self.velocity.length_squared();
        let max_speed_sq = self.max_speed * self.max_speed;
        if speed_sq > max_speed_sq {
            let factor = self.max_speed / speed_sq.sqrt();
            self.velocity *= factor;
        }

        self.position += self.velocity * dt;
    }

    /// Draw the spaceship as a triangle with an optional flame when thrusting.
    ///
    /// The coordinate system is standard gravita world space (y up).
    pub fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        let nose = self.position + Vec2::new(self.angle.cos(), self.angle.sin()) * self.size;

        // Compute left and right wing positions by rotating ±120° around the nose direction.
        let left_angle = self.angle + 2.0 * std::f32::consts::PI / 3.0;
        let right_angle = self.angle - 2.0 * std::f32::consts::PI / 3.0;
        let wing_radius = self.size * 0.7;

        let left = self.position + Vec2::new(left_angle.cos(), left_angle.sin()) * wing_radius;
        let right = self.position + Vec2::new(right_angle.cos(), right_angle.sin()) * wing_radius;

        let hull_color = [0xE0, 0xE0, 0xE0, 0xFF];
        let accent_color = [0x40, 0x80, 0xFF, 0xFF];

        // Hull outline.
        draw_line(frame, nose, left, hull_color, width, height);
        draw_line(frame, left, right, hull_color, width, height);
        draw_line(frame, right, nose, hull_color, width, height);

        // Cockpit as a small circle near the nose.
        let cockpit = (nose + self.position) * 0.5;
        draw_circle(
            frame,
            cockpit,
            self.size * 0.25,
            accent_color,
            width,
            height,
        );

        // Engine flame when thrusting.
        if self.thrust_input > 0.01 {
            let back =
                self.position - Vec2::new(self.angle.cos(), self.angle.sin()) * (self.size * 0.5);
            let flame_dir = -Vec2::new(self.angle.cos(), self.angle.sin());
            let flame_len = self.size * 0.4f32.mul_add(self.thrust_input, 0.8);
            let flame_tip = back + flame_dir * flame_len;

            let flame_color = [0xFF, 0x80, 0x20, 0xFF];
            draw_line(frame, back, flame_tip, flame_color, width, height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn new_spaceship_at_position() {
        let ship = Spaceship::new(Vec2::new(100.0, 200.0));
        assert_eq!(ship.position, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn new_spaceship_facing_right() {
        let ship = Spaceship::new(Vec2::ZERO);
        assert_eq!(ship.angle, 0.0);
    }

    #[test]
    fn new_spaceship_is_stationary() {
        let ship = Spaceship::new(Vec2::ZERO);
        assert_eq!(ship.velocity, Vec2::ZERO);
        assert_eq!(ship.angular_velocity, 0.0);
    }

    #[test]
    fn new_spaceship_has_default_properties() {
        let ship = Spaceship::new(Vec2::ZERO);
        assert!(ship.thrust_power > 0.0);
        assert!(ship.max_speed > 0.0);
        assert!(ship.size > 0.0);
    }

    // =========================================================================
    // Input Handling
    // =========================================================================

    #[test]
    fn set_input_thrust_clamped_to_0_1() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.set_input(2.0, 0.0);
        ship.update(0.016);
        // Thrust should be clamped to 1.0 (max), ship should accelerate
        assert!(ship.velocity.length() > 0.0);
    }

    #[test]
    fn set_input_turn_clamped() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.set_input(0.0, 5.0);
        ship.update(0.016);
        // Turn should be clamped to 1.0, angular velocity should change
        assert!(ship.angular_velocity != 0.0);
    }

    #[test]
    fn set_input_negative_thrust_ignored() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.set_input(-1.0, 0.0);
        ship.update(0.016);
        // Negative thrust clamped to 0, ship should not accelerate
        assert!(ship.velocity.length() < 0.1);
    }

    // =========================================================================
    // Movement and Physics
    // =========================================================================

    #[test]
    fn update_thrust_accelerates_forward() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.set_input(1.0, 0.0);
        ship.update(0.1);
        // Facing right (angle = 0), should accelerate in +X direction
        assert!(ship.velocity.x > 0.0);
    }

    #[test]
    fn update_turn_changes_angle() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.set_input(0.0, 1.0);
        ship.update(0.1);
        // Should have rotated
        assert!(ship.angle != 0.0);
    }

    #[test]
    fn update_velocity_moves_position() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.velocity = Vec2::new(100.0, 0.0);
        ship.update(0.1);
        // Position should have moved
        assert!(ship.position.x > 0.0);
    }

    #[test]
    fn update_applies_linear_damping() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.velocity = Vec2::new(100.0, 0.0);
        let initial_speed = ship.velocity.length();
        ship.update(0.1);
        // Speed should decrease due to damping
        assert!(ship.velocity.length() < initial_speed);
    }

    #[test]
    fn update_applies_angular_damping() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.angular_velocity = 5.0;
        let initial_angular = ship.angular_velocity;
        ship.update(0.1);
        // Angular velocity should decrease due to damping
        assert!(ship.angular_velocity.abs() < initial_angular.abs());
    }

    #[test]
    fn update_clamps_max_speed() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.velocity = Vec2::new(10000.0, 0.0);
        ship.update(0.016);
        // Speed should be clamped to max_speed
        assert!(ship.velocity.length() <= ship.max_speed + 1.0);
    }

    #[test]
    fn thrust_in_rotated_direction() {
        let mut ship = Spaceship::new(Vec2::ZERO);
        ship.angle = std::f32::consts::FRAC_PI_2; // Facing up
        ship.set_input(1.0, 0.0);
        ship.update(0.1);
        // Should accelerate in +Y direction
        assert!(ship.velocity.y > ship.velocity.x);
    }

    // =========================================================================
    // Rendering (smoke test)
    // =========================================================================

    #[test]
    fn render_does_not_panic() {
        let ship = Spaceship::new(Vec2::new(400.0, 300.0));
        let mut frame = vec![0u8; 800 * 600 * 4];
        ship.render(&mut frame, 800, 600);
    }

    #[test]
    fn render_with_thrust_does_not_panic() {
        let mut ship = Spaceship::new(Vec2::new(400.0, 300.0));
        ship.set_input(1.0, 0.5);
        ship.update(0.016);
        let mut frame = vec![0u8; 800 * 600 * 4];
        ship.render(&mut frame, 800, 600);
    }

    #[test]
    fn render_at_edge_does_not_panic() {
        let ship = Spaceship::new(Vec2::new(0.0, 0.0));
        let mut frame = vec![0u8; 800 * 600 * 4];
        ship.render(&mut frame, 800, 600);
    }
}
