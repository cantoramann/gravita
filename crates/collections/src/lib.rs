//! Reusable higher-level objects built on top of Gravita primitives.
//!
//! This crate provides ready-made game objects that combine physics,
//! rendering, and gameplay logic. Use these as starting points for
//! your own game characters and entities.
//!
//! # Available Objects
//!
//! - [`Stickman`] — Animated humanoid character with walking and jumping
//! - [`Spaceship`] — Thrust-based vehicle with rotation controls
//! - [`Planet`] — Static celestial body for orbital mechanics demos
//!
//! # Example
//!
//! ```ignore
//! use gravita_collections::Stickman;
//!
//! let mut stickman = Stickman::new(ground_y, screen_width);
//!
//! // Game loop
//! stickman.set_move_direction(1.0); // Move right
//! stickman.jump();
//! stickman.update(dt, screen_width);
//! stickman.render(&mut frame, width, height);
//! ```

#![warn(missing_docs)]

mod planet;
mod spaceship;
mod stickman;

pub use planet::Planet;
pub use spaceship::Spaceship;
pub use stickman::Stickman;

/// Anything that can render itself into an RGBA frame buffer.
///
/// Implemented by every game object in this crate ([`Stickman`], [`Spaceship`],
/// [`Planet`]). This lets example runners blit a heterogeneous scene with a
/// single iteration over `&dyn Drawable`.
pub trait Drawable {
    /// Render this object into `frame` (`width` × `height` RGBA pixels).
    fn render(&self, frame: &mut [u8], width: u32, height: u32);
}

impl Drawable for Stickman {
    #[inline]
    fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        Self::render(self, frame, width, height);
    }
}

impl Drawable for Spaceship {
    #[inline]
    fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        Self::render(self, frame, width, height);
    }
}

impl Drawable for Planet {
    #[inline]
    fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        Self::render(self, frame, width, height);
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::Vec2;

    use super::*;

    #[test]
    fn drawable_trait_object_dispatch() {
        // Scene with one of each kind, accessed via `&dyn Drawable`. This is
        // the pattern example runners are expected to use.
        let scene: Vec<Box<dyn Drawable>> = vec![
            Box::new(Stickman::new(0.0, 800.0)),
            Box::new(Spaceship::new(Vec2::new(400.0, 300.0))),
            Box::new(Planet::new(Vec2::new(400.0, 300.0), 100.0)),
        ];
        let mut frame = vec![0u8; 800 * 600 * 4];
        for entity in &scene {
            entity.render(&mut frame, 800, 600);
        }
        // Frame size should not have changed; this primarily tests that the
        // trait can be used dynamically without panicking.
        assert_eq!(frame.len(), 800 * 600 * 4);
    }
}
