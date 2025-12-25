use gravita_math::Vec2;
use gravita_renderer::{draw_circle, draw_line};

/// Simple animated stickman character with walking and jumping logic.
///
/// This type owns its own movement state and knows how to render itself,
/// but it does **not** manage input or the main loop.
pub struct Stickman {
    // Horizontal motion
    position_x: f32,
    velocity_x: f32,
    target_velocity_x: f32,

    // Vertical motion relative to the ground line (world up).
    base_y: f32,
    vertical_pos: f32,
    vertical_velocity: f32,

    // Proportions
    head_radius: f32,
    torso_length: f32,
    leg_length: f32,
    arm_length: f32,
    foot_separation: f32,

    // Current arm angles (radians, in screen/world space).
    right_arm_angle: f32,
    left_arm_angle: f32,
}

impl Stickman {
    /// Create a new stickman whose feet rest on the given ground y-coordinate.
    pub fn new(base_y: f32, screen_width: f32) -> Self {
        let head_radius = 10.0;
        let torso_length = 40.0;
        let leg_length = 35.0;
        let arm_length = 30.0;
        let foot_separation = 10.0;

        // Idle arms hanging mostly down.
        let right_arm_angle = std::f32::consts::FRAC_PI_2;
        let left_arm_angle = std::f32::consts::FRAC_PI_2;

        Self {
            position_x: screen_width * 0.5,
            velocity_x: 0.0,
            target_velocity_x: 0.0,
            base_y,
            vertical_pos: 0.0,
            vertical_velocity: 0.0,
            head_radius,
            torso_length,
            leg_length,
            arm_length,
            foot_separation,
            right_arm_angle,
            left_arm_angle,
        }
    }

    /// Ground y-coordinate the stickman's feet rest on.
    pub fn ground_y(&self) -> f32 {
        self.base_y
    }

    /// Set desired horizontal movement direction: -1 = left, 0 = idle, +1 = right.
    pub fn set_move_direction(&mut self, dir: f32) {
        let max_speed = 200.0;
        self.target_velocity_x = dir.clamp(-1.0, 1.0) * max_speed;
    }

    /// Trigger a jump if close enough to the ground.
    pub fn jump(&mut self) {
        if self.vertical_pos.abs() < 1.0 && self.vertical_velocity.abs() < 1.0 {
            self.vertical_velocity = 320.0;
        }
    }

    /// Advance the internal animation and physics state by `dt` seconds.
    pub fn update(&mut self, dt: f32, screen_width: f32) {
        // Smooth acceleration toward target velocity.
        let accel = 800.0;
        if self.velocity_x < self.target_velocity_x {
            self.velocity_x += accel * dt;
            if self.velocity_x > self.target_velocity_x {
                self.velocity_x = self.target_velocity_x;
            }
        } else if self.velocity_x > self.target_velocity_x {
            self.velocity_x -= accel * dt;
            if self.velocity_x < self.target_velocity_x {
                self.velocity_x = self.target_velocity_x;
            }
        }

        self.position_x += self.velocity_x * dt;

        // Keep stickman within screen bounds (simple clamping).
        let margin = 50.0;
        let min_x = margin;
        let max_x = screen_width - margin;
        if self.position_x < min_x {
            self.position_x = min_x;
            self.velocity_x = 0.0;
        } else if self.position_x > max_x {
            self.position_x = max_x;
            self.velocity_x = 0.0;
        }

        // Vertical motion for jumping.
        const GRAVITY: f32 = 800.0; // world up (positive), so gravity is negative
        self.vertical_velocity -= GRAVITY * dt;
        self.vertical_pos += self.vertical_velocity * dt;

        // Clamp to ground.
        if self.vertical_pos < 0.0 {
            self.vertical_pos = 0.0;
            self.vertical_velocity = 0.0;
        }

        let jumping = self.vertical_pos > 2.0 || self.vertical_velocity.abs() > 2.0;

        // Determine target arm angles based on movement and jump state.
        let speed = self.velocity_x;
        let idle = speed.abs() < 5.0;

        // Idle: both arms partially lowered (roughly straight down).
        let idle_right = std::f32::consts::FRAC_PI_2;
        let idle_left = std::f32::consts::FRAC_PI_2;

        // Jumping: both arms slightly lowered but more active than idle.
        let jump_right = std::f32::consts::FRAC_PI_2 + 0.2;
        let jump_left = std::f32::consts::FRAC_PI_2 - 0.2;

        // Moving right: raise right arm up and forward, keep left near idle.
        let move_right_right = -0.9; // up and to the right
        let move_right_left = idle_left;

        // Moving left: raise left arm up and forward, keep right near idle.
        let move_left_left = std::f32::consts::PI + 0.9; // up and to the left
        let move_left_right = idle_right;

        let (target_right, target_left) = if jumping {
            (jump_right, jump_left)
        } else if idle {
            (idle_right, idle_left)
        } else if speed > 0.0 {
            (move_right_right, move_right_left)
        } else {
            (move_left_right, move_left_left)
        };

        // Smoothly interpolate arm angles toward targets.
        let arm_lerp_rate = 8.0;
        self.right_arm_angle += (target_right - self.right_arm_angle) * arm_lerp_rate * dt;
        self.left_arm_angle += (target_left - self.left_arm_angle) * arm_lerp_rate * dt;
    }

    /// Render the stickman into the given RGBA frame.
    ///
    /// The coordinate system is the same as your world: y increases upward.
    pub fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        // Compute joint positions (apply vertical offset for jumping).
        let pos_x = self.position_x;
        let feet_y = self.base_y - self.vertical_pos;
        let hip_y = feet_y - self.leg_length;
        let shoulder_y = hip_y - self.torso_length;
        let head_center_y = self.head_radius.mul_add(-1.5, shoulder_y);

