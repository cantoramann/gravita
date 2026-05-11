//! Window + event-loop runner for 3D examples.
//!
//! Mirrors the structure of `gravita-example-shim` but constructs a
//! `Renderer3D` (with a real `wgpu` surface) instead of a `pixels` framebuffer.

// Diagnostic errors during window/surface setup go to stderr — no log
// subscriber is wired up in the example scaffolding.
#![allow(clippy::print_stderr)]

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub use gravita_input::{Input, KeyCode, MouseButton};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

use crate::renderer::Renderer3D;

/// Window + timing configuration for [`run`].
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: &'static str,
    /// Initial window width.
    pub width: u32,
    /// Initial window height.
    pub height: u32,
    /// Fixed simulation timestep in seconds.
    pub fixed_timestep: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Gravita 3D Example",
            width: 1280,
            height: 720,
            fixed_timestep: 1.0 / 60.0,
        }
    }
}

/// User-supplied application logic.
pub trait App3D: 'static {
    /// One-time hook to register meshes with the renderer.
    fn setup(&mut self, renderer: &mut Renderer3D);
    /// Step the simulation forward by `dt` seconds at a fixed timestep.
    fn update(&mut self, dt: f32, input: &Input);
    /// Submit draw calls for the current frame.
    fn render(&self, renderer: &mut Renderer3D);
}

struct Runner<A: App3D> {
    app: A,
    config: WindowConfig,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer3D>,
    input: Input,
    last_time: Option<Instant>,
    accumulator: f32,
}

impl<A: App3D> Runner<A> {
    fn new(config: WindowConfig, app: A) -> Self {
        Self {
            app,
            config,
            window: None,
            renderer: None,
            input: Input::default(),
            last_time: None,
            accumulator: 0.0,
        }
    }
}

impl<A: App3D> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let size = LogicalSize::new(self.config.width as f64, self.config.height as f64);
        let attrs = WindowAttributes::default()
            .with_title(self.config.title)
            .with_inner_size(size)
            .with_min_inner_size(LogicalSize::new(320.0, 240.0));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(err) => {
                eprintln!("create_window failed: {err}");
                event_loop.exit();
                return;
            },
        };
        let renderer = match Renderer3D::new(Arc::clone(&window)) {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Renderer3D::new failed: {err}");
                event_loop.exit();
                return;
            },
        };
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.last_time = Some(Instant::now());
        if let Some(renderer) = self.renderer.as_mut() {
            self.app.setup(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        ElementState::Pressed if !key_event.repeat => {
                            self.input.record_key_press(code);
                        },
                        ElementState::Released => {
                            self.input.record_key_release(code);
                        },
                        ElementState::Pressed => {},
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_cursor(Some((position.x as f32, position.y as f32)));
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.input.record_mouse_press(button),
                ElementState::Released => self.input.record_mouse_release(button),
            },
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(new_size.width, new_size.height);
                }
            },
            WindowEvent::RedrawRequested => {
                // App is responsible for assembling the camera + instance list
                // and calling `renderer.render(...)`. Surface errors caused by
                // a resize race are absorbed by the next frame; everything else
                // exits the loop.
                if let Some(renderer) = self.renderer.as_mut() {
                    self.app.render(renderer);
                }
            },
            _ => {},
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(last) = self.last_time else { return };
        let now = Instant::now();
        let dt = (now - last).as_secs_f32();
        self.last_time = Some(now);

        if self.input.key_pressed(KeyCode::Escape) {
            event_loop.exit();
            return;
        }

        self.accumulator += dt;
        let step = self.config.fixed_timestep;
        if self.accumulator > step * 8.0 {
            self.accumulator = step * 8.0;
        }
        while self.accumulator >= step {
            self.app.update(step, &self.input);
            self.accumulator -= step;
        }

        self.input.begin_frame();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }

        let target = now + Duration::from_secs_f32(step);
        event_loop.set_control_flow(ControlFlow::WaitUntil(target));
    }
}

/// Build the window, initialise wgpu, and drive an [`App3D`].
pub fn run<A: App3D>(config: WindowConfig, app: A) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = Runner::new(config, app);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

// Backwards-compatible aliases — older example code used these names.
pub use gravita_input::{KeyCode as ShimKeyCode, MouseButton as ShimMouseButton};
