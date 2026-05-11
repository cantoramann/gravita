//! Input state snapshot shared by Gravita example runners.
//!
//! [`Input`] is a small struct that tracks which keys and mouse buttons are
//! currently held + which were pressed/released this frame, plus the latest
//! cursor position. Both the 2D runner (`gravita-example-shim`) and the 3D
//! runner (`gravita-renderer-3d`) construct one of these per frame and pass
//! it to the user's `App::update`.
//!
//! # Reading inputs
//!
//! ```ignore
//! use gravita_input::{Input, KeyCode, MouseButton};
//!
//! fn update(input: &Input) {
//!     if input.key_pressed(KeyCode::Space) {
//!         // Jump
//!     }
//!     if input.key_held(KeyCode::ArrowLeft) {
//!         // Move left
//!     }
//!     if input.mouse_pressed(MouseButton::Left)
//!         && let Some((x, y)) = input.cursor()
//!     {
//!         // Spawn at (x, y)
//!     }
//! }
//! ```
//!
//! # Driving the state (runner-side)
//!
//! Runners call the `record_*` and `set_*` methods to update state in response
//! to `winit` window events, then `begin_frame` once per frame to clear
//! frame-edge events:
//!
//! ```ignore
//! input.record_key_press(KeyCode::Space);
//! // …
//! input.begin_frame(); // clears `keys_pressed_this_frame` etc.
//! ```

#![warn(missing_docs)]

use std::collections::HashSet;

pub use winit::{event::MouseButton, keyboard::KeyCode};

/// Snapshot of keyboard, mouse, and window state accessible during a frame's
/// `update` call.
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
    // ---------------------------------------------------------------------
    // Read-side (used by `App::update`)
    // ---------------------------------------------------------------------

    /// `true` while `key` is held down.
    #[must_use]
    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// `true` only on the frame `key` first went down.
    #[must_use]
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed_this_frame.contains(&key)
    }

    /// `true` only on the frame `key` was released.
    #[must_use]
    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keys_released_this_frame.contains(&key)
    }

    /// `true` while `button` is held.
    #[must_use]
    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }

    /// `true` only on the frame `button` first went down.
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

    // ---------------------------------------------------------------------
    // Write-side (runner-only)
    // ---------------------------------------------------------------------

    /// Record that `key` went down. Updates `held` and `pressed_this_frame`
    /// (idempotent — repeat key-down events from the OS don't re-trigger
    /// `pressed_this_frame`).
    pub fn record_key_press(&mut self, key: KeyCode) {
        if self.keys_held.insert(key) {
            self.keys_pressed_this_frame.insert(key);
        }
    }

    /// Record that `key` was released. Updates `held` and `released_this_frame`.
    pub fn record_key_release(&mut self, key: KeyCode) {
        if self.keys_held.remove(&key) {
            self.keys_released_this_frame.insert(key);
        }
    }

    /// Record that `button` went down.
    pub fn record_mouse_press(&mut self, button: MouseButton) {
        if self.mouse_held.insert(button) {
            self.mouse_pressed_this_frame.insert(button);
        }
    }

    /// Record that `button` was released.
    pub fn record_mouse_release(&mut self, button: MouseButton) {
        self.mouse_held.remove(&button);
    }

    /// Set the latest cursor position.
    pub fn set_cursor(&mut self, pos: Option<(f32, f32)>) {
        self.cursor = pos;
    }

    /// Set the close-requested flag.
    pub fn set_close_requested(&mut self, requested: bool) {
        self.close_requested = requested;
    }

    /// Clear the frame-edge state (pressed-this-frame, released-this-frame).
    /// Held state persists.
    pub fn begin_frame(&mut self) {
        self.keys_pressed_this_frame.clear();
        self.keys_released_this_frame.clear();
        self.mouse_pressed_this_frame.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_then_release_clears_held() {
        let mut input = Input::default();
        input.record_key_press(KeyCode::Space);
        assert!(input.key_held(KeyCode::Space));
        assert!(input.key_pressed(KeyCode::Space));
        input.record_key_release(KeyCode::Space);
        assert!(!input.key_held(KeyCode::Space));
        assert!(input.key_released(KeyCode::Space));
    }

    #[test]
    fn begin_frame_clears_edges_but_not_held() {
        let mut input = Input::default();
        input.record_key_press(KeyCode::KeyA);
        input.begin_frame();
        assert!(input.key_held(KeyCode::KeyA));
        assert!(!input.key_pressed(KeyCode::KeyA));
    }

    #[test]
    fn repeated_key_press_does_not_retrigger_pressed_this_frame() {
        let mut input = Input::default();
        input.record_key_press(KeyCode::Space);
        input.begin_frame();
        // OS key-repeat: held is still true, but a re-press should not flip
        // pressed_this_frame again until the key goes up + down.
        input.record_key_press(KeyCode::Space);
        assert!(input.key_held(KeyCode::Space));
        assert!(!input.key_pressed(KeyCode::Space));
    }

    #[test]
    fn cursor_round_trip() {
        let mut input = Input::default();
        assert_eq!(input.cursor(), None);
        input.set_cursor(Some((100.0, 50.0)));
        assert_eq!(input.cursor(), Some((100.0, 50.0)));
        input.set_cursor(None);
        assert_eq!(input.cursor(), None);
    }

    #[test]
    fn close_requested_round_trip() {
        let mut input = Input::default();
        assert!(!input.close_requested());
        input.set_close_requested(true);
        assert!(input.close_requested());
    }

    #[test]
    fn mouse_press_then_release() {
        let mut input = Input::default();
        input.record_mouse_press(MouseButton::Left);
        assert!(input.mouse_held(MouseButton::Left));
        assert!(input.mouse_pressed(MouseButton::Left));
        input.record_mouse_release(MouseButton::Left);
        assert!(!input.mouse_held(MouseButton::Left));
    }
}
