// Allow deprecated winit APIs - migration to new API is tracked separately
#![allow(deprecated)]

use std::time::Instant;

use gravita_math::Vec2;
use gravita_renderer::{clear, draw_circle, draw_line};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

struct RodDemo {
    angle: f32,
    angular_velocity: f32,
    angular_acceleration: f32,

    length: f32,
    mass_end: f32,
    inertia: f32, // moment of inertia about the pivot

    damping: f32,
}

impl RodDemo {
    fn new() -> Self {
        // Rod with a point mass at the end.
        let length = 200.0;
        let mass_end = 1.0;
        let inertia = mass_end * length * length;

        // Angle is measured from "straight down" (pendulum convention).
        // angle = 0   -> rod hanging straight down (stable equilibrium)
        // angle = pi  -> rod pointing straight up (unstable equilibrium)
        let angle = std::f32::consts::PI; // start straight up
                                          // Give the rod an initial "hit" so it begins swinging immediately.
                                          // This is equivalent to a one-time angular impulse at t = 0.
        let angular_velocity = 2.0;

        Self {
            angle,
            angular_velocity,
            angular_acceleration: 0.0,
            length,
            mass_end,
            inertia,
            damping: 0.5,
        }
    }

    fn step(&mut self, dt: f32) {
        // Simple pendulum dynamics for a point mass at distance `length`.
        //   torque_gravity = -m * g * L * sin(theta)
        // where theta is measured from downward vertical.
        const G: f32 = 9.81 * 50.0; // scaled gravity for visual effect

        let gravity_torque = -self.mass_end * G * self.length * self.angle.sin();

        // Viscous damping torque opposing motion.
        let damping_torque = -self.damping * self.angular_velocity;

        let net_torque = gravity_torque + damping_torque;

        self.angular_acceleration = net_torque / self.inertia;
        self.angular_velocity += self.angular_acceleration * dt;
        self.angle += self.angular_velocity * dt;
    }

    /// Centrifugal force magnitude on the end mass in the rotating frame.
    fn centrifugal_force(&self) -> f32 {
        self.mass_end * self.angular_velocity * self.angular_velocity * self.length
    }

    fn pivot_position(&self) -> Vec2 {
        Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5)
    }

    fn end_position(&self) -> Vec2 {
        let pivot = self.pivot_position();
        // Convert "angle from down" into standard screen direction.
        let screen_angle = self.angle + std::f32::consts::FRAC_PI_2;
        let dir = Vec2::new(screen_angle.cos(), screen_angle.sin());
        pivot + dir * self.length
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window_attributes = WindowAttributes::default()
        .with_title("Physics Engine - Rotating Rod")
        .with_inner_size(size)
        .with_min_inner_size(size);
    let window = Box::leak(Box::new(event_loop.create_window(window_attributes)?));
    let window = &*window;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut demo = RodDemo::new();
    let mut accumulator = 0.0_f32;
    let mut last_time = Instant::now();

    event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent { event, window_id } => {
                if window_id == window.id() {
                    if let WindowEvent::CloseRequested = event {
                        event_loop.exit();
                    }
                }
            },
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                accumulator += dt;
                while accumulator >= FIXED_TIMESTEP {
                    demo.step(FIXED_TIMESTEP);
                    accumulator -= FIXED_TIMESTEP;
                }

                // Render
                let frame = pixels.frame_mut();

                // Background: pure white
                clear(frame, [0xff, 0xff, 0xff, 0xff]);

                let pivot = demo.pivot_position();
                let end = demo.end_position();

                // Draw rod as a dark line
                draw_line(frame, pivot, end, [0x20, 0x20, 0x20, 0xff], WIDTH, HEIGHT);

                // Draw pivot and end mass
                draw_circle(frame, pivot, 6.0, [0x00, 0x00, 0x00, 0xff], WIDTH, HEIGHT);
                draw_circle(frame, end, 12.0, [0x40, 0x80, 0xff, 0xff], WIDTH, HEIGHT);

                // Visualize "centrifugal" force as a red arrow along the rod direction
                let f_mag = demo.centrifugal_force();
                let dir = (end - pivot).normalize();
                let arrow_len = (f_mag * 0.02).min(80.0); // scale down for display
                let arrow_end = end + dir * arrow_len;
                draw_line(
                    frame,
                    end,
                    arrow_end,
                    [0xff, 0x40, 0x40, 0xff],
                    WIDTH,
                    HEIGHT,
                );

                if let Err(err) = pixels.render() {
                    error!("pixels.render() failed: {}", err);
                    event_loop.exit();
                }
            },
            _ => {},
        }
    })?;

    Ok(())
}
