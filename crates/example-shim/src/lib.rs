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

use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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

/// Snapshot of keyboard, mouse, and window state accessible during
/// [`App::update`].
#[derive(Debug, Default)]
pub struct Input {
    keys_held: HashSet<KeyCode>,
    keys_pressed_this_frame: HashSet<KeyCode>,
    keys_released_this_frame: HashSet<KeyCode>,
    mouse_held: HashSet<MouseButton>,
    mouse_pressed_this_frame: HashSet<MouseButton>,
    cursor: Option<(f32, f32)>,
    close_requested: bool,
}

impl Input {
    /// `true` while the key is held down.
    #[must_use]
    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// `true` only on the frame the key first went down.
    #[must_use]
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed_this_frame.contains(&key)
    }

    /// `true` only on the frame the key was released.
    #[must_use]
    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keys_released_this_frame.contains(&key)
    }

    /// `true` while the mouse button is held.
    #[must_use]
    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }

    /// `true` only on the frame the button first went down.
    #[must_use]
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed_this_frame.contains(&button)
    }

    /// Last reported cursor position in logical pixels, if any.
    #[must_use]
    pub fn cursor(&self) -> Option<(f32, f32)> {
        self.cursor
    }

    /// `true` if the user requested window close (X button or Esc).
    #[must_use]
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn begin_frame(&mut self) {
        self.keys_pressed_this_frame.clear();
        self.keys_released_this_frame.clear();
        self.mouse_pressed_this_frame.clear();
    }
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
                error!("create_window failed: {err}");
                event_loop.exit();
                return;
            },
        };
        let inner = window.inner_size();
        let surface = SurfaceTexture::new(inner.width, inner.height, &*window);
        let pixels = match Pixels::new(self.config.width, self.config.height, surface) {
            Ok(p) => p,
            Err(err) => {
                error!("Pixels::new failed: {err}");
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
                self.input.close_requested = true;
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        ElementState::Pressed => {
                            if !key_event.repeat && self.input.keys_held.insert(code) {
                                self.input.keys_pressed_this_frame.insert(code);
                            }
                        },
                        ElementState::Released => {
                            if self.input.keys_held.remove(&code) {
                                self.input.keys_released_this_frame.insert(code);
                            }
                        },
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input.cursor = Some((position.x as f32, position.y as f32));
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if self.input.mouse_held.insert(button) {
                        self.input.mouse_pressed_this_frame.insert(button);
                    }
                },
                ElementState::Released => {
                    self.input.mouse_held.remove(&button);
                },
            },
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    self.app.render(pixels.frame_mut());
                    if let Err(err) = pixels.render() {
                        error!("pixels.render() failed: {err}");
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                if let Some(pixels) = self.pixels.as_mut()
                    && pixels.resize_surface(new_size.width, new_size.height).is_err()
                {
                    error!("pixels.resize_surface failed");
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
            self.input.close_requested = true;
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
    // Best-effort: examples are often re-run within the same process during dev.
    let _ = env_logger::try_init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = Runner::new(config, app);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

// Re-exports so examples don't need to add winit to their own deps.
pub use winit::event::MouseButton as ShimMouseButton;
pub use winit::keyboard::KeyCode as ShimKeyCode;
