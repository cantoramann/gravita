use gravita_example_shim::{App, Input, WindowConfig, run};
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_circle, draw_line, palette};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct RodDemo {
    angle: f32,
    angular_velocity: f32,
    angular_acceleration: f32,
    length: f32,
    mass_end: f32,
    inertia: f32,
    damping: f32,
}

impl RodDemo {
    fn new() -> Self {
        let length = 200.0;
        let mass_end = 1.0;
        let inertia = mass_end * length * length;
        Self {
            // Angle is measured from "straight down" (pendulum convention).
            // Start straight up with a small kick so it begins swinging immediately.
            angle: std::f32::consts::PI,
            angular_velocity: 2.0,
            angular_acceleration: 0.0,
            length,
            mass_end,
            inertia,
            damping: 0.5,
        }
    }

    fn centrifugal_force(&self) -> f32 {
        self.mass_end * self.angular_velocity * self.angular_velocity * self.length
    }

    fn pivot_position() -> Vec2 {
        Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5)
    }

    fn end_position(&self) -> Vec2 {
        let pivot = Self::pivot_position();
        // Convert "angle from down" into standard screen direction.
        let screen_angle = self.angle + std::f32::consts::FRAC_PI_2;
        let dir = Vec2::new(screen_angle.cos(), screen_angle.sin());
        pivot + dir * self.length
    }
}

impl App for RodDemo {
    fn update(&mut self, dt: f32, _input: &Input) {
        // Pendulum dynamics for a point mass at distance `length`.
        //   torque_gravity = -m * g * L * sin(theta)
        // Scaled gravity (real 9.81 m/s² is too slow at pixel scale).
        const G: f32 = 9.81 * 50.0;
        let gravity_torque = -self.mass_end * G * self.length * self.angle.sin();
        let damping_torque = -self.damping * self.angular_velocity;
        let net_torque = gravity_torque + damping_torque;
        self.angular_acceleration = net_torque / self.inertia;
        self.angular_velocity += self.angular_acceleration * dt;
        self.angle += self.angular_velocity * dt;
    }

    fn render(&self, frame: &mut [u8]) {
        clear(frame, palette::WHITE);
        let pivot = Self::pivot_position();
        let end = self.end_position();
        draw_line(frame, pivot, end, [0x20, 0x20, 0x20, 0xff], WIDTH, HEIGHT);
        draw_circle(frame, pivot, 6.0, palette::BLACK, WIDTH, HEIGHT);
        draw_circle(frame, end, 12.0, [0x40, 0x80, 0xff, 0xff], WIDTH, HEIGHT);

        // Centrifugal force as a red arrow.
        let f_mag = self.centrifugal_force();
        let dir = (end - pivot).normalize();
        let arrow_len = (f_mag * 0.02).min(80.0);
        let arrow_end = end + dir * arrow_len;
        draw_line(
            frame,
            end,
            arrow_end,
            [0xff, 0x40, 0x40, 0xff],
            WIDTH,
            HEIGHT,
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Physics Engine - Rotating Rod",
            width: WIDTH,
            height: HEIGHT,
            ..WindowConfig::default()
        },
        RodDemo::new(),
    )
}