        let hip = Vec2::new(pos_x, hip_y);
        let shoulder = Vec2::new(pos_x, shoulder_y);
        let head_center = Vec2::new(pos_x, head_center_y);

        // Feet positions.
        let left_foot = Vec2::new(pos_x - self.foot_separation, feet_y);
        let right_foot = Vec2::new(pos_x + self.foot_separation, feet_y);

        // Arm endpoints from shoulder.
        let right_dir = Vec2::new(self.right_arm_angle.cos(), self.right_arm_angle.sin());
        let left_dir = Vec2::new(self.left_arm_angle.cos(), self.left_arm_angle.sin());
        let right_hand = shoulder + right_dir * self.arm_length;
        let left_hand = shoulder + left_dir * self.arm_length;

        let black = [0x00, 0x00, 0x00, 0xff];

        // Legs.
        draw_line(frame, hip, left_foot, black, width, height);
        draw_line(frame, hip, right_foot, black, width, height);

        // Torso.
        draw_line(frame, hip, shoulder, black, width, height);

        // Arms.
        draw_line(frame, shoulder, right_hand, black, width, height);
        draw_line(frame, shoulder, left_hand, black, width, height);

        // Head.
        draw_circle(frame, head_center, self.head_radius, black, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.1
    }

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn new_stickman_starts_centered() {
        let stickman = Stickman::new(100.0, 800.0);
        assert!(approx_eq(stickman.position_x, 400.0));
    }

    #[test]
    fn new_stickman_on_ground() {
        let stickman = Stickman::new(100.0, 800.0);
        assert_eq!(stickman.ground_y(), 100.0);
        assert_eq!(stickman.vertical_pos, 0.0);
    }

    #[test]
    fn new_stickman_is_stationary() {
        let stickman = Stickman::new(100.0, 800.0);
        assert_eq!(stickman.velocity_x, 0.0);
        assert_eq!(stickman.vertical_velocity, 0.0);
    }

    // =========================================================================
    // Horizontal Movement
    // =========================================================================

    #[test]
    fn set_move_direction_right() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.set_move_direction(1.0);
        assert!(stickman.target_velocity_x > 0.0);
    }

    #[test]
    fn set_move_direction_left() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.set_move_direction(-1.0);
        assert!(stickman.target_velocity_x < 0.0);
    }

    #[test]
    fn set_move_direction_clamps_input() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.set_move_direction(5.0);
        // Should be clamped to max speed
        let max_speed = 200.0;
        assert_eq!(stickman.target_velocity_x, max_speed);
    }

    #[test]
    fn update_accelerates_toward_target() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.set_move_direction(1.0);
        stickman.update(0.1, 800.0);
        // Velocity should have increased
        assert!(stickman.velocity_x > 0.0);
    }

    #[test]
    fn update_decelerates_when_stopping() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.set_move_direction(1.0);
        stickman.update(1.0, 800.0); // Get up to speed
        stickman.set_move_direction(0.0);
        let prev_velocity = stickman.velocity_x;
        stickman.update(0.1, 800.0);
        // Velocity should decrease toward zero
        assert!(stickman.velocity_x < prev_velocity);
    }

    #[test]
    fn update_clamps_position_at_left_edge() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.position_x = 0.0;
        stickman.set_move_direction(-1.0);
        stickman.update(1.0, 800.0);
        // Should be clamped at margin
        assert!(stickman.position_x >= 50.0);
    }

    #[test]
    fn update_clamps_position_at_right_edge() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.position_x = 800.0;
        stickman.set_move_direction(1.0);
        stickman.update(1.0, 800.0);
        // Should be clamped at (width - margin)
        assert!(stickman.position_x <= 750.0);
    }

    // =========================================================================
    // Jumping
    // =========================================================================

    #[test]
    fn jump_sets_vertical_velocity() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.jump();
        assert!(stickman.vertical_velocity > 0.0);
    }

    #[test]
    fn jump_not_allowed_when_airborne() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.jump();
        let initial_velocity = stickman.vertical_velocity;
        stickman.vertical_pos = 50.0; // Simulate being in air
        stickman.jump();
        // Should not jump again while airborne
        assert_eq!(stickman.vertical_velocity, initial_velocity);
    }

    #[test]
    fn update_applies_gravity() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.jump();
        let initial_velocity = stickman.vertical_velocity;
        stickman.update(0.1, 800.0);
        // Gravity should reduce upward velocity
        assert!(stickman.vertical_velocity < initial_velocity);
    }

    #[test]
    fn update_clamps_to_ground() {
        let mut stickman = Stickman::new(100.0, 800.0);
        stickman.vertical_pos = -10.0; // Below ground
        stickman.update(0.016, 800.0);
        // Should be clamped to ground
        assert_eq!(stickman.vertical_pos, 0.0);
    }

    // =========================================================================
    // Rendering (smoke test)
    // =========================================================================

    #[test]
    fn render_does_not_panic() {
        let stickman = Stickman::new(300.0, 800.0);
        let mut frame = vec![0u8; 800 * 600 * 4];
        stickman.render(&mut frame, 800, 600);
        // Just verify it doesn't panic
    }

    #[test]
    fn render_after_movement_does_not_panic() {
        let mut stickman = Stickman::new(300.0, 800.0);
        stickman.set_move_direction(1.0);
        stickman.update(0.5, 800.0);
        stickman.jump();
        stickman.update(0.5, 800.0);
        let mut frame = vec![0u8; 800 * 600 * 4];
        stickman.render(&mut frame, 800, 600);
    }
}
