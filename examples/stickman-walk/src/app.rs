// Allow deprecated winit APIs - migration to new API is tracked separately
#![allow(deprecated)]

use std::time::Instant;

use gravita_collections::Stickman;
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_line};
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

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

struct App {
    stickman: Stickman,
}

impl App {
    fn new() -> Self {
        let ground_offset = 40.0;
        let base_y = HEIGHT as f32 - ground_offset;

        let stickman = Stickman::new(base_y, WIDTH as f32);

        Self { stickman }
    }

    fn set_move_direction(&mut self, dir: f32) {
        self.stickman.set_move_direction(dir);
    }

    fn jump(&mut self) {
        self.stickman.jump();
    }

    fn update(&mut self, dt: f32) {
        self.stickman.update(dt, WIDTH as f32);
    }

    fn render(&self, frame: &mut [u8]) {
        // Background: pure white.
        clear(frame, [0xff, 0xff, 0xff, 0xff]);

        // Draw ground line slightly above bottom.
        let ground_y = self.stickman.ground_y().round();
        draw_line(
            frame,
            Vec2::new(0.0, ground_y),
            Vec2::new(WIDTH as f32, ground_y),
            [0x00, 0x00, 0x00, 0xff],
            WIDTH,
            HEIGHT,
        );
        // Draw stickman body.
        self.stickman.render(frame, WIDTH, HEIGHT);
    }
}

pub fn run_native() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window_attributes = WindowAttributes::default()
        .with_title("Stickman Walk Demo")
        .with_inner_size(size)
        .with_min_inner_size(size);
    let window = Box::leak(Box::new(event_loop.create_window(window_attributes)?));
    let window = &*window;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut demo = App::new();
    let mut last_time = Instant::now();
    let mut accumulator = 0.0_f32;

    event_loop.run(move |event, event_loop| {
        match event {
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
            Event::DeviceEvent { event, .. } => {
                input.process_device_event(&event);
            },
            Event::NewEvents(_) => {
                input.step();
            },
            Event::AboutToWait => {
                input.end_step();

                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                accumulator += dt;

                // Determine input direction based on held keys.
                let mut dir = 0.0;
                if input.key_held(KeyCode::ArrowLeft) {
                    dir -= 1.0;
                }
                if input.key_held(KeyCode::ArrowRight) {
                    dir += 1.0;
                }
                demo.set_move_direction(dir);

                // Jump on space press.
                if input.key_pressed(KeyCode::Space) {
                    demo.jump();
                }

                while accumulator >= FIXED_TIMESTEP {
                    demo.update(FIXED_TIMESTEP);
                    accumulator -= FIXED_TIMESTEP;
                }

                // Exit on Escape.
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    event_loop.exit();
                    return;
                }

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
