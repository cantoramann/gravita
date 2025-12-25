// Allow deprecated winit APIs - migration to new API is tracked separately
#![allow(deprecated)]

use std::time::Instant;

use gravita_collections::{Planet, Spaceship};
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_axes};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowAttributes,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

struct Demo {
    spaceship: Spaceship,
    planets: Vec<Planet>,
    accumulator: f32,
}

impl Demo {
    fn new() -> Self {
        let center = Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5);

        let planets = vec![
            Planet::new(center, 40.0),
            Planet::new(center + Vec2::new(-220.0, -120.0), 30.0),
            Planet::new(center + Vec2::new(260.0, 160.0), 24.0),
        ];

        let spaceship = Spaceship::new(center + Vec2::new(0.0, -220.0));

        Self {
            spaceship,
            planets,
            accumulator: 0.0,
        }
    }

    fn apply_gravity(&mut self) {
        // Accumulate gravitational acceleration from all planets.
        let mut accel = Vec2::ZERO;

        for planet in &self.planets {
            let dir = planet.center - self.spaceship.position;
            let dist_sq = dir.length_squared().max(1.0);
            let dist = dist_sq.sqrt();

            // Avoid extreme forces very close to the center.
            if dist < planet.radius * 0.8 {
                continue;
            }

            // Simple inverse-square law with a tuned constant.
            let g = 200000.0;
            let strength = g / dist_sq;
            accel += dir * (strength / dist);
        }

        self.spaceship.velocity += accel * FIXED_TIMESTEP;
    }

    fn update(&mut self, dt: f32) {
        self.accumulator += dt;

        while self.accumulator >= FIXED_TIMESTEP {
            // Gravity is applied in discrete steps to keep it stable.
            self.apply_gravity();

            self.spaceship.update(FIXED_TIMESTEP);

            // Simple screen wrap so the ship never leaves view.
            let mut pos = self.spaceship.position;
            if pos.x < 0.0 {
                pos.x += WIDTH as f32;
            } else if pos.x > WIDTH as f32 {
                pos.x -= WIDTH as f32;
            }
            if pos.y < 0.0 {
                pos.y += HEIGHT as f32;
            } else if pos.y > HEIGHT as f32 {
                pos.y -= HEIGHT as f32;
            }
            self.spaceship.position = pos;

            self.accumulator -= FIXED_TIMESTEP;
        }
    }

    fn set_input(&mut self, input: &WinitInputHelper) {
        let mut thrust = 0.0;
        let mut turn = 0.0;

        if input.key_held(KeyCode::ArrowUp) || input.key_held(KeyCode::Space) {
            thrust = 1.0;
        }
        if input.key_held(KeyCode::ArrowLeft) {
            turn -= 1.0;
        }
        if input.key_held(KeyCode::ArrowRight) {
            turn += 1.0;
        }

        self.spaceship.set_input(thrust, turn);
    }

    fn render(&self, frame: &mut [u8]) {
        // Space background: near-black.
        clear(frame, [0x06, 0x08, 0x10, 0xFF]);

        // Optional debug axes at center.
        let center = Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5);
        draw_axes(frame, center, [0x20, 0x20, 0x40, 0xFF], WIDTH, HEIGHT);

        // Draw planets first so the ship is on top.
        for planet in &self.planets {
            planet.render(frame, WIDTH, HEIGHT);
        }

        // Draw the spaceship last.
        self.spaceship.render(frame, WIDTH, HEIGHT);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window_attributes = WindowAttributes::default()
        .with_title("Gravity Arena")
        .with_inner_size(size)
        .with_min_inner_size(size);
    let window = Box::leak(Box::new(event_loop.create_window(window_attributes)?));
    let window = &*window;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut demo = Demo::new();
    let mut last_time = Instant::now();

    event_loop.run(move |event, event_loop| {
        match event {
            Event::NewEvents(_) => {
                input.step();
            },
            Event::WindowEvent { event, window_id } => {
                if window_id == window.id() {
                    if input.process_window_event(&event) {
                        demo.render(pixels.frame_mut());
                        if let Err(err) = pixels.render() {
                            error!("pixels.render() failed: {}", err);
                            event_loop.exit();
                            return;
                        }
                    }

                    if let WindowEvent::CloseRequested = event {
                        event_loop.exit();
                    }
                }
            },
            Event::AboutToWait => {
                input.end_step();

                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                // Exit on Escape.
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    event_loop.exit();
                    return;
                }

                // Update controls and physics.
                demo.set_input(&input);
                demo.update(dt);

                demo.render(pixels.frame_mut());
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
