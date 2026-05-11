//! Shared windowing/event-loop scaffolding for Gravita examples.
//!
//! Each desktop example used to duplicate ~80 LOC of `winit` + `pixels` glue:
//! `EventLoop` construction, `WindowAttributes`, `Pixels::new`, a per-frame
//! `match Event` over the deprecated `event_loop.run` callback, plus a
//! roll-your-own fixed-timestep accumulator. This crate replaces that with a
//! single [`run`] call driving a user-supplied [`App`] trait.
//!
//! ```no_run
//! use gravita_example_shim::{App, Input, WindowConfig, run};
//!
//! struct Demo;
//! impl App for Demo {
//!     fn update(&mut self, _dt: f32, _input: &Input) {}
//!     fn render(&self, _frame: &mut [u8]) {}
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run(
//!         WindowConfig {
//!             title: "Demo",
//!             ..WindowConfig::default()
//!         },
//!         Demo,
//!     )
//! }
//! ```
//!
//! Runtime characteristics:
//! - Uses the modern `winit::application::ApplicationHandler` API (no `#[allow(deprecated)]`).
//! - Drives [`App::update`] with a fixed timestep + accumulator.
//! - Uses [`ControlFlow::WaitUntil`] between frames so the example doesn't pin a CPU core at 100%.

#![warn(missing_docs)]
// Diagnostic errors (window creation, surface rendering) go to stderr.
// Acceptable in example scaffolding where there is no log subscriber wired up.
#![allow(clippy::print_stderr)]

use std::time::{Duration, Instant};

pub use gravita_input::{Input, KeyCode, MouseButton};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

/// Window + timing configuration for [`run`].
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title.
    pub title: &'static str,
    /// Pixel buffer width.
    pub width: u32,
    /// Pixel buffer height.
    pub height: u32,
    /// Fixed simulation timestep in seconds. Defaults to `1/60`.
    pub fixed_timestep: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Gravita Example",
            width: 800,
            height: 600,
            fixed_timestep: 1.0 / 60.0,
        }
    }
}

/// User-supplied application logic driven by [`run`].
pub trait App: 'static {
    /// Step the simulation forward by `dt` seconds. Called zero or more times
    /// per rendered frame so each invocation receives a constant `dt`.
    fn update(&mut self, dt: f32, input: &Input);

    /// Draw the current state into the RGBA frame buffer.
    fn render(&self, frame: &mut [u8]);
}

struct Runner<A: App> {
    app: A,
    config: WindowConfig,
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    input: Input,
    last_time: Option<Instant>,
    accumulator: f32,
}

impl<A: App> Runner<A> {
    fn new(config: WindowConfig, app: A) -> Self {
        Self {
            app,
            config,
            window: None,
            pixels: None,
            input: Input::default(),
            last_time: None,
            accumulator: 0.0,
        }
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let size = LogicalSize::new(self.config.width as f64, self.config.height as f64);
        let attrs = WindowAttributes::default()
            .with_title(self.config.title)
            .with_inner_size(size)
            .with_min_inner_size(size);
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Box::leak(Box::new(w)),
            Err(err) => {
                eprintln!("create_window failed: {err}");
                event_loop.exit();
                return;
            },
        };
        let inner = window.inner_size();
        let surface = SurfaceTexture::new(inner.width, inner.height, &*window);
        let pixels = match Pixels::new(self.config.width, self.config.height, surface) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Pixels::new failed: {err}");
                event_loop.exit();
                return;
            },
        };
        self.window = Some(window);
        self.pixels = Some(pixels);
        self.last_time = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.input.set_close_requested(true);
                event_loop.exit();
            },
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
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    self.app.render(pixels.frame_mut());
                    if let Err(err) = pixels.render() {
                        eprintln!("pixels.render() failed: {err}");
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                if let Some(pixels) = self.pixels.as_mut()
                    && pixels
                        .resize_surface(new_size.width, new_size.height)
                        .is_err()
                {
                    eprintln!("pixels.resize_surface failed");
                    event_loop.exit();
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

        // Esc exits the demo (in addition to the OS close button).
        if self.input.key_pressed(KeyCode::Escape) {
            self.input.set_close_requested(true);
            event_loop.exit();
            return;
        }

        self.accumulator += dt;
        let step = self.config.fixed_timestep;
        // Cap the accumulator so a long pause doesn't trigger a spiral-of-death.
        if self.accumulator > step * 8.0 {
            self.accumulator = step * 8.0;
        }
        while self.accumulator >= step {
            self.app.update(step, &self.input);
            self.accumulator -= step;
        }

        // After running update, frame-edge inputs are consumed.
        self.input.begin_frame();

        if let Some(window) = self.window {
            window.request_redraw();
        }

        // Schedule the next wake just before the next frame is due so we don't
        // pin a CPU core spinning on `Poll`.
        let target = now + Duration::from_secs_f32(step);
        event_loop.set_control_flow(ControlFlow::WaitUntil(target));
    }
}

/// Run the example loop: create a window, drive `app` with fixed-timestep
/// updates, and render each frame.
pub fn run<A: App>(config: WindowConfig, app: A) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = Runner::new(config, app);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

// Backwards-compatible aliases — older example code referenced `ShimKeyCode`
// and `ShimMouseButton`. New code should use the bare `KeyCode` / `MouseButton`
// re-exported above.
pub use gravita_input::{KeyCode as ShimKeyCode, MouseButton as ShimMouseButton};
