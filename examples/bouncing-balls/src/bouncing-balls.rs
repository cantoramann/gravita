// examples/src/bouncing_balls.rs
// Allow deprecated winit APIs - migration to new API is tracked separately
#![allow(deprecated)]

use std::time::Instant;

use gravita_math::{Circle, Vec2, AABB};
use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody};
use gravita_renderer::{clear, draw_circle, draw_line};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowAttributes,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

struct Demo {
    physics_world: PhysicsWorld,
    accumulator: f32,
}

impl Demo {
    fn new() -> Self {
        let mut physics_world = PhysicsWorld::new();

        // Set gravity pointing down
        physics_world.set_gravity(Vec2::new(0.0, -500.0));

        // Create ground (static AABB)
        let ground_shape =
            CollisionShape::AABB(AABB::from_center_size(Vec2::ZERO, Vec2::new(700.0, 50.0)));
        let ground = RigidBody::new(0, ground_shape)
            .with_type(BodyType::Static)
            .with_position(Vec2::new(400.0, 50.0));
        physics_world.add_body(ground);

        // Create walls
        let left_wall = RigidBody::new(
            0,
            CollisionShape::AABB(AABB::from_center_size(Vec2::ZERO, Vec2::new(50.0, 600.0))),
        )
        .with_type(BodyType::Static)
        .with_position(Vec2::new(25.0, 300.0));
        physics_world.add_body(left_wall);

        let right_wall = RigidBody::new(
            0,
            CollisionShape::AABB(AABB::from_center_size(Vec2::ZERO, Vec2::new(50.0, 600.0))),
        )
        .with_type(BodyType::Static)
        .with_position(Vec2::new(775.0, 300.0));
        physics_world.add_body(right_wall);

        // Create some bouncing balls
        for i in 0..5 {
            let radius = 20.0 + (i as f32) * 5.0;
            let ball_shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, radius));

            let ball = RigidBody::new(0, ball_shape)
                .with_type(BodyType::Dynamic)
                .with_position(Vec2::new(200.0 + i as f32 * 100.0, 400.0 + i as f32 * 30.0))
                // Start with no horizontal or vertical velocity; gravity will take over.
                .with_velocity(Vec2::new(0.0, 0.0))
                .with_density(1.0);

            let mut ball = ball;
            ball.restitution = 0.7; // Make them bouncy
            ball.friction = 0.3;

            physics_world.add_body(ball);
        }

        Self {
            physics_world,
            accumulator: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        // Fixed timestep with accumulator
        self.accumulator += dt;

        while self.accumulator >= FIXED_TIMESTEP {
            self.physics_world.step(FIXED_TIMESTEP);
            self.accumulator -= FIXED_TIMESTEP;
        }
    }

    fn render(&self, frame: &mut [u8]) {
        // Clear screen with dark blue
        clear(frame, [0x20, 0x20, 0x40, 0xff]);

        // Draw all bodies (currently: ground, walls, and balls)
        for body in self.physics_world.get_bodies() {
            match &body.shape {
                CollisionShape::Circle(circle) => {
                    self.draw_circle(
                        frame,
                        body.position + circle.center,
                        circle.radius,
                        body.body_type,
                    );
                },
                CollisionShape::AABB(aabb) => {
                    self.draw_aabb(
                        frame,
                        aabb.translate(body.position),
                        body.rotation,
                        body.body_type,
                    );
                },
            }
        }
    }

    fn draw_circle(&self, frame: &mut [u8], center: Vec2, radius: f32, body_type: BodyType) {
        let color = match body_type {
            BodyType::Static => [0x60, 0x60, 0x60, 0xff],
            BodyType::Dynamic => [0x40, 0xA0, 0xFF, 0xff],
            BodyType::Kinematic => [0xFF, 0xA0, 0x40, 0xff],
        };

        // Convert world space (y up) to screen space (y down).
        let screen_center = Vec2::new(center.x, HEIGHT as f32 - center.y);
        draw_circle(frame, screen_center, radius, color, WIDTH, HEIGHT);
    }

    fn draw_aabb(&self, frame: &mut [u8], aabb: AABB, rotation: f32, body_type: BodyType) {
        let color = match body_type {
            BodyType::Static => [0x80, 0x80, 0x80, 0xff],
            BodyType::Dynamic => [0xFF, 0x60, 0x60, 0xff],
            BodyType::Kinematic => [0xFF, 0xA0, 0x40, 0xff],
        };

        // Simple axis-aligned drawing (ignoring rotation for now).
        // Use floor/ceil so we don't leave 1-pixel gaps due to truncation.
        let min_x = aabb.min.x.floor() as i32;
        let max_x = aabb.max.x.ceil() as i32;
        let min_y = (HEIGHT as f32 - aabb.max.y).floor() as i32; // Flip Y
        let max_y = (HEIGHT as f32 - aabb.min.y).ceil() as i32; // Flip Y

        for y in min_y.max(0)..max_y.min(HEIGHT as i32) {
            for x in min_x.max(0)..max_x.min(WIDTH as i32) {
                // If rotation is significant, we could transform points here
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&color);
                }
            }
        }

        // Draw rotation indicator if rotating
        if rotation.abs() > 0.01 {
            let center = aabb.center();
            let direction = Vec2::new(rotation.cos(), rotation.sin()) * 20.0;
            self.draw_world_line(frame, center, center + direction, [0xFF, 0xFF, 0x00, 0xFF]);
        }
    }

    fn draw_world_line(&self, frame: &mut [u8], start: Vec2, end: Vec2, color: [u8; 4]) {
        let start_screen = Vec2::new(start.x, HEIGHT as f32 - start.y);
        let end_screen = Vec2::new(end.x, HEIGHT as f32 - end.y);
        draw_line(frame, start_screen, end_screen, color, WIDTH, HEIGHT);
    }

    fn spawn_ball_at(&mut self, x: f32, y: f32) {
        let ball_shape = CollisionShape::Circle(Circle::new(Vec2::ZERO, 15.0));

        let ball = RigidBody::new(0, ball_shape)
            .with_type(BodyType::Dynamic)
            .with_position(Vec2::new(x, HEIGHT as f32 - y))
            // Spawn with zero initial velocity so balls drop straight down.
            .with_velocity(Vec2::new(0.0, 0.0))
            .with_density(1.0);

        let mut ball = ball;
        ball.restitution = 0.8;
        ball.friction = 0.2;

        self.physics_world.add_body(ball);
    }
}

// Simple random number generator (replace with rand crate if you prefer)
#[allow(dead_code)]
fn rand() -> i32 {
    static mut SEED: u32 = 0x12345678;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED / 65536) as i32
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window_attributes = WindowAttributes::default()
        .with_title("Physics Engine - Bouncing Balls")
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
                    // Let the helper process events and tell us when to redraw
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
            Event::DeviceEvent { event, .. } => {
                input.process_device_event(&event);
            },
            Event::AboutToWait => {
                // End the input step and compute delta time
                input.end_step();

                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                // Handle input-driven actions
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    event_loop.exit();
                    return;
                }

                if input.mouse_pressed(MouseButton::Left) {
                    if let Some((x, y)) = input.cursor() {
                        demo.spawn_ball_at(x, y);
                    }
                }

                // Update physics and request a redraw
                demo.update(dt);
                window.request_redraw();
            },
            Event::LoopExiting => {},
            _ => {},
        }
    })?;

    Ok(())
}
